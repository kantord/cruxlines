use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lasso::Spur;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::cache::FileCache;
use crate::find_references::{
    Location, ReferenceEdge, ReferenceScan, find_references, find_references_cached,
};
use crate::graph::build_file_graph;
use crate::intern::intern;
use crate::io::{CruxlinesError, gather_paths};
use crate::languages::Ecosystem;

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub rank: f64,
    pub local_score: f64,
    pub file_rank: f64,
    pub definition: Location,
    /// Definition line text from the input snapshot.
    pub definition_line: String,
    /// Heuristic reference locations; may include false positives.
    pub references: Vec<Location>,
}

pub fn cruxlines(
    repo_root: &PathBuf,
    ecosystems: &std::collections::HashSet<Ecosystem>,
) -> Result<Vec<OutputRow>, CruxlinesError> {
    let paths = gather_paths(repo_root, ecosystems);
    cruxlines_from_paths(paths, Some(repo_root.clone()))
}

#[doc(hidden)]
pub fn cruxlines_from_inputs(
    inputs: Vec<(PathBuf, String)>,
    repo_root: Option<PathBuf>,
) -> Vec<OutputRow> {
    let inputs = inputs.into_iter().map(Ok);
    let (scan, frecency) = compute_edges_and_frecency(inputs, repo_root).unwrap_or_else(|_| {
        (
            ReferenceScan {
                edges: Vec::new(),
                definition_lines: HashMap::new(),
            },
            HashMap::new(),
        )
    });

    let grouped_by_ecosystem = group_edges_by_ecosystem(scan.edges);
    let capacity: usize = grouped_by_ecosystem
        .values()
        .map(|grouped| grouped.len())
        .sum();
    let mut output_rows = Vec::with_capacity(capacity);
    for grouped in grouped_by_ecosystem.into_values() {
        let file_ranks = rank_files(&grouped);

        let mut name_counts: FxHashMap<Spur, usize> = FxHashMap::default();
        for definition in grouped.keys() {
            *name_counts.entry(definition.name).or_default() += 1;
        }

        output_rows.extend(build_rows(
            grouped,
            &file_ranks,
            &frecency,
            &name_counts,
            &scan.definition_lines,
        ));
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                let key_a = (
                    a.definition.path,
                    a.definition.line,
                    a.definition.column,
                    a.definition.name,
                );
                let key_b = (
                    b.definition.path,
                    b.definition.line,
                    b.definition.column,
                    b.definition.name,
                );
                key_a.cmp(&key_b)
            })
    });
    output_rows
}

pub fn cruxlines_from_paths(
    paths: Vec<PathBuf>,
    repo_root: Option<PathBuf>,
) -> Result<Vec<OutputRow>, CruxlinesError> {
    let (scan, frecency) = if let Some(ref root) = repo_root {
        compute_edges_and_frecency_cached(paths, root)?
    } else {
        let inputs = paths.into_iter().filter_map(read_input);
        compute_edges_and_frecency(inputs, repo_root)?
    };

    let grouped_by_ecosystem = group_edges_by_ecosystem(scan.edges);
    let capacity: usize = grouped_by_ecosystem
        .values()
        .map(|grouped| grouped.len())
        .sum();

    let mut output_rows = Vec::with_capacity(capacity);
    for (_ecosystem, grouped) in grouped_by_ecosystem {
        let file_ranks = rank_files(&grouped);

        let mut name_counts: FxHashMap<Spur, usize> = FxHashMap::default();
        for definition in grouped.keys() {
            *name_counts.entry(definition.name).or_default() += 1;
        }

        let rows = build_rows(
            grouped,
            &file_ranks,
            &frecency,
            &name_counts,
            &scan.definition_lines,
        );
        output_rows.extend(rows);
    }

    output_rows.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                let key_a = (
                    a.definition.path,
                    a.definition.line,
                    a.definition.column,
                    a.definition.name,
                );
                let key_b = (
                    b.definition.path,
                    b.definition.line,
                    b.definition.column,
                    b.definition.name,
                );
                key_a.cmp(&key_b)
            })
    });

    Ok(output_rows)
}

fn rank_files(grouped: &HashMap<Location, Vec<Location>>) -> FxHashMap<Spur, f64> {
    let (graph, indices) = build_file_graph(grouped);

    if graph.node_count() == 0 {
        return FxHashMap::default();
    }

    let ranks = petgraph::algo::page_rank::parallel_page_rank(&graph, 0.85_f64, 5, None);

    let mut out = FxHashMap::default();
    for (path, idx) in indices {
        out.insert(path, ranks[idx.index()]);
    }
    out
}

fn compute_edges_and_frecency(
    inputs: impl IntoIterator<Item = Result<(PathBuf, String), CruxlinesError>>,
    repo_root: Option<PathBuf>,
) -> Result<(ReferenceScan, HashMap<Spur, f64>), CruxlinesError> {
    let frecency_handle = std::thread::spawn(move || frecency_scores(repo_root.as_deref()));

    let scan = find_references(inputs)?;
    let frecency = frecency_handle.join().unwrap_or_default();

    Ok((scan, frecency))
}

fn compute_edges_and_frecency_cached(
    paths: Vec<PathBuf>,
    repo_root: &Path,
) -> Result<(ReferenceScan, HashMap<Spur, f64>), CruxlinesError> {
    let cache = FileCache::new(repo_root);

    let repo_root_clone = repo_root.to_path_buf();
    let frecency_handle =
        std::thread::spawn(move || frecency_scores(Some(repo_root_clone.as_path())));

    let scan = find_references_cached(paths, &cache)?;
    let frecency = frecency_handle.join().unwrap_or_default();

    Ok((scan, frecency))
}

fn build_rows(
    grouped: HashMap<Location, Vec<Location>>,
    file_ranks: &FxHashMap<Spur, f64>,
    frecency: &HashMap<Spur, f64>,
    name_counts: &FxHashMap<Spur, usize>,
    definition_lines: &HashMap<Location, String>,
) -> Vec<OutputRow> {
    grouped
        .into_par_iter()
        .map(|(definition, mut references)| {
            references.sort_by(|a, b| {
                let key_a = (a.path, a.line, a.column, a.name);
                let key_b = (b.path, b.line, b.column, b.name);
                key_a.cmp(&key_b)
            });
            let name_count = name_counts.get(&definition.name).copied().unwrap_or(1) as f64;
            let weighted_refs: f64 = references
                .iter()
                .map(|reference| {
                    let file_rank = file_ranks.get(&reference.path).copied().unwrap_or(0.0);
                    let frecency = frecency.get(&reference.path).copied().unwrap_or(1.0);
                    file_rank * frecency
                })
                .sum();
            let local_score = weighted_refs / name_count;
            let file_rank = file_ranks.get(&definition.path).copied().unwrap_or(0.0);
            let rank = local_score * file_rank;
            let definition_line = definition_lines
                .get(&definition)
                .cloned()
                .unwrap_or_default();
            OutputRow {
                rank,
                local_score,
                file_rank,
                definition,
                definition_line,
                references,
            }
        })
        .collect()
}

fn group_edges_by_ecosystem(
    edges: Vec<ReferenceEdge>,
) -> HashMap<Ecosystem, HashMap<Location, Vec<Location>>> {
    let mut grouped_by_ecosystem: HashMap<Ecosystem, HashMap<Location, Vec<Location>>> =
        HashMap::new();
    for edge in edges {
        let ecosystem = edge.ecosystem;
        grouped_by_ecosystem
            .entry(ecosystem)
            .or_default()
            .entry(edge.definition)
            .or_default()
            .push(edge.usage);
    }
    grouped_by_ecosystem
}

fn frecency_scores(repo_root: Option<&std::path::Path>) -> HashMap<Spur, f64> {
    let Some(repo_root) = repo_root else {
        return HashMap::new();
    };
    if !repo_root.join(".git").is_dir() {
        return HashMap::new();
    }
    let Ok(scores) = frecenfile::analyze_repo(repo_root, None, None) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (path, score) in scores {
        let full_path = repo_root.join(path);
        out.insert(intern(&full_path.to_string_lossy()), score);
    }
    out
}

fn read_input(path: PathBuf) -> Option<Result<(PathBuf, String), CruxlinesError>> {
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(source) => {
            return Some(Err(CruxlinesError::ReadFile { path, source }));
        }
    };
    let contents = match String::from_utf8(bytes) {
        Ok(contents) => contents,
        Err(_) => return None,
    };
    Some(Ok((path, contents)))
}

#[cfg(test)]
mod tests {
    use super::{cruxlines_from_inputs, group_edges_by_ecosystem};
    use crate::find_references::{Location, ReferenceEdge};
    use crate::intern::intern;
    use crate::languages::Ecosystem;
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
        let rows = cruxlines_from_inputs(files, None);
        assert!(!rows.is_empty());
        assert!(rows.iter().any(|row| row.definition.name_str() == "add"));
    }

    #[test]
    fn scores_are_normalized_by_definition_count() {
        let inputs = vec![
            (
                PathBuf::from("a.py"),
                "def foo():\n    pass\n\ndef foo():\n    pass\n\ndef bar():\n    pass\n"
                    .to_string(),
            ),
            (
                PathBuf::from("c.py"),
                "from a import foo, bar\n\nfoo()\nbar()\n".to_string(),
            ),
        ];
        let rows = cruxlines_from_inputs(inputs, None);
        let foo_scores: Vec<f64> = rows
            .iter()
            .filter(|row| row.definition.name_str() == "foo")
            .map(|row| row.rank)
            .collect();
        let bar_score = rows
            .iter()
            .find(|row| row.definition.name_str() == "bar")
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
            (PathBuf::from("a.py"), "def foo():\n    pass\n".to_string()),
            (PathBuf::from("b.py"), "def foo():\n    pass\n".to_string()),
            (
                PathBuf::from("c.py"),
                "from a import foo\nfrom b import foo\n\nfoo()\n".to_string(),
            ),
        ];
        let rows = cruxlines_from_inputs(inputs, None);
        let a_score = rows
            .iter()
            .find(|row| row.definition.path_str().ends_with("a.py"))
            .map(|row| row.rank)
            .unwrap_or(0.0);
        let b_score = rows
            .iter()
            .find(|row| row.definition.path_str().ends_with("b.py"))
            .map(|row| row.rank)
            .unwrap_or(0.0);
        assert!(a_score > 0.0);
        assert!(b_score > 0.0);
    }

    #[test]
    fn groups_edges_without_extension_by_ecosystem() {
        let edge = ReferenceEdge {
            definition: Location {
                path: intern("defs/alpha"),
                line: 1,
                column: 1,
                name: intern("alpha"),
            },
            usage: Location {
                path: intern("use"),
                line: 2,
                column: 1,
                name: intern("alpha"),
            },
            ecosystem: Ecosystem::Python,
        };

        let grouped = group_edges_by_ecosystem(vec![edge]);
        let count: usize = grouped.values().map(|map| map.len()).sum();
        assert_eq!(count, 1, "expected edge to be grouped by ecosystem");
    }
}
