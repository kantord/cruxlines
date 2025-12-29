use std::fs;
use std::path::{Path, PathBuf};

use cruxlines::{cruxlines_from_inputs, OutputRow};

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

    let rows = cruxlines_from_inputs(files, None);

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

    let rows = cruxlines_from_inputs(files, None);

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

    let rows = cruxlines_from_inputs(files, None);

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
fn finds_java_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/java/fixtures/Main.java"),
        read_fixture("src/languages/java/fixtures/Models.java"),
        read_fixture("src/languages/java/fixtures/Utils.java"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "User", "src/languages/java/fixtures/Models.java", "src/languages/java/fixtures/Main.java"),
        "expected reference to Models.User from Main.java"
    );
}

#[test]
fn finds_kotlin_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/kotlin/fixtures/main.kt"),
        read_fixture("src/languages/kotlin/fixtures/models.kt"),
        read_fixture("src/languages/kotlin/fixtures/utils.kt"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "add", "src/languages/kotlin/fixtures/utils.kt", "src/languages/kotlin/fixtures/main.kt"),
        "expected reference to utils.add from main.kt"
    );
    assert!(
        has_reference(&rows, "User", "src/languages/kotlin/fixtures/models.kt", "src/languages/kotlin/fixtures/main.kt"),
        "expected reference to models.User from main.kt"
    );
}

#[test]
fn finds_java_kotlin_cross_language_references() {
    let files = vec![
        read_fixture("src/languages/java/fixtures/InteropCall.java"),
        read_fixture("src/languages/java/fixtures/JavaUser.java"),
        read_fixture("src/languages/kotlin/fixtures/interop.kt"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "kotlinGreet", "src/languages/kotlin/fixtures/interop.kt", "src/languages/java/fixtures/InteropCall.java"),
        "expected reference to kotlinGreet from InteropCall.java"
    );
    assert!(
        has_reference(&rows, "JavaUser", "src/languages/java/fixtures/JavaUser.java", "src/languages/kotlin/fixtures/interop.kt"),
        "expected reference to JavaUser from interop.kt"
    );
}

#[test]
fn kotlin_references_are_not_duplicated() {
    let files = vec![
        (
            PathBuf::from("utils.kt"),
            "fun add(a: Int, b: Int): Int {\n    return a + b\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.kt"),
            "fun main() {\n    utils.add(1, 2)\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);
    let add_row = rows
        .iter()
        .find(|row| row.definition.name == "add")
        .expect("expected add definition");
    let ref_count = add_row
        .references
        .iter()
        .filter(|reference| reference.path.ends_with("main.kt"))
        .count();

    assert_eq!(
        ref_count, 1,
        "expected one reference for add, got {ref_count}"
    );
}

#[test]
fn finds_rust_type_identifier_references() {
    let files = vec![
        (
            PathBuf::from("models.rs"),
            "pub struct User;\n".to_string(),
        ),
        (
            PathBuf::from("main.rs"),
            "mod models;\n\nfn main() {\n    let _u: models::User;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "User", "models.rs", "main.rs"),
        "expected reference to models::User from main.rs type usage"
    );
}

#[test]
fn finds_typescript_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/javascript/fixtures/index.ts"),
        read_fixture("src/languages/javascript/fixtures/utils.ts"),
        read_fixture("src/languages/javascript/fixtures/models.ts"),
    ];

    let rows = cruxlines_from_inputs(files, None);

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

    let rows = cruxlines_from_inputs(files, None);

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

    let rows = cruxlines_from_inputs(files, None);

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

    let rows = cruxlines_from_inputs(files, None);
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

    let rows = cruxlines_from_inputs(files, None);
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

    let rows = cruxlines_from_inputs(files, None);
    assert!(
        !rows.iter().any(|row| row.definition.name == "inner"),
        "expected nested inner to be ignored"
    );
}

#[test]
fn ties_are_sorted_by_definition_location() {
    let mut files = Vec::new();
    let mut use_lines = String::new();
    for idx in 0..8 {
        let name = format!("symbol_{idx}");
        let path = format!("file_{idx}.py");
        files.push((
            PathBuf::from(&path),
            format!("def {name}():\n    return {idx}\n"),
        ));
        use_lines.push_str(&format!("from file_{idx} import {name}\n"));
    }
    use_lines.push('\n');
    for idx in 0..8 {
        use_lines.push_str(&format!("symbol_{idx}()\n"));
    }
    files.push((PathBuf::from("use.py"), use_lines));

    let rows = cruxlines_from_inputs(files, None);
    let mut expected = rows.clone();
    expected.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                let key_a = (
                    &a.definition.path,
                    a.definition.line,
                    a.definition.column,
                    &a.definition.name,
                );
                let key_b = (
                    &b.definition.path,
                    b.definition.line,
                    b.definition.column,
                    &b.definition.name,
                );
                key_a.cmp(&key_b)
            })
    });

    assert_eq!(
        rows.iter().map(|row| &row.definition.path).collect::<Vec<_>>(),
        expected.iter().map(|row| &row.definition.path).collect::<Vec<_>>(),
        "expected tie-breaker ordering by definition location"
    );
}
