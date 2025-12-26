use std::fs;
use std::path::{Path, PathBuf};

use cruxlines::find_references::{find_references, ReferenceEdge};

fn read_fixture(path: impl AsRef<Path>) -> (PathBuf, String) {
    let path = path.as_ref().to_path_buf();
    let contents = fs::read_to_string(&path).expect("read fixture");
    (path, contents)
}

fn has_edge(edges: &[ReferenceEdge], def_name: &str, def_path_ends: &str, use_path_ends: &str) -> bool {
    edges.iter().any(|edge| {
        edge.definition.name == def_name
            && edge.definition.path.ends_with(def_path_ends)
            && edge.usage.path.ends_with(use_path_ends)
    })
}

#[test]
fn finds_python_cross_file_references() {
    let files = vec![
        read_fixture("fixtures/python/main.py"),
        read_fixture("fixtures/python/utils.py"),
        read_fixture("fixtures/python/models.py"),
    ];

    let edges: Vec<_> = find_references(files).collect();

    assert!(
        has_edge(&edges, "add", "fixtures/python/utils.py", "fixtures/python/main.py"),
        "expected reference to utils.add from main.py"
    );
    assert!(
        has_edge(&edges, "User", "fixtures/python/models.py", "fixtures/python/main.py"),
        "expected reference to models.User from main.py"
    );
}

#[test]
fn finds_javascript_cross_file_references() {
    let files = vec![
        read_fixture("fixtures/javascript/index.js"),
        read_fixture("fixtures/javascript/utils.js"),
        read_fixture("fixtures/javascript/models.js"),
    ];

    let edges: Vec<_> = find_references(files).collect();

    assert!(
        has_edge(&edges, "add", "fixtures/javascript/utils.js", "fixtures/javascript/index.js"),
        "expected reference to utils.add from index.js"
    );
    assert!(
        has_edge(&edges, "User", "fixtures/javascript/models.js", "fixtures/javascript/index.js"),
        "expected reference to models.User from index.js"
    );
}

#[test]
fn finds_rust_cross_file_references() {
    let files = vec![
        read_fixture("fixtures/rust/main.rs"),
        read_fixture("fixtures/rust/utils.rs"),
        read_fixture("fixtures/rust/models.rs"),
    ];

    let edges: Vec<_> = find_references(files).collect();

    assert!(
        has_edge(&edges, "add", "fixtures/rust/utils.rs", "fixtures/rust/main.rs"),
        "expected reference to utils::add from main.rs"
    );
    assert!(
        has_edge(&edges, "User", "fixtures/rust/models.rs", "fixtures/rust/main.rs"),
        "expected reference to models::User from main.rs"
    );
}
