use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["kt", "kts"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["simple_identifier", "identifier", "type_identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_kotlin_ng::LANGUAGE.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "class_declaration"
        | "object_declaration"
        | "function_declaration"
        | "property_declaration"
        | "type_alias" => {
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
