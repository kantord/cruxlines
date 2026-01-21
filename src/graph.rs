use std::collections::HashMap;

use lasso::Spur;
use petgraph::graph::{Graph, NodeIndex};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::find_references::Location;

pub fn build_file_graph(
    grouped: &HashMap<Location, Vec<Location>>,
) -> (Graph<Spur, ()>, FxHashMap<Spur, NodeIndex>) {
    let mut graph: Graph<Spur, ()> = Graph::new();
    let mut indices: FxHashMap<Spur, NodeIndex> = FxHashMap::default();
    // Track existing edges to avoid duplicates
    let mut existing_edges: FxHashSet<(NodeIndex, NodeIndex)> = FxHashSet::default();

    for (definition, usages) in grouped {
        let def_idx = node_index(&mut graph, &mut indices, definition.path);
        for usage in usages {
            if usage.path == definition.path {
                continue;
            }
            let use_idx = node_index(&mut graph, &mut indices, usage.path);
            // Only add edge if it doesn't already exist
            if existing_edges.insert((use_idx, def_idx)) {
                graph.add_edge(use_idx, def_idx, ());
            }
        }
    }
    (graph, indices)
}

fn node_index(
    graph: &mut Graph<Spur, ()>,
    indices: &mut FxHashMap<Spur, NodeIndex>,
    path: Spur,
) -> NodeIndex {
    if let Some(index) = indices.get(&path) {
        *index
    } else {
        let index = graph.add_node(path);
        indices.insert(path, index);
        index
    }
}

#[cfg(test)]
mod tests {
    use super::build_file_graph;
    use crate::find_references::Location;
    use crate::intern::intern;
    use std::collections::HashMap;

    #[test]
    fn builds_file_graph_with_cross_file_edges() {
        let def = Location {
            path: intern("a.py"),
            line: 1,
            column: 1,
            name: intern("foo"),
        };
        let usage = Location {
            path: intern("b.py"),
            line: 2,
            column: 1,
            name: intern("foo"),
        };
        let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
        grouped.insert(def, vec![usage]);

        let (graph, indices) = build_file_graph(&grouped);
        let def_idx = indices.get(&def.path).expect("def node");
        let use_idx = indices.get(&usage.path).expect("use node");
        assert!(graph.contains_edge(*use_idx, *def_idx));
    }
}
