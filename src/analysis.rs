use std::collections::HashMap;
use std::path::PathBuf;

use crate::find_references::{find_references, Location, ReferenceEdge};
use crate::graph::build_file_graph;

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub rank: f64,
    pub local_score: f64,
    pub file_rank: f64,
    pub definition: Location,
    pub references: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct ProfileStats {
    pub parse_ms: u128,
    pub frecency_ms: u128,
    pub file_rank_ms: u128,
    pub score_ms: u128,
    pub definitions: usize,
    pub references: usize,
}

pub fn cruxlines<I>(inputs: I) -> Vec<OutputRow>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let inputs: Vec<(PathBuf, String)> = inputs.into_iter().collect();
    let frecency = frecency_scores(&inputs);
    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();

    let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }

    let file_ranks = rank_files(&grouped);

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for definition in grouped.keys() {
        *name_counts.entry(definition.name.clone()).or_default() += 1;
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, mut references) in grouped {
        references.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        let name_count = name_counts
            .get(&definition.name)
            .copied()
            .unwrap_or(1) as f64;
        let weighted_refs: f64 = references
            .iter()
            .map(|reference| {
                let file_rank = file_ranks
                    .get(&reference.path)
                    .copied()
                    .unwrap_or(0.0);
                let frecency = frecency.get(&reference.path).copied().unwrap_or(1.0);
                file_rank * frecency
            })
            .sum();
        let local_score = weighted_refs / name_count;
        let file_rank = file_ranks
            .get(&definition.path)
            .copied()
            .unwrap_or(0.0);
        let rank = local_score * file_rank;
        output_rows.push(OutputRow {
            rank,
            local_score,
            file_rank,
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

pub fn cruxlines_profiled<I>(inputs: I) -> (Vec<OutputRow>, ProfileStats)
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let start_frecency = std::time::Instant::now();
    let inputs: Vec<(PathBuf, String)> = inputs.into_iter().collect();
    let frecency = frecency_scores(&inputs);
    let frecency_ms = start_frecency.elapsed().as_millis();

    let start_parse = std::time::Instant::now();
    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();
    let parse_ms = start_parse.elapsed().as_millis();

    let start_file_rank = std::time::Instant::now();
    let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }
    let file_ranks = rank_files(&grouped);
    let file_rank_ms = start_file_rank.elapsed().as_millis();

    let start_score = std::time::Instant::now();
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for definition in grouped.keys() {
        *name_counts.entry(definition.name.clone()).or_default() += 1;
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, mut references) in grouped {
        references.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        let name_count = name_counts
            .get(&definition.name)
            .copied()
            .unwrap_or(1) as f64;
        let weighted_refs: f64 = references
            .iter()
            .map(|reference| {
                let file_rank = file_ranks
                    .get(&reference.path)
                    .copied()
                    .unwrap_or(0.0);
                let frecency = frecency.get(&reference.path).copied().unwrap_or(1.0);
                file_rank * frecency
            })
            .sum();
        let local_score = weighted_refs / name_count;
        let file_rank = file_ranks
            .get(&definition.path)
            .copied()
            .unwrap_or(0.0);
        let rank = local_score * file_rank;
        output_rows.push(OutputRow {
            rank,
            local_score,
            file_rank,
            definition,
            references,
        });
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let score_ms = start_score.elapsed().as_millis();

    let definitions = output_rows.len();
    let references = output_rows.iter().map(|row| row.references.len()).sum();

    (
        output_rows,
        ProfileStats {
            parse_ms,
            frecency_ms,
            file_rank_ms,
            score_ms,
            definitions,
            references,
        },
    )
}

fn rank_files(grouped: &HashMap<Location, Vec<Location>>) -> HashMap<PathBuf, f64> {
    let (graph, indices) = build_file_graph(grouped);
    if graph.node_count() == 0 {
        return HashMap::new();
    }
    let ranks = petgraph::algo::page_rank(&graph, 0.85_f64, 20);
    let mut out = HashMap::new();
    for (path, idx) in indices {
        out.insert(path, ranks[idx.index()]);
    }
    out
}

fn frecency_scores(inputs: &[(PathBuf, String)]) -> HashMap<PathBuf, f64> {
    let Some(repo_root) = find_repo_root(inputs) else {
        return HashMap::new();
    };
    let Ok(scores) = frecenfile::analyze_repo(&repo_root, None, None) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (path, score) in scores {
        out.insert(repo_root.join(path), score);
    }
    out
}

fn find_repo_root(inputs: &[(PathBuf, String)]) -> Option<PathBuf> {
    let first = inputs.first()?.0.as_path();
    for ancestor in first.ancestors() {
        let git_dir = ancestor.join(".git");
        if git_dir.is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
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

    #[test]
    fn scores_are_normalized_by_definition_count() {
        let inputs = vec![
            (
                PathBuf::from("a.py"),
                "def foo():\n    pass\n\ndef foo():\n    pass\n\ndef bar():\n    pass\n".to_string(),
            ),
            (
                PathBuf::from("c.py"),
                "from a import foo, bar\n\nfoo()\nbar()\n".to_string(),
            ),
        ];
        let rows = cruxlines(inputs);
        let foo_scores: Vec<f64> = rows
            .iter()
            .filter(|row| row.definition.name == "foo")
            .map(|row| row.rank)
            .collect();
        let bar_score = rows
            .iter()
            .find(|row| row.definition.name == "bar")
            .map(|row| row.rank)
            .unwrap_or(0.0);
        assert_eq!(foo_scores.len(), 2);
        for score in foo_scores {
            assert!((bar_score - score * 2.0).abs() < 1e-6, "score was {score}");
        }
    }

    #[test]
    fn file_rank_influences_score() {
        let inputs = vec![
            (
                PathBuf::from("a.py"),
                "def foo():\n    pass\n".to_string(),
            ),
            (
                PathBuf::from("b.py"),
                "def foo():\n    pass\n".to_string(),
            ),
            (
                PathBuf::from("c.py"),
                "from a import foo\nfrom b import foo\n\nfoo()\n".to_string(),
            ),
        ];
        let rows = cruxlines(inputs);
        let a_score = rows
            .iter()
            .find(|row| row.definition.path.ends_with("a.py"))
            .map(|row| row.rank)
            .unwrap_or(0.0);
        let b_score = rows
            .iter()
            .find(|row| row.definition.path.ends_with("b.py"))
            .map(|row| row.rank)
            .unwrap_or(0.0);
        assert!(a_score > 0.0);
        assert!(b_score > 0.0);
    }
}
