use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use directories::ProjectDirs;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::find_references::{Location, SerializedLocation};
use crate::languages::Ecosystem;

// Bump version when cache format changes
const CACHE_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct CachedFile {
    version: u32,
    mtime_secs: u64,
    mtime_nanos: u32,
    size: u64,
    ecosystem: Ecosystem,
    definitions: Vec<SerializedLocation>,
    references: Vec<SerializedLocation>,
    definition_lines: Vec<(SerializedLocation, String)>,
}

pub struct FileCache {
    cache_dir: PathBuf,
}

pub struct CachedFileResult {
    pub ecosystem: Ecosystem,
    pub definitions: Vec<Location>,
    pub references: Vec<Location>,
    pub definition_lines: FxHashMap<Location, String>,
}

impl FileCache {
    pub fn new(repo_root: &Path) -> Self {
        // Get platform-appropriate cache directory:
        // - Linux: ~/.cache/cruxlines/<repo-hash>/
        // - macOS: ~/Library/Caches/cruxlines/<repo-hash>/
        // - Windows: C:\Users\<user>\AppData\Local\cruxlines\cache\<repo-hash>\
        let cache_base = ProjectDirs::from("", "", "cruxlines")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".cruxlines-cache"));

        let repo_hash = hash_path(repo_root);
        let cache_dir = cache_base.join(format!("{:016x}", repo_hash));
        Self { cache_dir }
    }

    /// Try to load cached data for a file. Returns None if cache miss or invalid.
    pub fn get(&self, path: &Path) -> Option<CachedFileResult> {
        let cache_path = self.cache_path(path);
        let bytes = fs::read(&cache_path).ok()?;
        let cached: CachedFile = bincode::deserialize(&bytes).ok()?;

        // Check version
        if cached.version != CACHE_VERSION {
            return None;
        }

        // Check mtime and size
        let metadata = fs::metadata(path).ok()?;
        let mtime = metadata.modified().ok()?;
        let size = metadata.len();

        let (mtime_secs, mtime_nanos) = system_time_to_parts(mtime);
        if cached.mtime_secs != mtime_secs
            || cached.mtime_nanos != mtime_nanos
            || cached.size != size
        {
            return None;
        }

        // Convert SerializedLocation back to Location
        let definitions: Vec<Location> =
            cached.definitions.into_iter().map(Location::from).collect();
        let references: Vec<Location> = cached.references.into_iter().map(Location::from).collect();
        let definition_lines: FxHashMap<Location, String> = cached
            .definition_lines
            .into_iter()
            .map(|(loc, line)| (Location::from(loc), line))
            .collect();

        Some(CachedFileResult {
            ecosystem: cached.ecosystem,
            definitions,
            references,
            definition_lines,
        })
    }

    /// Store cached data for a file.
    pub fn set(
        &self,
        path: &Path,
        ecosystem: Ecosystem,
        definitions: &[Location],
        references: &[Location],
        definition_lines: &FxHashMap<Location, String>,
    ) -> io::Result<()> {
        // Get current mtime and size
        let metadata = fs::metadata(path)?;
        let mtime = metadata.modified()?;
        let size = metadata.len();
        let (mtime_secs, mtime_nanos) = system_time_to_parts(mtime);

        // Convert Location to SerializedLocation for storage
        let definitions_ser: Vec<SerializedLocation> =
            definitions.iter().map(SerializedLocation::from).collect();
        let references_ser: Vec<SerializedLocation> =
            references.iter().map(SerializedLocation::from).collect();
        let definition_lines_ser: Vec<(SerializedLocation, String)> = definition_lines
            .iter()
            .map(|(k, v)| (SerializedLocation::from(k), v.clone()))
            .collect();

        let cached = CachedFile {
            version: CACHE_VERSION,
            mtime_secs,
            mtime_nanos,
            size,
            ecosystem,
            definitions: definitions_ser,
            references: references_ser,
            definition_lines: definition_lines_ser,
        };

        let bytes = bincode::serialize(&cached).map_err(io::Error::other)?;

        // Ensure cache directory exists
        fs::create_dir_all(&self.cache_dir)?;

        let cache_path = self.cache_path(path);
        fs::write(cache_path, bytes)?;

        Ok(())
    }

    fn cache_path(&self, source_path: &Path) -> PathBuf {
        let hash = hash_path(source_path);
        self.cache_dir.join(format!("{:016x}.bin", hash))
    }
}

fn hash_path(path: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    path.hash(&mut hasher);
    hasher.finish()
}

fn system_time_to_parts(time: SystemTime) -> (u64, u32) {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => (duration.as_secs(), duration.subsec_nanos()),
        Err(_) => (0, 0),
    }
}
