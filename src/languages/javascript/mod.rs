use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use tree_sitter::Node;

use crate::find_references::{collect_identifier_nodes, record_definition, Location};

pub(crate) const EXTENSIONS: &[&str] = &["js"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_javascript::LANGUAGE.into()
}

pub(crate) fn collect_definition(
    path: &Path,
    source: &str,
    node: Node,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match node.kind() {
        "function_declaration" | "class_declaration" => {
            if is_exported(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    record_definition(path, source, name, definitions, definition_positions);
                }
            }
        }
        "variable_declarator" => {
            if is_exported(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    collect_identifier_nodes(name, source, |ident| {
                        record_definition(path, source, ident, definitions, definition_positions);
                    });
                }
            }
        }
        _ => {}
    }
}

fn is_exported(node: Node) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        let kind = parent.kind();
        if kind == "export_statement" || kind == "export_default_declaration" {
            return true;
        }
        if kind == "program" {
            break;
        }
        current = parent;
    }
    false
}
