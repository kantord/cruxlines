use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

#[derive(Debug)]
pub enum CliIoError {
    CurrentDir(std::io::Error),
    ReadFile { path: PathBuf, source: std::io::Error },
}

pub fn gather_inputs<I>(paths: I) -> Result<Vec<(PathBuf, String)>, CliIoError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let cwd = std::env::current_dir().map_err(CliIoError::CurrentDir)?;
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
        if !is_supported_path(path) {
            continue;
        }
        let bytes = std::fs::read(path)
            .map_err(|source| CliIoError::ReadFile {
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

fn is_supported_path(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("py") | Some("js") | Some("rs") => true,
        _ => false,
    }
}
