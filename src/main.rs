use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use petgraph::algo::page_rank;

use cruxlines::find_references::{find_references, ReferenceEdge};
use cruxlines::graph::build_reference_graph;
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
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("cruxlines: failed to read {}: {err}", path.display());
                process::exit(1);
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

    let (graph, indices) = build_reference_graph(&edges);
    let ranks = page_rank(&graph, 0.85_f64, 20);

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
        let def_idx = indices
            .get(&definition)
            .expect("definition index missing");
        let rank = ranks[def_idx.index()];
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
