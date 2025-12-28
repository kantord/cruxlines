use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use tree_sitter::Node;

use crate::find_references::{collect_references_by_kinds, record_definition, Location, ReferenceEdge};

pub(crate) const EXTENSIONS: &[&str] = &["rs"];
pub(crate) const REFERENCE_KINDS: &[&str] = &["identifier", "type_identifier"];

pub(crate) fn language() -> tree_sitter::Language {
    tree_sitter_rust::LANGUAGE.into()
}

pub(crate) fn collect_definition(
    path: &Path,
    source: &str,
    node: Node,
    definitions: &mut HashMap<String, Vec<Location>>,
    definition_positions: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match node.kind() {
        "function_item" | "struct_item" | "enum_item" | "const_item" | "static_item"
        | "type_item" | "trait_item" => {
            if is_top_level(node) {
                if let Some(name) = node.child_by_field_name("name") {
                    record_definition(path, source, name, definitions, definition_positions);
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn collect_references(
    path: &Path,
    source: &str,
    tree: &tree_sitter::Tree,
    definitions: &HashMap<String, Vec<Location>>,
    definition_positions: &HashSet<(PathBuf, usize, usize)>,
    ecosystem: crate::languages::Ecosystem,
    edges: &mut Vec<ReferenceEdge>,
) {
    collect_references_by_kinds(
        path,
        source,
        tree,
        definitions,
        definition_positions,
        REFERENCE_KINDS,
        ecosystem,
        edges,
    );
}

fn is_top_level(node: Node) -> bool {
    node.parent()
        .map(|parent| parent.kind() == "source_file")
        .unwrap_or(false)
}
