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

fn extension(path: &Path) -> Option<String> {
    path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_string())
}

#[test]
fn finds_python_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/python/fixtures/main.py"),
        read_fixture("src/languages/python/fixtures/utils.py"),
        read_fixture("src/languages/python/fixtures/models.py"),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "src/languages/python/fixtures/utils.py", "src/languages/python/fixtures/main.py"),
        "expected reference to utils.add from main.py"
    );
    assert!(
        has_reference(&rows, "User", "src/languages/python/fixtures/models.py", "src/languages/python/fixtures/main.py"),
        "expected reference to models.User from main.py"
    );
}

#[test]
fn finds_javascript_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/javascript/fixtures/index.js"),
        read_fixture("src/languages/javascript/fixtures/utils.js"),
        read_fixture("src/languages/javascript/fixtures/models.js"),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "src/languages/javascript/fixtures/utils.js", "src/languages/javascript/fixtures/index.js"),
        "expected reference to utils.add from index.js"
    );
    assert!(
        has_reference(&rows, "User", "src/languages/javascript/fixtures/models.js", "src/languages/javascript/fixtures/index.js"),
        "expected reference to models.User from index.js"
    );
}

#[test]
fn finds_rust_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/rust/fixtures/main.rs"),
        read_fixture("src/languages/rust/fixtures/utils.rs"),
        read_fixture("src/languages/rust/fixtures/models.rs"),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "src/languages/rust/fixtures/utils.rs", "src/languages/rust/fixtures/main.rs"),
        "expected reference to utils::add from main.rs"
    );
    assert!(
        has_reference(&rows, "User", "src/languages/rust/fixtures/models.rs", "src/languages/rust/fixtures/main.rs"),
        "expected reference to models::User from main.rs"
    );
}

#[test]
fn finds_typescript_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/javascript/fixtures/index.ts"),
        read_fixture("src/languages/javascript/fixtures/utils.ts"),
        read_fixture("src/languages/javascript/fixtures/models.ts"),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "src/languages/javascript/fixtures/utils.ts", "src/languages/javascript/fixtures/index.ts"),
        "expected reference to utils.add from index.ts"
    );
    assert!(
        has_reference(&rows, "User", "src/languages/javascript/fixtures/models.ts", "src/languages/javascript/fixtures/index.ts"),
        "expected reference to models.User from index.ts"
    );
}

#[test]
fn finds_tsx_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/javascript/fixtures/index.tsx"),
        read_fixture("src/languages/javascript/fixtures/components.tsx"),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "Button", "src/languages/javascript/fixtures/components.tsx", "src/languages/javascript/fixtures/index.tsx"),
        "expected reference to components.Button from index.tsx"
    );
}

#[test]
fn finds_cross_language_references_within_ecosystem() {
    let files = vec![
        (
            PathBuf::from("utils.ts"),
            "export function add(a: number, b: number): number {\n    return a + b;\n}\n"
                .to_string(),
        ),
        (
            PathBuf::from("main.js"),
            "import { add } from \"./utils\";\nconsole.log(add(1, 2));\n".to_string(),
        ),
    ];

    let rows = cruxlines(files);

    assert!(
        has_reference(&rows, "add", "utils.ts", "main.js"),
        "expected reference to utils.ts add from main.js"
    );
}

#[test]
fn does_not_cross_language_references() {
    let files = vec![
        (
            PathBuf::from("a.py"),
            "def add():\n    return 1\n\nadd()\n".to_string(),
        ),
        (
            PathBuf::from("b.rs"),
            "fn add() -> i32 { 1 }\n\nfn main() {\n    add();\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines(files);
    for row in &rows {
        let def_ext = extension(&row.definition.path);
        for reference in &row.references {
            let ref_ext = extension(&reference.path);
            assert_eq!(
                def_ext, ref_ext,
                "expected references to stay within language, got {:?} -> {:?}",
                row.definition.path, reference.path
            );
        }
    }
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
