fn main() {
    use petgraph::algo::page_rank;
    use petgraph::graph::Graph;

    let mut g: Graph<&str, u32> = Graph::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    g.extend_with_edges(&[(a, b), (a, c), (b, c), (c, a)]);

    let ranks = page_rank(&g, 0.85_f64, 20);
    for node in g.node_indices() {
        println!("{}: {:.6}", g[node], ranks[node.index()]);
    }
}
