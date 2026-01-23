use std::path::Path;

use tree_sitter::Node;

use crate::find_references::{Location, location_from_node, walk_tree};

pub(crate) const EXTENSIONS: &[&str] = &["cs"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier", "generic_name"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_c_sharp::LANGUAGE.into()
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
        | "struct_declaration"
        | "enum_declaration"
        | "record_declaration"
        | "record_struct_declaration"
        | "delegate_declaration" => {
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
    // In C#, top-level types can be:
    // 1. Direct children of compilation_unit
    // 2. Inside a namespace_declaration
    // 3. Inside a file_scoped_namespace_declaration (C# 10+)
    let Some(parent) = node.parent() else {
        return false;
    };

    let parent_kind = parent.kind();

    // Direct child of compilation unit
    if parent_kind == "compilation_unit" {
        return true;
    }

    // Inside a namespace (either block or file-scoped)
    if parent_kind == "namespace_declaration" || parent_kind == "file_scoped_namespace_declaration"
    {
        return true;
    }

    // Inside a declaration_list which is inside a namespace
    if parent_kind == "declaration_list" {
        if let Some(grandparent) = parent.parent() {
            let gp_kind = grandparent.kind();
            if gp_kind == "namespace_declaration"
                || gp_kind == "file_scoped_namespace_declaration"
            {
                return true;
            }
        }
    }

    false
}
