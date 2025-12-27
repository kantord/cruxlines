use std::collections::HashMap;
use std::path::PathBuf;

use petgraph::graph::{Graph, NodeIndex};

use crate::find_references::Location;

pub fn build_file_graph(
    grouped: &HashMap<Location, Vec<Location>>,
) -> (Graph<PathBuf, ()>, HashMap<PathBuf, NodeIndex>) {
    let mut graph: Graph<PathBuf, ()> = Graph::new();
    let mut indices: HashMap<PathBuf, NodeIndex> = HashMap::new();
    for (definition, usages) in grouped {
        let def_idx = node_index(&mut graph, &mut indices, &definition.path);
        for usage in usages {
            if usage.path == definition.path {
                continue;
            }
            let use_idx = node_index(&mut graph, &mut indices, &usage.path);
            graph.add_edge(use_idx, def_idx, ());
        }
    }
    (graph, indices)
}

fn node_index(
    graph: &mut Graph<PathBuf, ()>,
    indices: &mut HashMap<PathBuf, NodeIndex>,
    path: &PathBuf,
) -> NodeIndex {
    if let Some(index) = indices.get(path) {
        *index
    } else {
        let index = graph.add_node(path.clone());
        indices.insert(path.clone(), index);
        index
    }
}

#[cfg(test)]
mod tests {
    use super::build_file_graph;
    use crate::find_references::Location;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn builds_file_graph_with_cross_file_edges() {
        let def = Location {
            path: PathBuf::from("a.py"),
            line: 1,
            column: 1,
            name: "foo".to_string(),
        };
        let usage = Location {
            path: PathBuf::from("b.py"),
            line: 2,
            column: 1,
            name: "foo".to_string(),
        };
        let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
        grouped.insert(def.clone(), vec![usage.clone()]);

        let (graph, indices) = build_file_graph(&grouped);
        let def_idx = indices.get(&def.path).expect("def node");
        let use_idx = indices.get(&usage.path).expect("use node");
        assert!(graph.contains_edge(*use_idx, *def_idx));
    }
}
