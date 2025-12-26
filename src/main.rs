use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use petgraph::algo::page_rank;
use petgraph::graph::Graph;

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

    let mut graph: Graph<cruxlines::find_references::Location, ()> = Graph::new();
    let mut indices = HashMap::new();
    for edge in &edges {
        let def_idx = node_index(&mut graph, &mut indices, &edge.definition);
        let use_idx = node_index(&mut graph, &mut indices, &edge.usage);
        graph.add_edge(use_idx, def_idx, ());
    }
    let ranks = page_rank(&graph, 0.85_f64, 20);

    let mut output_rows = Vec::with_capacity(edges.len());
    for edge in edges {
        let def_idx = indices
            .get(&edge.definition)
            .expect("definition index missing");
        let rank = ranks[def_idx.index()];
        output_rows.push((rank, edge));
    }

    output_rows.sort_by(|a, b| {
        b.0
            .partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (rank, edge) in output_rows {
        println!(
            "{:.6}\t{}\t{}:{}:{}\t{}:{}:{}",
            rank,
            edge.definition.name,
            edge.definition.path.display(),
            edge.definition.line,
            edge.definition.column,
            edge.usage.path.display(),
            edge.usage.line,
            edge.usage.column
        );
    }
}

fn node_index(
    graph: &mut Graph<cruxlines::find_references::Location, ()>,
    indices: &mut HashMap<cruxlines::find_references::Location, petgraph::graph::NodeIndex>,
    location: &cruxlines::find_references::Location,
) -> petgraph::graph::NodeIndex {
    if let Some(index) = indices.get(location) {
        *index
    } else {
        let index = graph.add_node(location.clone());
        indices.insert(location.clone(), index);
        index
    }
}
