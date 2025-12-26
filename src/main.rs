use std::path::PathBuf;
use std::process;

use clap::Parser;

use cruxlines::find_references::{find_references, ReferenceEdge};

#[derive(Debug, Parser)]
struct Cli {
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

    for edge in edges {
        println!(
            "{}:{}:{}:{} -> {}:{}:{}:{}",
            edge.definition.path.display(),
            edge.definition.line,
            edge.definition.column,
            edge.definition.name,
            edge.usage.path.display(),
            edge.usage.line,
            edge.usage.column,
            edge.usage.name
        );
    }
}
