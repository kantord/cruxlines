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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Language {
    Python,
    JavaScript,
}

struct FileInput {
    path: PathBuf,
    source: String,
    language: Language,
    tree: Tree,
}

pub fn find_references<I, P>(files: I) -> impl Iterator<Item = ReferenceEdge>
where
    I: IntoIterator<Item = (P, String)>,
    P: Into<PathBuf>,
{
    let mut inputs = Vec::new();
    for (path, source) in files {
        let path = path.into();
        let Some(language) = crate::format_router::language_for_path(&path) else {
            continue;
        };
        let Some(tree) = parse_tree(&language, &source) else {
            continue;
        };
        inputs.push(FileInput {
            path,
            source,
            language,
            tree,
        });
    }

    let mut definitions: HashMap<String, Vec<Location>> = HashMap::new();
    let mut definition_positions: HashSet<(PathBuf, usize, usize)> = HashSet::new();

    for input in &inputs {
        collect_definitions(
            &input.path,
            &input.source,
            &input.language,
            &input.tree,
            &mut definitions,
            &mut definition_positions,
        );
    }

    let mut edges = Vec::new();
    for input in &inputs {
        collect_references(
            &input.path,
            &input.source,
            &input.tree,
            &definitions,
            &definition_positions,
            &mut edges,
        );
    }

    edges.into_iter()
}

fn parse_tree(language: &Language, source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    match language {
        Language::Python => {
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .ok()?;
        }
        Language::JavaScript => {
            parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .ok()?;
        }
    }
    parser.parse(source, None)
}

fn collect_definitions(
    path: &Path,
    source: &str,
    language: &Language,
    tree: &Tree,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    let root = tree.root_node();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match language {
            Language::Python => collect_python_definition(
                path,
                source,
                node,
                definitions,
                definition_positions,
            ),
            Language::JavaScript => collect_js_definition(
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

fn collect_python_definition(
    path: &Path,
    source: &str,
    node: Node,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match node.kind() {
        "function_definition" | "class_definition" => {
            if let Some(name) = node.child_by_field_name("name") {
                record_definition(path, source, name, definitions, definition_positions);
            }
        }
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                collect_identifier_nodes(left, source, |ident| {
                    record_definition(path, source, ident, definitions, definition_positions);
                });
            }
        }
        _ => {}
    }
}

fn collect_js_definition(
    path: &Path,
    source: &str,
    node: Node,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match node.kind() {
        "function_declaration" | "class_declaration" => {
            if let Some(name) = node.child_by_field_name("name") {
                record_definition(path, source, name, definitions, definition_positions);
            }
        }
        "variable_declarator" => {
            if let Some(name) = node.child_by_field_name("name") {
                collect_identifier_nodes(name, source, |ident| {
                    record_definition(path, source, ident, definitions, definition_positions);
                });
            }
        }
        _ => {}
    }
}

fn collect_references(
    path: &Path,
    source: &str,
    tree: &Tree,
    definitions: &HashMap<String, Vec<Location>>,
    definition_positions: &HashSet<(PathBuf, usize, usize)>,
    edges: &mut Vec<ReferenceEdge>,
) {
    let root = tree.root_node();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "identifier" {
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

fn record_definition(
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

fn collect_identifier_nodes<F>(node: Node, source: &str, mut on_ident: F)
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
