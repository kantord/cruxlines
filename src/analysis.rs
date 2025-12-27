use std::collections::HashMap;
use std::path::PathBuf;

use petgraph::algo::page_rank;

use crate::find_references::{find_references, Location, ReferenceEdge};
use crate::graph::build_reference_graphs_by_language;

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub rank: f64,
    pub definition: Location,
    pub references: Vec<Location>,
}

pub fn cruxlines<I>(inputs: I) -> Vec<OutputRow>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();

    let graphs = build_reference_graphs_by_language(&edges);
    let mut ranks_by_location: HashMap<Location, f64> = HashMap::new();
    for (_language, (graph, indices)) in graphs {
        let ranks = page_rank(&graph, 0.85_f64, 20);
        for (location, idx) in indices {
            ranks_by_location.insert(location, ranks[idx.index()]);
        }
    }

    let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, mut references) in grouped {
        references.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        let rank = ranks_by_location
            .get(&definition)
            .copied()
            .unwrap_or(0.0);
        output_rows.push(OutputRow {
            rank,
            definition,
            references,
        });
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    output_rows
}

#[cfg(test)]
mod tests {
    use super::cruxlines;
    use std::path::PathBuf;

    #[test]
    fn analyze_paths_produces_rows() {
        let files = vec![
            (
                PathBuf::from("fixtures/python/main.py"),
                std::fs::read_to_string("fixtures/python/main.py").expect("read"),
            ),
            (
                PathBuf::from("fixtures/python/utils.py"),
                std::fs::read_to_string("fixtures/python/utils.py").expect("read"),
            ),
            (
                PathBuf::from("fixtures/python/models.py"),
                std::fs::read_to_string("fixtures/python/models.py").expect("read"),
            ),
        ];
        let rows = cruxlines(files);
        assert!(!rows.is_empty());
        assert!(rows.iter().any(|row| row.definition.name == "add"));
    }
}
