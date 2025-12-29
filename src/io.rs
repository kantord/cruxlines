use std::collections::HashSet;
use std::path::PathBuf;

use ignore::WalkBuilder;

use crate::Ecosystem;

#[derive(Debug)]
pub enum CruxlinesError {
    ReadFile { path: PathBuf, source: std::io::Error },
}

pub fn gather_paths(
    repo_root: &PathBuf,
    ecosystems: &HashSet<Ecosystem>,
) -> Vec<PathBuf> {
    let builder = WalkBuilder::new(repo_root);

    let mut paths = Vec::new();
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
        let Some(ecosystem) = crate::ecosystem_for_path(path) else {
            continue;
        };
        if !ecosystems.contains(&ecosystem) {
            continue;
        }
        paths.push(path.to_path_buf());
    }

    paths
}
