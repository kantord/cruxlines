use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["go"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier", "type_identifier", "field_identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_go::LANGUAGE.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "function_declaration" | "method_declaration" => {
            if is_top_level(node)
                && let Some(name) = node.child_by_field_name("name")
                && let Some(location) = location_from_node(path, source, name)
            {
                emit(location);
            }
        }
        "type_spec" | "const_spec" | "var_spec" => {
            if is_top_level_spec(node)
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

fn is_top_level_spec(node: Node) -> bool {
    // type_spec, const_spec, var_spec can be:
    // 1. Direct children of *_declaration (single declaration)
    // 2. Inside *_spec_list which is inside *_declaration (grouped declaration)
    let Some(parent) = node.parent() else {
        return false;
    };

    // Check for direct parent being a declaration
    if matches!(
        parent.kind(),
        "type_declaration" | "const_declaration" | "var_declaration"
    ) {
        return is_top_level(parent);
    }

    // Check for grouped declarations: spec -> spec_list -> declaration
    if matches!(
        parent.kind(),
        "type_spec_list" | "const_spec_list" | "var_spec_list"
    ) {
        if let Some(grandparent) = parent.parent() {
            return matches!(
                grandparent.kind(),
                "type_declaration" | "const_declaration" | "var_declaration"
            ) && is_top_level(grandparent);
        }
    }

    false
}
