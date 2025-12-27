use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process;

use clap::Parser;
use ignore::WalkBuilder;
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
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            eprintln!("cruxlines: failed to read current dir: {err}");
            process::exit(1);
        }
    };
    let mut requested_files: HashSet<PathBuf> = HashSet::new();
    let mut requested_dirs: HashSet<PathBuf> = HashSet::new();
    for path in &cli.files {
        let abs = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        if abs.is_dir() {
            requested_dirs.insert(abs);
        } else {
            requested_files.insert(abs);
        }
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    for dir in &requested_dirs {
        roots.push(dir.clone());
    }
    for file in &requested_files {
        if let Some(parent) = file.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    if roots.is_empty() {
        return;
    }
    roots.sort();
    roots.dedup();

    let mut builder = WalkBuilder::new(&roots[0]);
    for path in &roots[1..] {
        builder.add(path);
    }
    let requested_files = requested_files;
    let requested_dirs = requested_dirs;
    builder.filter_entry(move |entry| {
        let path = entry.path();
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        };
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            requested_dirs.iter().any(|dir| dir.starts_with(&abs))
                || requested_files.iter().any(|file| file.starts_with(&abs))
        } else {
            requested_files.contains(&abs)
                || requested_dirs.iter().any(|dir| abs.starts_with(dir))
        }
    });

    let mut inputs = Vec::new();
    for entry in builder.build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry
            .file_type()
            .map(|file_type| file_type.is_file())
            .unwrap_or(false)
        {
            continue;
        }
        let path = entry.path();
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
        inputs.push((path.to_path_buf(), contents));
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
