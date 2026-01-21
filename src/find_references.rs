use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use tree_sitter::{Node, Parser, Tree};

use crate::timing;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub name: String,
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
    definitions: HashMap<String, Vec<Location>>,
    definition_positions: HashSet<(PathBuf, usize, usize)>,
    references: Vec<Location>,
    definition_lines: HashMap<Location, String>,
}

pub fn find_references<I, P>(files: I) -> Result<ReferenceScan, crate::io::CruxlinesError>
where
    I: IntoIterator<Item = Result<(P, String), crate::io::CruxlinesError>>,
    P: Into<PathBuf>,
{
    let mut symbols_by_ecosystem: HashMap<crate::languages::Ecosystem, EcosystemSymbols> =
        HashMap::new();

    let mut total_parse_time = std::time::Duration::ZERO;
    let mut total_definitions_time = std::time::Duration::ZERO;
    let mut total_references_time = std::time::Duration::ZERO;
    let mut total_definition_lines_time = std::time::Duration::ZERO;
    let mut file_count = 0usize;
    let mut total_definitions = 0usize;
    let mut total_references = 0usize;

    for item in files {
        let (path, source) = item?;
        let path = path.into();
        let Some(language) = crate::languages::language_for_path(&path) else {
            continue;
        };

        let parse_start = Instant::now();
        let Some(tree) = parse_tree(&language, &source) else {
            continue;
        };
        total_parse_time += parse_start.elapsed();
        file_count += 1;

        let ecosystem = crate::languages::ecosystem_for_language(language);
        let entry = symbols_by_ecosystem.entry(ecosystem).or_insert_with(|| EcosystemSymbols {
            definitions: HashMap::new(),
            definition_positions: HashSet::new(),
            references: Vec::new(),
            definition_lines: HashMap::new(),
        });

        let defs_before = entry.definitions.values().map(|v| v.len()).sum::<usize>();

        let def_start = Instant::now();
        match language {
            crate::languages::Language::Java => crate::languages::java::emit_definitions(
                &path,
                &source,
                &tree,
                |location| {
                    record_definition(
                        location.clone(),
                        &mut entry.definitions,
                        &mut entry.definition_positions,
                    );
                    let line_start = Instant::now();
                    record_definition_line(&location, &source, &mut entry.definition_lines);
                    total_definition_lines_time += line_start.elapsed();
                },
            ),
            crate::languages::Language::Kotlin => crate::languages::kotlin::emit_definitions(
                &path,
                &source,
                &tree,
                |location| {
                    record_definition(
                        location.clone(),
                        &mut entry.definitions,
                        &mut entry.definition_positions,
                    );
                    let line_start = Instant::now();
                    record_definition_line(&location, &source, &mut entry.definition_lines);
                    total_definition_lines_time += line_start.elapsed();
                },
            ),
            crate::languages::Language::Python => crate::languages::python::emit_definitions(
                &path,
                &source,
                &tree,
                |location| {
                    record_definition(
                        location.clone(),
                        &mut entry.definitions,
                        &mut entry.definition_positions,
                    );
                    let line_start = Instant::now();
                    record_definition_line(&location, &source, &mut entry.definition_lines);
                    total_definition_lines_time += line_start.elapsed();
                },
            ),
            crate::languages::Language::JavaScript
            | crate::languages::Language::TypeScript
            | crate::languages::Language::TypeScriptReact => {
                crate::languages::javascript::emit_definitions(
                    &path,
                    &source,
                    &tree,
                    |location| {
                        record_definition(
                            location.clone(),
                            &mut entry.definitions,
                            &mut entry.definition_positions,
                        );
                        let line_start = Instant::now();
                        record_definition_line(&location, &source, &mut entry.definition_lines);
                        total_definition_lines_time += line_start.elapsed();
                    },
                )
            }
            crate::languages::Language::Rust => crate::languages::rust::emit_definitions(
                &path,
                &source,
                &tree,
                |location| {
                    record_definition(
                        location.clone(),
                        &mut entry.definitions,
                        &mut entry.definition_positions,
                    );
                    let line_start = Instant::now();
                    record_definition_line(&location, &source, &mut entry.definition_lines);
                    total_definition_lines_time += line_start.elapsed();
                },
            ),
        }
        total_definitions_time += def_start.elapsed();
        total_definitions += entry.definitions.values().map(|v| v.len()).sum::<usize>() - defs_before;

        let refs_before = entry.references.len();
        let ref_start = Instant::now();
        match language {
            crate::languages::Language::Java => crate::languages::java::emit_references(
                &path,
                &source,
                &tree,
                |location| entry.references.push(location),
            ),
            crate::languages::Language::Kotlin => crate::languages::kotlin::emit_references(
                &path,
                &source,
                &tree,
                |location| entry.references.push(location),
            ),
            crate::languages::Language::Python => crate::languages::python::emit_references(
                &path,
                &source,
                &tree,
                |location| entry.references.push(location),
            ),
            crate::languages::Language::JavaScript
            | crate::languages::Language::TypeScript
            | crate::languages::Language::TypeScriptReact => {
                crate::languages::javascript::emit_references(
                    &path,
                    &source,
                    &tree,
                    |location| entry.references.push(location),
                )
            }
            crate::languages::Language::Rust => crate::languages::rust::emit_references(
                &path,
                &source,
                &tree,
                |location| entry.references.push(location),
            ),
        }
        total_references_time += ref_start.elapsed();
        total_references += entry.references.len() - refs_before;
    }

    timing::log_with_count("  parse_tree (total)", total_parse_time, file_count);
    timing::log_with_count("  emit_definitions (total)", total_definitions_time, total_definitions);
    timing::log_with_count("    record_definition_line (total)", total_definition_lines_time, total_definitions);
    timing::log_with_count("  emit_references (total)", total_references_time, total_references);

    let start = Instant::now();
    let mut edges = Vec::new();
    let mut definition_lines = HashMap::new();
    for (ecosystem, symbols) in &symbols_by_ecosystem {
        for reference in &symbols.references {
            record_reference(
                reference.clone(),
                *ecosystem,
                &symbols.definitions,
                &symbols.definition_positions,
                &mut edges,
            );
        }
        for (location, line) in &symbols.definition_lines {
            definition_lines
                .entry(location.clone())
                .or_insert_with(|| line.clone());
        }
    }
    timing::log_with_count("  build_edges", start.elapsed(), edges.len());

    Ok(ReferenceScan {
        edges,
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
    if node.kind() == "identifier"
        && node.utf8_text(source.as_bytes()).is_ok() {
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
        path: path.to_path_buf(),
        line,
        column,
        name: name.to_string(),
    })
}

fn record_definition(
    location: Location,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    let key = location.name.clone();
    let entry = definitions.entry(key).or_default();
    if !entry.iter().any(|item| {
        item.path == location.path && item.line == location.line && item.column == location.column
    }) {
        entry.push(location.clone());
    }
    definition_positions.insert((location.path, location.line, location.column));
}

fn record_reference(
    location: Location,
    ecosystem: crate::languages::Ecosystem,
    definitions: &HashMap<String, Vec<Location>>,
    definition_positions: &HashSet<(PathBuf, usize, usize)>,
    edges: &mut Vec<ReferenceEdge>,
) {
    if definition_positions.contains(&(location.path.clone(), location.line, location.column)) {
        return;
    }
    if let Some(defs) = definitions.get(&location.name) {
        for def in defs {
            edges.push(ReferenceEdge {
                definition: def.clone(),
                usage: location.clone(),
                ecosystem,
            });
        }
    }
}

fn position(node: Node) -> (usize, usize) {
    let pos = node.start_position();
    (pos.row + 1, pos.column + 1)
}

fn record_definition_line(
    location: &Location,
    source: &str,
    lines: &mut HashMap<Location, String>,
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
    lines.insert(location.clone(), text);
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
