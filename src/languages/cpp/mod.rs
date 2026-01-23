use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["cpp", "cc", "cxx", "hpp", "hh", "hxx"];
pub(crate) const REFERENCE_KINDS: &[&str] = &[
    "identifier",
    "type_identifier",
    "field_identifier",
    "qualified_identifier",
];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_cpp::LANGUAGE.into()
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
        "class_specifier" | "struct_specifier" | "enum_specifier" | "union_specifier" => {
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
        "namespace_definition" => {
            if is_top_level(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    if let Some(location) = location_from_node(path, source, name) {
                        emit(location);
                    }
                }
            }
        }
        "declaration" => {
            // Global variable declarations (not function declarations)
            if is_top_level(node) && !is_function_declaration(node) {
                if let Some(declarator) = node.child_by_field_name("declarator") {
                    if let Some(name) = find_identifier_in_declarator(declarator) {
                        if let Some(location) = location_from_node(path, source, name) {
                            emit(location);
                        }
                    }
                }
            }
        }
        "template_declaration" => {
            // Template classes, structs, and functions
            if is_top_level(node) {
                // Find the actual declaration inside the template
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "class_specifier" | "struct_specifier" => {
                                if let Some(name) = child.child_by_field_name("name") {
                                    if let Some(location) = location_from_node(path, source, name) {
                                        emit(location);
                                    }
                                }
                            }
                            "function_definition" => {
                                if let Some(declarator) = child.child_by_field_name("declarator") {
                                    if let Some(name) = find_identifier_in_declarator(declarator) {
                                        if let Some(location) = location_from_node(path, source, name)
                                        {
                                            emit(location);
                                        }
                                    }
                                }
                            }
                            "declaration" => {
                                // Template function declaration (not definition)
                                if let Some(declarator) = child.child_by_field_name("declarator") {
                                    if let Some(name) = find_identifier_in_declarator(declarator) {
                                        if let Some(location) = location_from_node(path, source, name)
                                        {
                                            emit(location);
                                        }
                                    }
                                }
                            }
                            _ => {}
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
    let Some(parent) = node.parent() else {
        return false;
    };

    let parent_kind = parent.kind();

    // Direct child of translation_unit
    if parent_kind == "translation_unit" {
        return true;
    }

    // Inside a namespace
    if parent_kind == "namespace_definition" {
        return true;
    }

    // Inside a declaration_list within a namespace
    if parent_kind == "declaration_list" {
        if let Some(grandparent) = parent.parent() {
            if grandparent.kind() == "namespace_definition" {
                return true;
            }
        }
    }

    false
}

fn is_top_level_type_specifier(node: Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    let parent_kind = parent.kind();

    if parent_kind == "translation_unit" {
        return true;
    }

    // Inside a namespace directly
    if parent_kind == "namespace_definition" {
        return true;
    }

    // Inside a declaration_list within a namespace
    if parent_kind == "declaration_list" {
        if let Some(grandparent) = parent.parent() {
            if grandparent.kind() == "namespace_definition" {
                return true;
            }
        }
    }

    // Check if parent is a type_definition or declaration at top level
    if parent_kind == "type_definition" || parent_kind == "declaration" {
        return is_top_level(parent);
    }

    false
}

fn is_function_declaration(node: Node) -> bool {
    if let Some(declarator) = node.child_by_field_name("declarator") {
        return has_function_declarator(declarator);
    }
    false
}

fn has_function_declarator(node: Node) -> bool {
    if node.kind() == "function_declarator" {
        return true;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if has_function_declarator(child) {
                return true;
            }
        }
    }
    false
}

fn find_identifier_in_declarator(node: Node) -> Option<Node> {
    match node.kind() {
        "identifier" => Some(node),
        "type_identifier" => Some(node),
        "field_identifier" => Some(node),
        "pointer_declarator"
        | "reference_declarator"
        | "array_declarator"
        | "function_declarator" => {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                find_identifier_in_declarator(declarator)
            } else {
                None
            }
        }
        "qualified_identifier" => {
            // For qualified identifiers like MyClass::method, get the name part
            if let Some(name) = node.child_by_field_name("name") {
                Some(name)
            } else {
                None
            }
        }
        _ => None,
    }
}
