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

#[cfg(test)]
mod tests {
    use super::build_reference_graph;
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
}
