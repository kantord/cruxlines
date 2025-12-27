use std::collections::HashSet;
use std::path::PathBuf;

use ignore::WalkBuilder;

use cruxlines::Ecosystem;

#[derive(Debug)]
pub enum CliIoError {
    ReadFile { path: PathBuf, source: std::io::Error },
}

pub fn gather_inputs(
    repo_root: &PathBuf,
    ecosystems: &HashSet<Ecosystem>,
) -> Result<Vec<(PathBuf, String)>, CliIoError> {
    let builder = WalkBuilder::new(repo_root);

    let mut inputs = Vec::new();
    for entry in builder.build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.path().starts_with(repo_root) {
            continue;
        }
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

    Ok(inputs)
}

 
