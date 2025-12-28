use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser, Tree};

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

struct FileInput {
    path: PathBuf,
    source: String,
    language: crate::languages::Language,
    tree: Tree,
}

pub fn find_references<I, P>(files: I) -> impl Iterator<Item = ReferenceEdge>
where
    I: IntoIterator<Item = (P, String)>,
    P: Into<PathBuf>,
{
    let mut inputs_by_ecosystem: HashMap<crate::languages::Ecosystem, Vec<FileInput>> =
        HashMap::new();
    for (path, source) in files {
        let path = path.into();
        let Some(language) = crate::languages::language_for_path(&path) else {
            continue;
        };
        let Some(tree) = parse_tree(&language, &source) else {
            continue;
        };
        let ecosystem = crate::languages::ecosystem_for_language(language);
        inputs_by_ecosystem
            .entry(ecosystem)
            .or_default()
            .push(FileInput {
                path,
                source,
                language,
                tree,
            });
    }

    let mut edges = Vec::new();
    for (ecosystem, inputs) in &inputs_by_ecosystem {
        let mut definitions: HashMap<String, Vec<Location>> = HashMap::new();
        let mut definition_positions: HashSet<(PathBuf, usize, usize)> = HashSet::new();

        for input in inputs {
            collect_definitions(
                &input.path,
                &input.source,
                &input.language,
                &input.tree,
                &mut definitions,
                &mut definition_positions,
            );
        }

        for input in inputs {
            collect_references(
                &input.path,
                &input.source,
                &input.tree,
                &definitions,
                &definition_positions,
                crate::languages::reference_kinds(input.language),
                *ecosystem,
                &mut edges,
            );
        }
    }

    edges.into_iter()
}

fn parse_tree(language: &crate::languages::Language, source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let ts_language = crate::languages::tree_sitter_language(*language);
    parser.set_language(&ts_language).ok()?;
    parser.parse(source, None)
}

fn collect_definitions(
    path: &Path,
    source: &str,
    language: &crate::languages::Language,
    tree: &Tree,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    let root = tree.root_node();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match language {
            crate::languages::Language::Python => crate::languages::python::collect_definition(
                path,
                source,
                node,
                definitions,
                definition_positions,
            ),
            crate::languages::Language::JavaScript
            | crate::languages::Language::TypeScript
            | crate::languages::Language::TypeScriptReact => {
                crate::languages::javascript::collect_definition(
                    path,
                    source,
                    node,
                    definitions,
                    definition_positions,
                )
            }
            crate::languages::Language::Rust => crate::languages::rust::collect_definition(
                path,
                source,
                node,
                definitions,
                definition_positions,
            ),
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
}

fn collect_references(
    path: &Path,
    source: &str,
    tree: &Tree,
    definitions: &HashMap<String, Vec<Location>>,
    definition_positions: &HashSet<(PathBuf, usize, usize)>,
    reference_kinds: &[&str],
    ecosystem: crate::languages::Ecosystem,
    edges: &mut Vec<ReferenceEdge>,
) {
    let root = tree.root_node();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if reference_kinds.contains(&node.kind()) {
            let (line, column) = position(node);
            if !definition_positions.contains(&(path.to_path_buf(), line, column)) {
                if let Ok(name) = node.utf8_text(source.as_bytes()) {
                    if let Some(defs) = definitions.get(name) {
                        let usage = Location {
                            path: path.to_path_buf(),
                            line,
                            column,
                            name: name.to_string(),
                        };
                        for def in defs {
                            edges.push(ReferenceEdge {
                                definition: def.clone(),
                                usage: usage.clone(),
                                ecosystem,
                            });
                        }
                    }
                }
            }
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
}

pub(crate) fn record_definition(
    path: &Path,
    source: &str,
    name_node: Node,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    let (line, column) = position(name_node);
    let Ok(name) = name_node.utf8_text(source.as_bytes()) else {
        return;
    };
    let location = Location {
        path: path.to_path_buf(),
        line,
        column,
        name: name.to_string(),
    };
    let key = location.name.clone();
    let entry = definitions.entry(key).or_default();
    if !entry.iter().any(|item| item.path == location.path && item.line == line && item.column == column) {
        entry.push(location);
    }
    definition_positions.insert((path.to_path_buf(), line, column));
}

pub(crate) fn collect_identifier_nodes<F>(node: Node, source: &str, mut on_ident: F)
where
    F: FnMut(Node),
{
    if node.kind() == "identifier" {
        if node.utf8_text(source.as_bytes()).is_ok() {
            on_ident(node);
        }
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

fn position(node: Node) -> (usize, usize) {
    let pos = node.start_position();
    (pos.row + 1, pos.column + 1)
}
