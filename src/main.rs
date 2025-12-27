use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use petgraph::algo::page_rank;

use cruxlines::find_references::{find_references, ReferenceEdge};
use cruxlines::graph::build_reference_graphs_by_language;
use cruxlines::scoring::sort_by_rank_desc;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'u', long = "references")]
    references: bool,
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let mut inputs = Vec::with_capacity(cli.files.len());
    for path in &cli.files {
        if path.is_dir() {
            continue;
        }
        if cruxlines::format_router::language_for_path(path).is_none() {
            continue;
        }
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("cruxlines: failed to read {}: {err}", path.display());
                process::exit(1);
            }
        };
        let contents = match String::from_utf8(bytes) {
            Ok(contents) => contents,
            Err(_) => {
                continue;
            }
        };
        inputs.push((path.clone(), contents));
    }

    let mut edges: Vec<ReferenceEdge> = find_references(inputs).collect();
    edges.sort_by(|a, b| {
        let key_a = (
            &a.definition.path,
            a.definition.line,
            a.definition.column,
            &a.definition.name,
            &a.usage.path,
            a.usage.line,
            a.usage.column,
            &a.usage.name,
        );
        let key_b = (
            &b.definition.path,
            b.definition.line,
            b.definition.column,
            &b.definition.name,
            &b.usage.path,
            b.usage.line,
            b.usage.column,
            &b.usage.name,
        );
        key_a.cmp(&key_b)
    });

    let graphs = build_reference_graphs_by_language(&edges);
    let mut ranks_by_location: HashMap<cruxlines::find_references::Location, f64> = HashMap::new();
    for (_language, (graph, indices)) in graphs {
        let ranks = page_rank(&graph, 0.85_f64, 20);
        for (location, idx) in indices {
            ranks_by_location.insert(location, ranks[idx.index()]);
        }
    }

    let mut grouped: HashMap<cruxlines::find_references::Location, Vec<cruxlines::find_references::Location>> =
        HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, usages) in grouped {
        let rank = ranks_by_location
            .get(&definition)
            .copied()
            .unwrap_or(0.0);
        output_rows.push((rank, definition, usages));
    }

    sort_by_rank_desc(&mut output_rows);

    for (rank, definition, mut usages) in output_rows {
        usages.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        if cli.references {
            println!(
                "{:.6}\t{}\t{}:{}:{}{}",
                rank,
                definition.name,
                definition.path.display(),
                definition.line,
                definition.column,
                format_usage_list(&usages)
            );
        } else {
            println!(
                "{:.6}\t{}\t{}:{}:{}",
                rank,
                definition.name,
                definition.path.display(),
                definition.line,
                definition.column
            );
        }
    }
}

fn format_usage_list(usages: &[cruxlines::find_references::Location]) -> String {
    let mut out = String::new();
    for usage in usages {
        out.push('\t');
        out.push_str(&format!(
            "{}:{}:{}",
            usage.path.display(),
            usage.line,
            usage.column
        ));
    }
    out
}
