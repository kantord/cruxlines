use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use ignore::WalkBuilder;
use petgraph::algo::page_rank;

use crate::find_references::{find_references, Location, ReferenceEdge};
use crate::graph::build_reference_graphs_by_language;
use crate::scoring::sort_by_rank_desc;

#[derive(Debug, Clone)]
pub struct OutputRow {
    pub rank: f64,
    pub definition: Location,
    pub references: Vec<Location>,
}

#[derive(Debug)]
pub enum AnalyzeError {
    CurrentDir(std::io::Error),
    ReadFile { path: PathBuf, source: std::io::Error },
}

pub fn analyze_paths<I>(paths: I) -> Result<Vec<OutputRow>, AnalyzeError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let inputs = gather_inputs(paths)?;
    let edges: Vec<ReferenceEdge> = find_references(inputs).collect();

    let graphs = build_reference_graphs_by_language(&edges);
    let mut ranks_by_location: HashMap<Location, f64> = HashMap::new();
    for (_language, (graph, indices)) in graphs {
        let ranks = page_rank(&graph, 0.85_f64, 20);
        for (location, idx) in indices {
            ranks_by_location.insert(location, ranks[idx.index()]);
        }
    }

    let mut grouped: HashMap<Location, Vec<Location>> = HashMap::new();
    for edge in edges {
        grouped
            .entry(edge.definition.clone())
            .or_default()
            .push(edge.usage);
    }

    let mut output_rows = Vec::with_capacity(grouped.len());
    for (definition, mut references) in grouped {
        references.sort_by(|a, b| {
            let key_a = (&a.path, a.line, a.column, &a.name);
            let key_b = (&b.path, b.line, b.column, &b.name);
            key_a.cmp(&key_b)
        });
        let rank = ranks_by_location
            .get(&definition)
            .copied()
            .unwrap_or(0.0);
        output_rows.push(OutputRow {
            rank,
            definition,
            references,
        });
    }

    sort_by_rank_desc(&mut output_rows);
    Ok(output_rows)
}

fn gather_inputs<I>(paths: I) -> Result<Vec<(PathBuf, String)>, AnalyzeError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let cwd = std::env::current_dir().map_err(AnalyzeError::CurrentDir)?;
    let mut requested_files: HashSet<PathBuf> = HashSet::new();
    let mut requested_dirs: HashSet<PathBuf> = HashSet::new();
    for path in paths {
        let abs = if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        };
        if abs.is_dir() {
            requested_dirs.insert(abs);
        } else {
            requested_files.insert(abs);
        }
    }

    let mut roots: Vec<PathBuf> = Vec::new();
    for dir in &requested_dirs {
        roots.push(dir.clone());
    }
    for file in &requested_files {
        if let Some(parent) = file.parent() {
            roots.push(parent.to_path_buf());
        }
    }
    if roots.is_empty() {
        return Ok(Vec::new());
    }
    roots.sort();
    roots.dedup();

    let mut builder = WalkBuilder::new(&roots[0]);
    for path in &roots[1..] {
        builder.add(path);
    }
    let requested_files = requested_files;
    let requested_dirs = requested_dirs;
    let cwd_filter = cwd.clone();
    builder.filter_entry(move |entry| {
        let path = entry.path();
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd_filter.join(path)
        };
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            requested_dirs.iter().any(|dir| dir.starts_with(&abs))
                || requested_files.iter().any(|file| file.starts_with(&abs))
        } else {
            requested_files.contains(&abs)
                || requested_dirs.iter().any(|dir| abs.starts_with(dir))
        }
    });

    let mut inputs = Vec::new();
    for entry in builder.build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry
            .file_type()
            .map(|file_type| file_type.is_file())
            .unwrap_or(false)
        {
            continue;
        }
        let path = entry.path();
        if crate::format_router::language_for_path(path).is_none() {
            continue;
        }
        let bytes = std::fs::read(path)
            .map_err(|source| AnalyzeError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;
        let contents = match String::from_utf8(bytes) {
            Ok(contents) => contents,
            Err(_) => {
                continue;
            }
        };
        inputs.push((path.to_path_buf(), contents));
    }

    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use super::analyze_paths;
    use std::path::PathBuf;

    #[test]
    fn analyze_paths_produces_rows() {
        let rows = analyze_paths(vec![
            PathBuf::from("fixtures/python/main.py"),
            PathBuf::from("fixtures/python/utils.py"),
            PathBuf::from("fixtures/python/models.py"),
        ])
        .expect("analysis");
        assert!(!rows.is_empty());
        assert!(rows.iter().any(|row| row.definition.name == "add"));
    }
}
