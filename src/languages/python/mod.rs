use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{
    collect_identifier_nodes, location_from_node, walk_tree, Location,
};

pub(crate) const EXTENSIONS: &[&str] = &["py"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_python::LANGUAGE.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "function_definition" | "class_definition" => {
            if is_top_level(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    if let Some(location) = location_from_node(path, source, name) {
                        emit(location);
                    }
                }
            }
        }
        "assignment" => {
            if is_top_level(node) {
                if let Some(left) = node.child_by_field_name("left") {
                    collect_identifier_nodes(left, source, |ident| {
                        if let Some(location) = location_from_node(path, source, ident) {
                            emit(location);
                        }
                    });
                }
            }
        }
        _ => {}
    });
}

pub(crate) fn emit_references(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| {
        if REFERENCE_KINDS.contains(&node.kind()) {
            if let Some(location) = location_from_node(path, source, node) {
                emit(location);
            }
        }
    });
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
