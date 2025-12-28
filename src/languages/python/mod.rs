use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use tree_sitter::Node;

use crate::find_references::{collect_identifier_nodes, record_definition, Location};

pub(crate) const EXTENSIONS: &[&str] = &["py"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_python::LANGUAGE.into()
}

pub(crate) fn collect_definition(
    path: &Path,
    source: &str,
    node: Node,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match node.kind() {
        "function_definition" | "class_definition" => {
            if is_top_level(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    record_definition(path, source, name, definitions, definition_positions);
                }
            }
        }
        "assignment" => {
            if is_top_level(node) {
                if let Some(left) = node.child_by_field_name("left") {
                    collect_identifier_nodes(left, source, |ident| {
                        record_definition(path, source, ident, definitions, definition_positions);
                    });
                }
            }
        }
        _ => {}
    }
}

fn is_top_level(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() == "module" {
        return true;
    }
    if parent.kind() == "decorated_definition" {
        return parent
            .parent()
            .map(|grand| grand.kind() == "module")
            .unwrap_or(false);
    }
    false
}
