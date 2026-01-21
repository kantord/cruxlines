use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, collect_identifier_nodes, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["js", "jsx"];
pub(crate) const TYPESCRIPT_EXTENSIONS: &[&str] = &["ts"];
pub(crate) const TSX_EXTENSIONS: &[&str] = &["tsx"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier", "jsx_identifier", "type_identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_javascript::LANGUAGE.into()
}

pub(crate) fn language_typescript() -> tree_sitter::Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

pub(crate) fn language_tsx() -> tree_sitter::Language {
    tree_sitter_typescript::LANGUAGE_TSX.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "function_declaration"
        | "class_declaration"
        | "interface_declaration"
        | "type_alias_declaration"
        | "enum_declaration" => {
            if is_exported(node)
                && let Some(name) = node.child_by_field_name("name")
                && let Some(location) = location_from_node(path, source, name)
            {
                emit(location);
            }
        }
        "variable_declarator" => {
            if is_exported(node)
                && let Some(name) = node.child_by_field_name("name")
            {
                collect_identifier_nodes(name, source, |ident| {
                    if let Some(location) = location_from_node(path, source, ident) {
                        emit(location);
                    }
                });
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
        if REFERENCE_KINDS.contains(&node.kind())
            && let Some(location) = location_from_node(path, source, node)
        {
            emit(location);
        }
    });
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
