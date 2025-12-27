use std::collections::HashSet;
use std::path::PathBuf;

use ignore::WalkBuilder;

use cruxlines::Ecosystem;

#[derive(Debug)]
pub enum CliIoError {
    CurrentDir(std::io::Error),
    ReadFile { path: PathBuf, source: std::io::Error },
}

pub fn gather_inputs<I>(
    paths: I,
    ecosystems: &HashSet<Ecosystem>,
) -> Result<Vec<(PathBuf, String)>, CliIoError>
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

    let mut inputs = Vec::new();
    for file in requested_files {
        if !file.is_file() {
            continue;
        }
        let Some(ecosystem) = cruxlines::ecosystem_for_path(&file) else {
            continue;
        };
        if !ecosystems.contains(&ecosystem) {
            continue;
        }
        let bytes = std::fs::read(&file)
            .map_err(|source| CliIoError::ReadFile {
                path: file.clone(),
                source,
            })?;
        let contents = match String::from_utf8(bytes) {
            Ok(contents) => contents,
            Err(_) => {
                continue;
            }
        };
        inputs.push((file, contents));
    }

    if !requested_dirs.is_empty() {
        let mut roots: Vec<PathBuf> = requested_dirs.into_iter().collect();
        roots.sort();
        roots.dedup();

        let mut builder = WalkBuilder::new(&roots[0]);
        for path in &roots[1..] {
            builder.add(path);
        }
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
            let Some(ecosystem) = cruxlines::ecosystem_for_path(path) else {
                continue;
            };
            if !ecosystems.contains(&ecosystem) {
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
    }

    Ok(inputs)
}

 
