use std::collections::HashMap;
use std::path::PathBuf;

use crate::find_references::{find_references, Location, ReferenceEdge};
use crate::graph::build_file_graph;
use crate::languages::{language_for_path, Language};

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub rank: f64,
    pub local_score: f64,
    pub file_rank: f64,
    pub definition: Location,
    pub references: Vec<Location>,
}

pub fn cruxlines<I>(inputs: I) -> Vec<OutputRow>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let inputs: Vec<(PathBuf, String)> = inputs.into_iter().collect();
    let (edges, frecency) = compute_edges_and_frecency(inputs, |paths| frecency_scores(&paths));

    let grouped_by_language = group_edges_by_language(edges);
    let capacity: usize = grouped_by_language.values().map(|grouped| grouped.len()).sum();
    let mut output_rows = Vec::with_capacity(capacity);
    for grouped in grouped_by_language.into_values() {
        let file_ranks = rank_files(&grouped);

        let mut name_counts: HashMap<String, usize> = HashMap::new();
        for definition in grouped.keys() {
            *name_counts.entry(definition.name.clone()).or_default() += 1;
        }

        output_rows.extend(build_rows(grouped, &file_ranks, &frecency, &name_counts));
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    output_rows
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

fn compute_edges_and_frecency<F>(
    inputs: Vec<(PathBuf, String)>,
    frecency_fn: F,
) -> (Vec<ReferenceEdge>, HashMap<PathBuf, f64>)
where
    F: FnOnce(Vec<PathBuf>) -> HashMap<PathBuf, f64> + Send + 'static,
{
    let paths: Vec<PathBuf> = inputs.iter().map(|(path, _)| path.clone()).collect();
    let frecency_handle = std::thread::spawn(move || frecency_fn(paths));

    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();

    match frecency_handle.join() {
        Ok(frecency) => (edges, frecency),
        Err(_) => (edges, HashMap::new()),
    }
}

fn build_rows(
    grouped: HashMap<Location, Vec<Location>>,
    file_ranks: &HashMap<PathBuf, f64>,
    frecency: &HashMap<PathBuf, f64>,
    name_counts: &HashMap<String, usize>,
) -> Vec<OutputRow> {
    let mut rows = Vec::with_capacity(grouped.len());
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
        rows.push(OutputRow {
            rank,
            local_score,
            file_rank,
            definition,
            references,
        });
    }
    rows
}

fn group_edges_by_language(
    edges: Vec<ReferenceEdge>,
) -> HashMap<Language, HashMap<Location, Vec<Location>>> {
    let mut grouped_by_language: HashMap<Language, HashMap<Location, Vec<Location>>> = HashMap::new();
    for edge in edges {
        let Some(language) = language_for_path(&edge.definition.path) else {
            continue;
        };
        grouped_by_language
            .entry(language)
            .or_default()
            .entry(edge.definition)
            .or_default()
            .push(edge.usage);
    }
    grouped_by_language
}

fn frecency_scores(paths: &[PathBuf]) -> HashMap<PathBuf, f64> {
    let Some(repo_root) = find_repo_root(paths) else {
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

fn find_repo_root(paths: &[PathBuf]) -> Option<PathBuf> {
    let first = paths.first()?.as_path();
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
    use super::{compute_edges_and_frecency, cruxlines};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn analyze_paths_produces_rows() {
        let files = vec![
            (
                PathBuf::from("src/languages/python/fixtures/main.py"),
                std::fs::read_to_string("src/languages/python/fixtures/main.py").expect("read"),
            ),
            (
                PathBuf::from("src/languages/python/fixtures/utils.py"),
                std::fs::read_to_string("src/languages/python/fixtures/utils.py").expect("read"),
            ),
            (
                PathBuf::from("src/languages/python/fixtures/models.py"),
                std::fs::read_to_string("src/languages/python/fixtures/models.py").expect("read"),
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

    #[test]
    fn frecency_runs_in_background_thread() {
        let inputs = vec![(
            PathBuf::from("a.py"),
            "def foo():\n    pass\n".to_string(),
        )];
        let main_thread = std::thread::current().id();
        let recorded = std::sync::Arc::new(std::sync::Mutex::new(None));
        let recorded_handle = std::sync::Arc::clone(&recorded);

        let (_edges, _frecency) = compute_edges_and_frecency(inputs, move |_paths| {
            *recorded_handle.lock().unwrap() = Some(std::thread::current().id());
            HashMap::new()
        });

        let thread_id = recorded.lock().unwrap().expect("thread id recorded");
        assert_ne!(thread_id, main_thread);
    }
}
