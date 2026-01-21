use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["rs"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier", "type_identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "function_item" | "struct_item" | "enum_item" | "const_item" | "static_item"
        | "type_item" | "trait_item" => {
            if is_top_level(node)
                && let Some(name) = node.child_by_field_name("name")
                && let Some(location) = location_from_node(path, source, name)
            {
                emit(location);
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

fn is_top_level(node: Node) -> bool {
    node.parent()
        .map(|parent| parent.kind() == "source_file")
        .unwrap_or(false)
}
