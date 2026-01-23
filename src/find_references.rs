use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lasso::Spur;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree};

use crate::cache::FileCache;
use crate::intern::{intern, resolve};

/// A source code location with interned path and name for efficiency.
/// Use `path_str()` and `name_str()` to get string values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub path: Spur,
    pub line: usize,
    pub column: usize,
    pub name: Spur,
}

impl Location {
    /// Get the path as a string slice
    #[inline]
    pub fn path_str(&self) -> &'static str {
        resolve(self.path)
    }

    /// Get the name as a string slice
    #[inline]
    pub fn name_str(&self) -> &'static str {
        resolve(self.name)
    }

    /// Get the path as a PathBuf (for compatibility)
    #[inline]
    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(self.path_str())
    }
}

/// Serializable version of Location for cache storage
#[derive(Serialize, Deserialize)]
pub struct SerializedLocation {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub name: String,
}

impl From<&Location> for SerializedLocation {
    fn from(loc: &Location) -> Self {
        Self {
            path: loc.path_str().to_string(),
            line: loc.line,
            column: loc.column,
            name: loc.name_str().to_string(),
        }
    }
}

impl From<SerializedLocation> for Location {
    fn from(loc: SerializedLocation) -> Self {
        Self {
            path: intern(&loc.path),
            line: loc.line,
            column: loc.column,
            name: intern(&loc.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceEdge {
    pub definition: Location,
    pub usage: Location,
    pub ecosystem: crate::languages::Ecosystem,
}

pub struct ReferenceScan {
    pub edges: Vec<ReferenceEdge>,
    pub definition_lines: HashMap<Location, String>,
}

struct EcosystemSymbols {
    definitions: FxHashMap<Spur, Vec<Location>>,
    definition_positions: FxHashSet<(Spur, usize, usize)>,
    references: Vec<Location>,
    definition_lines: FxHashMap<Location, String>,
}

/// Results from processing a single file
struct FileResult {
    ecosystem: crate::languages::Ecosystem,
    definitions: Vec<Location>,
    references: Vec<Location>,
    definition_lines: FxHashMap<Location, String>,
}

pub fn find_references<I, P>(files: I) -> Result<ReferenceScan, crate::io::CruxlinesError>
where
    I: IntoIterator<Item = Result<(P, String), crate::io::CruxlinesError>>,
    P: Into<PathBuf>,
{
    // Collect files first (need Vec for parallel iteration)
    let files: Vec<(PathBuf, String)> = files
        .into_iter()
        .filter_map(|item| item.ok())
        .map(|(p, s)| (p.into(), s))
        .collect();

    // Process files in parallel
    let file_results: Vec<FileResult> = files
        .par_iter()
        .filter_map(|(path, source)| process_file(path, source))
        .collect();

    // Merge results by ecosystem
    let mut symbols_by_ecosystem: HashMap<crate::languages::Ecosystem, EcosystemSymbols> =
        HashMap::new();

    for result in file_results {
        let entry = symbols_by_ecosystem
            .entry(result.ecosystem)
            .or_insert_with(|| EcosystemSymbols {
                definitions: FxHashMap::default(),
                definition_positions: FxHashSet::default(),
                references: Vec::new(),
                definition_lines: FxHashMap::default(),
            });

        for location in result.definitions {
            record_definition(
                location,
                &mut entry.definitions,
                &mut entry.definition_positions,
            );
        }
        entry.references.extend(result.references);
        entry.definition_lines.extend(result.definition_lines);
    }

    let mut edges = Vec::new();
    let mut definition_lines = HashMap::new();
    for (ecosystem, symbols) in &symbols_by_ecosystem {
        let ecosystem_edges: Vec<ReferenceEdge> = symbols
            .references
            .par_iter()
            .flat_map(|reference| {
                make_edges(
                    reference,
                    *ecosystem,
                    &symbols.definitions,
                    &symbols.definition_positions,
                )
            })
            .collect();
        edges.extend(ecosystem_edges);

        for (location, line) in &symbols.definition_lines {
            definition_lines
                .entry(*location)
                .or_insert_with(|| line.clone());
        }
    }

    Ok(ReferenceScan {
        edges,
        definition_lines,
    })
}

/// Find references with caching support. Only reads and parses files that aren't cached.
pub fn find_references_cached(
    paths: Vec<PathBuf>,
    cache: &FileCache,
) -> Result<ReferenceScan, crate::io::CruxlinesError> {
    // Process files in parallel - check cache first, parse on miss
    let file_results: Vec<FileResult> = paths
        .par_iter()
        .filter_map(|path| process_file_cached(path, cache))
        .collect();

    // Rest is same as find_references
    let mut symbols_by_ecosystem: HashMap<crate::languages::Ecosystem, EcosystemSymbols> =
        HashMap::new();

    for result in file_results {
        let entry = symbols_by_ecosystem
            .entry(result.ecosystem)
            .or_insert_with(|| EcosystemSymbols {
                definitions: FxHashMap::default(),
                definition_positions: FxHashSet::default(),
                references: Vec::new(),
                definition_lines: FxHashMap::default(),
            });

        for location in result.definitions {
            record_definition(
                location,
                &mut entry.definitions,
                &mut entry.definition_positions,
            );
        }
        entry.references.extend(result.references);
        entry.definition_lines.extend(result.definition_lines);
    }

    let mut edges = Vec::new();
    let mut definition_lines = HashMap::new();
    for (ecosystem, symbols) in &symbols_by_ecosystem {
        let ecosystem_edges: Vec<ReferenceEdge> = symbols
            .references
            .par_iter()
            .flat_map(|reference| {
                make_edges(
                    reference,
                    *ecosystem,
                    &symbols.definitions,
                    &symbols.definition_positions,
                )
            })
            .collect();
        edges.extend(ecosystem_edges);

        for (location, line) in &symbols.definition_lines {
            definition_lines
                .entry(*location)
                .or_insert_with(|| line.clone());
        }
    }

    Ok(ReferenceScan {
        edges,
        definition_lines,
    })
}

/// Process a file with cache support - returns cached result or parses fresh
fn process_file_cached(path: &Path, cache: &FileCache) -> Option<FileResult> {
    // Try cache first
    if let Some(cached) = cache.get(path) {
        return Some(FileResult {
            ecosystem: cached.ecosystem,
            definitions: cached.definitions,
            references: cached.references,
            definition_lines: cached.definition_lines,
        });
    }

    // Cache miss - read and parse file
    let source = std::fs::read_to_string(path).ok()?;
    let result = process_file(path, &source)?;

    // Save to cache (ignore errors)
    let _ = cache.set(
        path,
        result.ecosystem,
        &result.definitions,
        &result.references,
        &result.definition_lines,
    );

    Some(result)
}

fn collect_definitions(
    path: &Path,
    source: &str,
    tree: &Tree,
    language: crate::languages::Language,
) -> (Vec<Location>, FxHashMap<Location, String>) {
    let mut definitions = Vec::new();
    let mut definition_lines = FxHashMap::default();

    let emit_def =
        |loc: Location, defs: &mut Vec<Location>, lines: &mut FxHashMap<Location, String>| {
            record_definition_line(&loc, source, lines);
            defs.push(loc);
        };

    match language {
        crate::languages::Language::C => {
            crate::languages::c::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Cpp => {
            crate::languages::cpp::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::CSharp => {
            crate::languages::csharp::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Go => {
            crate::languages::go::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Java => {
            crate::languages::java::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Kotlin => {
            crate::languages::kotlin::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Php => {
            crate::languages::php::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Python => {
            crate::languages::python::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::JavaScript
        | crate::languages::Language::TypeScript
        | crate::languages::Language::TypeScriptReact => {
            crate::languages::javascript::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
        crate::languages::Language::Rust => {
            crate::languages::rust::emit_definitions(path, source, tree, |loc| {
                emit_def(loc, &mut definitions, &mut definition_lines);
            });
        }
    }

    (definitions, definition_lines)
}

/// Process a single file: parse and extract definitions/references
fn process_file(path: &Path, source: &str) -> Option<FileResult> {
    let language = crate::languages::language_for_path(path)?;
    let tree = parse_tree(&language, source)?;
    let ecosystem = crate::languages::ecosystem_for_language(language);

    let (definitions, definition_lines) = collect_definitions(path, source, &tree, language);

    let mut references = Vec::new();
    match language {
        crate::languages::Language::C => {
            crate::languages::c::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Cpp => {
            crate::languages::cpp::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::CSharp => {
            crate::languages::csharp::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Go => {
            crate::languages::go::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Java => {
            crate::languages::java::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Kotlin => {
            crate::languages::kotlin::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Php => {
            crate::languages::php::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Python => {
            crate::languages::python::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::JavaScript
        | crate::languages::Language::TypeScript
        | crate::languages::Language::TypeScriptReact => {
            crate::languages::javascript::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
        crate::languages::Language::Rust => {
            crate::languages::rust::emit_references(path, source, &tree, |loc| {
                references.push(loc);
            });
        }
    }

    Some(FileResult {
        ecosystem,
        definitions,
        references,
        definition_lines,
    })
}

fn parse_tree(language: &crate::languages::Language, source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let ts_language = crate::languages::tree_sitter_language(*language);
    parser.set_language(&ts_language).ok()?;
    parser.parse(source, None)
}

pub(crate) fn walk_tree(tree: &Tree, mut visit: impl FnMut(Node)) {
    let root = tree.root_node();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        visit(node);
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
}

pub(crate) fn collect_identifier_nodes<F>(node: Node, source: &str, mut on_ident: F)
where
    F: FnMut(Node),
{
    if node.kind() == "identifier" && node.utf8_text(source.as_bytes()).is_ok() {
        on_ident(node);
    }
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        for i in 0..current.child_count() {
            if let Some(child) = current.child(i) {
                if child.kind() == "identifier" {
                    if child.utf8_text(source.as_bytes()).is_ok() {
                        on_ident(child);
                    }
                } else {
                    stack.push(child);
                }
            }
        }
    }
}

pub(crate) fn location_from_node(path: &Path, source: &str, node: Node) -> Option<Location> {
    let (line, column) = position(node);
    let name = node.utf8_text(source.as_bytes()).ok()?;
    Some(Location {
        path: intern(&path.to_string_lossy()),
        line,
        column,
        name: intern(name),
    })
}

fn record_definition(
    location: Location,
    definitions: &mut FxHashMap<Spur, Vec<Location>>,
    definition_positions: &mut FxHashSet<(Spur, usize, usize)>,
) {
    let key = location.name;
    let entry = definitions.entry(key).or_default();
    if !entry.iter().any(|item| {
        item.path == location.path && item.line == location.line && item.column == location.column
    }) {
        entry.push(location);
    }
    definition_positions.insert((location.path, location.line, location.column));
}

/// Returns edges for a reference (used in parallel processing)
fn make_edges(
    location: &Location,
    ecosystem: crate::languages::Ecosystem,
    definitions: &FxHashMap<Spur, Vec<Location>>,
    definition_positions: &FxHashSet<(Spur, usize, usize)>,
) -> Vec<ReferenceEdge> {
    if definition_positions.contains(&(location.path, location.line, location.column)) {
        return Vec::new();
    }
    if let Some(defs) = definitions.get(&location.name) {
        defs.iter()
            .map(|def| ReferenceEdge {
                definition: *def,
                usage: *location,
                ecosystem,
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn position(node: Node) -> (usize, usize) {
    let pos = node.start_position();
    (pos.row + 1, pos.column + 1)
}

fn record_definition_line(
    location: &Location,
    source: &str,
    lines: &mut FxHashMap<Location, String>,
) {
    if lines.contains_key(location) {
        return;
    }
    let text = source
        .lines()
        .nth(location.line.saturating_sub(1))
        .unwrap_or("")
        .trim_end()
        .to_string();
    lines.insert(*location, text);
}

#[cfg(test)]
mod tests {
    use super::walk_tree;
    use tree_sitter::Parser;

    #[test]
    fn walk_tree_visits_nodes() {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into()).expect("set language");
        let tree = parser.parse("x = 1\n", None).expect("parse");

        let mut kinds = Vec::new();
        walk_tree(&tree, |node| {
            kinds.push(node.kind().to_string());
        });

        assert!(kinds.contains(&"module".to_string()));
        assert!(kinds.contains(&"identifier".to_string()));
    }
}
