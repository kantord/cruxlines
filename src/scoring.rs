use petgraph::algo::page_rank;
use petgraph::graph::Graph;

pub fn page_rank_scores<'a>(
    graph: &'a Graph<&'a str, u32>,
    damping: f64,
    iterations: usize,
) -> Vec<(&'a str, f64)> {
    let ranks = page_rank(graph, damping, iterations);
    let mut out = Vec::with_capacity(ranks.len());
    for node in graph.node_indices() {
        out.push((graph[node], ranks[node.index()]));
    }
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

#[cfg(test)]
mod tests {
    use super::page_rank_scores;
    use petgraph::graph::Graph;

    #[test]
    fn page_rank_scores_are_descending() {
        let mut g: Graph<&str, u32> = Graph::new();
        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        g.extend_with_edges(&[(a, b), (a, c), (b, c), (c, a)]);

        let scores = page_rank_scores(&g, 0.85_f64, 20);
        let is_desc = scores
            .windows(2)
            .all(|pair| pair[0].1 >= pair[1].1);
        assert!(is_desc, "scores are not in descending order: {scores:?}");
    }
}
