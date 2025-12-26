mod scoring;

fn main() {
    use petgraph::graph::Graph;
    use crate::scoring::page_rank_scores;

    let mut g: Graph<&str, u32> = Graph::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    g.extend_with_edges(&[(a, b), (a, c), (b, c), (c, a)]);

    let ranks = page_rank_scores(&g, 0.85_f64, 20);
    for (label, score) in ranks {
        println!("{label}: {score:.6}");
    }
}
