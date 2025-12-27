use std::fs;
use std::path::{Path, PathBuf};

use cruxlines::{cruxlines, OutputRow};

fn read_fixture(path: impl AsRef<Path>) -> (PathBuf, String) {
    let path = path.as_ref().to_path_buf();
    let contents = fs::read_to_string(&path).expect("read fixture");
    (path, contents)
}

fn has_reference(rows: &[OutputRow], def_name: &str, def_path_ends: &str, use_path_ends: &str) -> bool {
    rows.iter().any(|row| {
        row.definition.name == def_name
            && row.definition.path.ends_with(def_path_ends)
            && row
                .references
                .iter()
                .any(|reference| reference.path.ends_with(use_path_ends))
    })
}

#[test]
fn finds_python_cross_file_references() {
    let files = vec![
        read_fixture("fixtures/python/main.py"),
        read_fixture("fixtures/python/utils.py"),
        read_fixture("fixtures/python/models.py"),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "fixtures/python/utils.py", "fixtures/python/main.py"),
        "expected reference to utils.add from main.py"
    );
    assert!(
        has_reference(&rows, "User", "fixtures/python/models.py", "fixtures/python/main.py"),
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

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "fixtures/javascript/utils.js", "fixtures/javascript/index.js"),
        "expected reference to utils.add from index.js"
    );
    assert!(
        has_reference(&rows, "User", "fixtures/javascript/models.js", "fixtures/javascript/index.js"),
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

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "fixtures/rust/utils.rs", "fixtures/rust/main.rs"),
        "expected reference to utils::add from main.rs"
    );
    assert!(
        has_reference(&rows, "User", "fixtures/rust/models.rs", "fixtures/rust/main.rs"),
        "expected reference to models::User from main.rs"
    );
}

#[test]
fn ignores_non_exported_javascript_definitions() {
    let files = vec![
        (
            PathBuf::from("a.js"),
            "function foo() { return 1; }\n".to_string(),
        ),
        (
            PathBuf::from("b.js"),
            "import { foo } from \"./a.js\";\nfoo();\n".to_string(),
        ),
    ];

    let rows = cruxlines(files);
    assert!(
        !rows.iter().any(|row| row.definition.name == "foo"),
        "expected non-exported foo to be ignored"
    );
}

#[test]
fn ignores_nested_python_definitions() {
    let files = vec![(
        PathBuf::from("a.py"),
        "def outer():\n    def inner():\n        return 1\n    return inner()\n".to_string(),
    )];

    let rows = cruxlines(files);
    assert!(
        !rows.iter().any(|row| row.definition.name == "inner"),
        "expected nested inner to be ignored"
    );
}
