fn main() {
    use petgraph::algo::dijkstra;
    use petgraph::graph::Graph;

    let mut g = Graph::<&str, u32>::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    g.add_edge(a, b, 1);

    let paths = dijkstra(&g, a, None, |e| *e.weight());
    println!("{paths:?}");
}
