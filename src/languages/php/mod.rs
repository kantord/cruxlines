use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["php"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["name", "qualified_name"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_php::LANGUAGE_PHP.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "class_declaration"
        | "interface_declaration"
        | "trait_declaration"
        | "enum_declaration" => {
            if is_top_level(node)
                && let Some(name) = node.child_by_field_name("name")
                && let Some(location) = location_from_node(path, source, name)
            {
                emit(location);
            }
        }
        "function_definition" => {
            if is_top_level(node)
                && let Some(name) = node.child_by_field_name("name")
                && let Some(location) = location_from_node(path, source, name)
            {
                emit(location);
            }
        }
        "const_declaration" => {
            // const declarations can have multiple const_element children
            if is_top_level(node) {
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        if child.kind() == "const_element" {
                            // Find the name child (it's a child with kind "name", not a named field)
                            for j in 0..child.child_count() {
                                if let Some(name_node) = child.child(j) {
                                    if name_node.kind() == "name" {
                                        if let Some(location) =
                                            location_from_node(path, source, name_node)
                                        {
                                            emit(location);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
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
        if REFERENCE_KINDS.contains(&node.kind())
            && let Some(location) = location_from_node(path, source, node)
        {
            emit(location);
        }
    });
}


fn is_top_level(node: Node) -> bool {
    // In PHP, top-level items can be:
    // 1. Direct children of program
    // 2. Inside a namespace_definition
    // 3. Inside a declaration_list within a namespace
    let Some(parent) = node.parent() else {
        return false;
    };

    let parent_kind = parent.kind();

    // Direct child of program
    if parent_kind == "program" {
        return true;
    }

    // Inside a namespace
    if parent_kind == "namespace_definition" {
        return true;
    }

    // Inside a declaration_list (compound statement in namespace)
    if parent_kind == "declaration_list" || parent_kind == "compound_statement" {
        if let Some(grandparent) = parent.parent() {
            if grandparent.kind() == "namespace_definition" {
                return true;
            }
        }
    }

    false
}
