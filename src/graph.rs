use std::collections::HashMap;

use petgraph::graph::{Graph, NodeIndex};

use crate::find_references::{Location, ReferenceEdge};

pub fn build_reference_graph(
    edges: &[ReferenceEdge],
) -> (Graph<Location, ()>, HashMap<Location, NodeIndex>) {
    let mut graph: Graph<Location, ()> = Graph::new();
    let mut indices = HashMap::new();
    for edge in edges {
        let def_idx = node_index(&mut graph, &mut indices, &edge.definition);
        let use_idx = node_index(&mut graph, &mut indices, &edge.usage);
        graph.add_edge(use_idx, def_idx, ());
    }
    (graph, indices)
}

fn node_index(
    graph: &mut Graph<Location, ()>,
    indices: &mut HashMap<Location, NodeIndex>,
    location: &Location,
) -> NodeIndex {
    if let Some(index) = indices.get(location) {
        *index
    } else {
        let index = graph.add_node(location.clone());
        indices.insert(location.clone(), index);
        index
    }
}

pub fn build_reference_graphs_by_language(
    edges: &[ReferenceEdge],
) -> HashMap<crate::find_references::Language, (Graph<Location, ()>, HashMap<Location, NodeIndex>)> {
    let mut grouped: HashMap<crate::find_references::Language, Vec<ReferenceEdge>> = HashMap::new();
    for edge in edges {
        let Some(language) = crate::format_router::language_for_path(&edge.definition.path) else {
            continue;
        };
        grouped.entry(language).or_default().push(edge.clone());
    }

    let mut graphs = HashMap::new();
    for (language, language_edges) in grouped {
        graphs.insert(language, build_reference_graph(&language_edges));
    }
    graphs
}

#[cfg(test)]
mod tests {
    use super::build_reference_graph;
    use super::build_reference_graphs_by_language;
    use crate::find_references::{Location, ReferenceEdge};
    use std::path::PathBuf;

    #[test]
    fn builds_usage_to_definition_edges() {
        let def = Location {
            path: PathBuf::from("def.py"),
            line: 1,
            column: 1,
            name: "foo".to_string(),
        };
        let usage = Location {
            path: PathBuf::from("use.py"),
            line: 2,
            column: 5,
            name: "foo".to_string(),
        };
        let edges = vec![ReferenceEdge {
            definition: def.clone(),
            usage: usage.clone(),
        }];

        let (graph, indices) = build_reference_graph(&edges);
        let def_idx = indices.get(&def).expect("def node");
        let use_idx = indices.get(&usage).expect("usage node");
        assert!(graph.contains_edge(*use_idx, *def_idx));
    }

    #[test]
    fn builds_graphs_per_language() {
        let py_def = Location {
            path: PathBuf::from("a.py"),
            line: 1,
            column: 1,
            name: "foo".to_string(),
        };
        let py_use = Location {
            path: PathBuf::from("b.py"),
            line: 2,
            column: 1,
            name: "foo".to_string(),
        };
        let rs_def = Location {
            path: PathBuf::from("a.rs"),
            line: 1,
            column: 1,
            name: "bar".to_string(),
        };
        let rs_use = Location {
            path: PathBuf::from("b.rs"),
            line: 2,
            column: 1,
            name: "bar".to_string(),
        };
        let edges = vec![
            ReferenceEdge {
                definition: py_def,
                usage: py_use,
            },
            ReferenceEdge {
                definition: rs_def,
                usage: rs_use,
            },
        ];

        let graphs = build_reference_graphs_by_language(&edges);
        assert_eq!(graphs.len(), 2);
    }
}
