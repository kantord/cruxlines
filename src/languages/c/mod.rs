use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["c", "h"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier", "type_identifier", "field_identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_c::LANGUAGE.into()
}

pub(crate) fn emit_definitions(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    mut emit: impl FnMut(Location),
) {
    walk_tree(tree, |node| match node.kind() {
        "function_definition" => {
            if is_top_level(node) {
                if let Some(declarator) = node.child_by_field_name("declarator") {
                    if let Some(name) = find_identifier_in_declarator(declarator) {
                        if let Some(location) = location_from_node(path, source, name) {
                            emit(location);
                        }
                    }
                }
            }
        }
        "struct_specifier" | "enum_specifier" | "union_specifier" => {
            if is_top_level_type_specifier(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    if let Some(location) = location_from_node(path, source, name) {
                        emit(location);
                    }
                }
            }
        }
        "type_definition" => {
            if is_top_level(node) {
                if let Some(declarator) = node.child_by_field_name("declarator") {
                    if let Some(name) = find_identifier_in_declarator(declarator) {
                        if let Some(location) = location_from_node(path, source, name) {
                            emit(location);
                        }
                    }
                }
            }
        }
        "declaration" => {
            // Global variable declarations (can have multiple declarators like `int a, b, c;`)
            if is_top_level(node) && !is_function_declaration(node) {
                let mut cursor = node.walk();
                for child in node.children_by_field_name("declarator", &mut cursor) {
                    if let Some(name) = find_identifier_in_declarator(child) {
                        if let Some(location) = location_from_node(path, source, name) {
                            emit(location);
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
    node.parent()
        .is_some_and(|parent| parent.kind() == "translation_unit")
}

fn is_top_level_type_specifier(node: Node) -> bool {
    // Type specifiers can be inside a type_definition or declaration at top level
    let Some(parent) = node.parent() else {
        return false;
    };

    if parent.kind() == "translation_unit" {
        return true;
    }

    // Check if parent is a type_definition or declaration at top level
    if parent.kind() == "type_definition" || parent.kind() == "declaration" {
        return is_top_level(parent);
    }

    false
}

fn is_function_declaration(node: Node) -> bool {
    // A declaration is a function declaration if its declarator contains a parameter_list
    if let Some(declarator) = node.child_by_field_name("declarator") {
        return has_parameter_list(declarator);
    }
    false
}

fn has_parameter_list(node: Node) -> bool {
    if node.kind() == "function_declarator" {
        return true;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if has_parameter_list(child) {
                return true;
            }
        }
    }
    false
}

fn find_identifier_in_declarator(node: Node) -> Option<Node> {
    // Recursively find the identifier in a declarator (handles pointers, arrays, functions, parenthesized, init)
    match node.kind() {
        "identifier" => Some(node),
        "type_identifier" => Some(node),
        "pointer_declarator"
        | "array_declarator"
        | "function_declarator"
        | "parenthesized_declarator"
        | "init_declarator" => {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                find_identifier_in_declarator(declarator)
            } else {
                // For parenthesized_declarator, sometimes we need to search children
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        if let Some(found) = find_identifier_in_declarator(child) {
                            return Some(found);
                        }
                    }
                }
                None
            }
        }
        _ => None,
    }
}
