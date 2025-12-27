pub fn sort_by_rank_desc<T, U>(rows: &mut Vec<(f64, T, U)>) {
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
}

#[cfg(test)]
mod tests {
    #[test]
    fn sort_by_rank_desc_orders_scores() {
        let mut rows = vec![(0.1, "a", 1), (0.3, "b", 2), (0.2, "c", 3)];
        super::sort_by_rank_desc(&mut rows);
        let scores: Vec<f64> = rows.iter().map(|(score, _, _)| *score).collect();
        assert_eq!(scores, vec![0.3, 0.2, 0.1]);
    }
}
