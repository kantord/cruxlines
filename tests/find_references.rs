use std::fs;
use std::path::{Path, PathBuf};

use cruxlines::{OutputRow, cruxlines_from_inputs};

fn read_fixture(path: impl AsRef<Path>) -> (PathBuf, String) {
    let path = path.as_ref().to_path_buf();
    let contents = fs::read_to_string(&path).expect("read fixture");
    (path, contents)
}

fn has_reference(
    rows: &[OutputRow],
    def_name: &str,
    def_path_ends: &str,
    use_path_ends: &str,
) -> bool {
    rows.iter().any(|row| {
        row.definition.name_str() == def_name
            && row.definition.path_str().ends_with(def_path_ends)
            && row
                .references
                .iter()
                .any(|reference| reference.path_str().ends_with(use_path_ends))
    })
}

fn extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_string())
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
        has_reference(
            &rows,
            "add",
            "src/languages/python/fixtures/utils.py",
            "src/languages/python/fixtures/main.py"
        ),
        "expected reference to utils.add from main.py"
    );
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/python/fixtures/models.py",
            "src/languages/python/fixtures/main.py"
        ),
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
        has_reference(
            &rows,
            "add",
            "src/languages/javascript/fixtures/utils.js",
            "src/languages/javascript/fixtures/index.js"
        ),
        "expected reference to utils.add from index.js"
    );
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/javascript/fixtures/models.js",
            "src/languages/javascript/fixtures/index.js"
        ),
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
        has_reference(
            &rows,
            "add",
            "src/languages/rust/fixtures/utils.rs",
            "src/languages/rust/fixtures/main.rs"
        ),
        "expected reference to utils::add from main.rs"
    );
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/rust/fixtures/models.rs",
            "src/languages/rust/fixtures/main.rs"
        ),
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
        has_reference(
            &rows,
            "User",
            "src/languages/java/fixtures/Models.java",
            "src/languages/java/fixtures/Main.java"
        ),
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
        has_reference(
            &rows,
            "add",
            "src/languages/kotlin/fixtures/utils.kt",
            "src/languages/kotlin/fixtures/main.kt"
        ),
        "expected reference to utils.add from main.kt"
    );
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/kotlin/fixtures/models.kt",
            "src/languages/kotlin/fixtures/main.kt"
        ),
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
        has_reference(
            &rows,
            "kotlinGreet",
            "src/languages/kotlin/fixtures/interop.kt",
            "src/languages/java/fixtures/InteropCall.java"
        ),
        "expected reference to kotlinGreet from InteropCall.java"
    );
    assert!(
        has_reference(
            &rows,
            "JavaUser",
            "src/languages/java/fixtures/JavaUser.java",
            "src/languages/kotlin/fixtures/interop.kt"
        ),
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
        .find(|row| row.definition.name_str() == "add")
        .expect("expected add definition");
    let ref_count = add_row
        .references
        .iter()
        .filter(|reference| reference.path_str().ends_with("main.kt"))
        .count();

    assert_eq!(
        ref_count, 1,
        "expected one reference for add, got {ref_count}"
    );
}

#[test]
fn finds_rust_type_identifier_references() {
    let files = vec![
        (PathBuf::from("models.rs"), "pub struct User;\n".to_string()),
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
        has_reference(
            &rows,
            "add",
            "src/languages/javascript/fixtures/utils.ts",
            "src/languages/javascript/fixtures/index.ts"
        ),
        "expected reference to utils.add from index.ts"
    );
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/javascript/fixtures/models.ts",
            "src/languages/javascript/fixtures/index.ts"
        ),
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
        has_reference(
            &rows,
            "Button",
            "src/languages/javascript/fixtures/components.tsx",
            "src/languages/javascript/fixtures/index.tsx"
        ),
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
        let def_ext = extension(row.definition.path_str());
        for reference in &row.references {
            let ref_ext = extension(reference.path_str());
            assert_eq!(
                def_ext,
                ref_ext,
                "expected references to stay within language, got {:?} -> {:?}",
                row.definition.path_str(),
                reference.path_str()
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
        !rows.iter().any(|row| row.definition.name_str() == "foo"),
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
        !rows.iter().any(|row| row.definition.name_str() == "inner"),
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
        rows.iter()
            .map(|row| &row.definition.path)
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|row| &row.definition.path)
            .collect::<Vec<_>>(),
        "expected tie-breaker ordering by definition location"
    );
}

#[test]
fn finds_go_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/go/fixtures/main.go"),
        read_fixture("src/languages/go/fixtures/utils.go"),
        read_fixture("src/languages/go/fixtures/models.go"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(
            &rows,
            "Add",
            "src/languages/go/fixtures/utils.go",
            "src/languages/go/fixtures/main.go"
        ),
        "expected reference to utils.Add from main.go"
    );
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/go/fixtures/models.go",
            "src/languages/go/fixtures/main.go"
        ),
        "expected reference to models.User from main.go"
    );
    assert!(
        has_reference(
            &rows,
            "NewUser",
            "src/languages/go/fixtures/models.go",
            "src/languages/go/fixtures/main.go"
        ),
        "expected reference to models.NewUser from main.go"
    );
}

#[test]
fn finds_go_constant_definitions() {
    let files = vec![
        (
            PathBuf::from("constants.go"),
            "package main\n\nconst MaxRetries = 3\nconst DefaultTimeout = 30\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc main() {\n\tretries := MaxRetries\n\ttimeout := DefaultTimeout\n\t_ = retries + timeout\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "MaxRetries", "constants.go", "main.go"),
        "expected reference to MaxRetries constant from main.go"
    );
    assert!(
        has_reference(&rows, "DefaultTimeout", "constants.go", "main.go"),
        "expected reference to DefaultTimeout constant from main.go"
    );
}

#[test]
fn finds_go_variable_definitions() {
    let files = vec![
        (
            PathBuf::from("globals.go"),
            "package main\n\nvar GlobalCounter int\nvar AppName = \"myapp\"\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc main() {\n\tGlobalCounter++\n\tprintln(AppName)\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "GlobalCounter", "globals.go", "main.go"),
        "expected reference to GlobalCounter variable from main.go"
    );
    assert!(
        has_reference(&rows, "AppName", "globals.go", "main.go"),
        "expected reference to AppName variable from main.go"
    );
}

#[test]
fn finds_go_method_definitions() {
    let files = vec![
        (
            PathBuf::from("user.go"),
            "package main\n\ntype User struct {\n\tName string\n}\n\nfunc (u *User) Greet() string {\n\treturn \"Hello, \" + u.Name\n}\n\nfunc (u User) GetName() string {\n\treturn u.Name\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc main() {\n\tu := &User{Name: \"Alice\"}\n\tprintln(u.Greet())\n\tprintln(u.GetName())\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Greet", "user.go", "main.go"),
        "expected reference to Greet method from main.go"
    );
    assert!(
        has_reference(&rows, "GetName", "user.go", "main.go"),
        "expected reference to GetName method from main.go"
    );
}

#[test]
fn finds_go_type_identifier_references() {
    let files = vec![
        (
            PathBuf::from("types.go"),
            "package main\n\ntype Config struct {\n\tHost string\n\tPort int\n}\n\ntype Handler interface {\n\tHandle()\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc NewConfig() *Config {\n\treturn &Config{}\n}\n\nfunc process(h Handler) {\n\th.Handle()\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Config", "types.go", "main.go"),
        "expected reference to Config type from main.go"
    );
    assert!(
        has_reference(&rows, "Handler", "types.go", "main.go"),
        "expected reference to Handler interface from main.go"
    );
}

#[test]
fn ignores_go_nested_function_definitions() {
    let files = vec![
        (
            PathBuf::from("funcs.go"),
            "package main\n\nfunc outer() {\n\tinner := func() int {\n\t\treturn 1\n\t}\n\t_ = inner()\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            // Reference both outer and inner to test which appears as definition
            "package main\n\nfunc main() {\n\touter()\n\t// inner would be referenced here if it were exported\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    // outer is a top-level function and should be a definition
    assert!(
        has_reference(&rows, "outer", "funcs.go", "main.go"),
        "expected outer function to be a definition with reference from main.go"
    );
    // inner is a local variable holding a closure, not a top-level definition
    // It should not appear as a definition at all
    assert!(
        !rows.iter().any(|row| row.definition.name_str() == "inner"),
        "expected nested closure variable to not be a top-level definition"
    );
}

#[test]
fn finds_go_multiple_const_in_block() {
    let files = vec![
        (
            PathBuf::from("constants.go"),
            "package main\n\nconst (\n\tFoo = 1\n\tBar = 2\n\tBaz = 3\n)\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc main() {\n\t_ = Foo + Bar + Baz\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Foo", "constants.go", "main.go"),
        "expected reference to Foo from const block"
    );
    assert!(
        has_reference(&rows, "Bar", "constants.go", "main.go"),
        "expected reference to Bar from const block"
    );
    assert!(
        has_reference(&rows, "Baz", "constants.go", "main.go"),
        "expected reference to Baz from const block"
    );
}

#[test]
fn finds_go_multiple_var_in_block() {
    let files = vec![
        (
            PathBuf::from("globals.go"),
            "package main\n\nvar (\n\tX int\n\tY int\n\tZ = 100\n)\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc main() {\n\t_ = X + Y + Z\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "X", "globals.go", "main.go"),
        "expected reference to X from var block"
    );
    assert!(
        has_reference(&rows, "Y", "globals.go", "main.go"),
        "expected reference to Y from var block"
    );
    assert!(
        has_reference(&rows, "Z", "globals.go", "main.go"),
        "expected reference to Z from var block"
    );
}

#[test]
fn finds_go_multiple_type_in_block() {
    let files = vec![
        (
            PathBuf::from("types.go"),
            "package main\n\ntype (\n\tPoint struct{ X, Y int }\n\tSize struct{ W, H int }\n)\n".to_string(),
        ),
        (
            PathBuf::from("main.go"),
            "package main\n\nfunc main() {\n\tp := Point{X: 1, Y: 2}\n\ts := Size{W: 10, H: 20}\n\t_ = p\n\t_ = s\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Point", "types.go", "main.go"),
        "expected reference to Point from type block"
    );
    assert!(
        has_reference(&rows, "Size", "types.go", "main.go"),
        "expected reference to Size from type block"
    );
}

#[test]
fn finds_csharp_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/csharp/fixtures/Program.cs"),
        read_fixture("src/languages/csharp/fixtures/Models.cs"),
        read_fixture("src/languages/csharp/fixtures/Services.cs"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    // Classes
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/csharp/fixtures/Models.cs",
            "src/languages/csharp/fixtures/Program.cs"
        ),
        "expected reference to User class from Program.cs"
    );
    assert!(
        has_reference(
            &rows,
            "Calculator",
            "src/languages/csharp/fixtures/Services.cs",
            "src/languages/csharp/fixtures/Program.cs"
        ),
        "expected reference to Calculator class from Program.cs"
    );

    // Enum
    assert!(
        has_reference(
            &rows,
            "OrderStatus",
            "src/languages/csharp/fixtures/Models.cs",
            "src/languages/csharp/fixtures/Program.cs"
        ),
        "expected reference to OrderStatus enum from Program.cs"
    );

    // Interface
    assert!(
        has_reference(
            &rows,
            "IRepository",
            "src/languages/csharp/fixtures/Models.cs",
            "src/languages/csharp/fixtures/Program.cs"
        ),
        "expected reference to IRepository interface from Program.cs"
    );

    // Implementation references interface
    assert!(
        has_reference(
            &rows,
            "IRepository",
            "src/languages/csharp/fixtures/Models.cs",
            "src/languages/csharp/fixtures/Services.cs"
        ),
        "expected reference to IRepository from Services.cs"
    );
}

#[test]
fn finds_csharp_interface_definitions() {
    let files = vec![
        (
            PathBuf::from("Interfaces.cs"),
            "public interface IRepository {\n    void Save();\n}\n\npublic interface IService {\n    void Execute();\n}\n".to_string(),
        ),
        (
            PathBuf::from("Implementation.cs"),
            "public class MyRepo : IRepository {\n    public void Save() { }\n}\n\npublic class MyService : IService {\n    public void Execute() { }\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "IRepository", "Interfaces.cs", "Implementation.cs"),
        "expected reference to IRepository interface"
    );
    assert!(
        has_reference(&rows, "IService", "Interfaces.cs", "Implementation.cs"),
        "expected reference to IService interface"
    );
}

#[test]
fn finds_csharp_struct_definitions() {
    let files = vec![
        (
            PathBuf::from("Structs.cs"),
            "public struct Point {\n    public int X;\n    public int Y;\n}\n\npublic struct Size {\n    public int Width;\n    public int Height;\n}\n".to_string(),
        ),
        (
            PathBuf::from("Usage.cs"),
            "public class Canvas {\n    public Point Origin;\n    public Size Dimensions;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Point", "Structs.cs", "Usage.cs"),
        "expected reference to Point struct"
    );
    assert!(
        has_reference(&rows, "Size", "Structs.cs", "Usage.cs"),
        "expected reference to Size struct"
    );
}

#[test]
fn finds_csharp_enum_definitions() {
    let files = vec![
        (
            PathBuf::from("Enums.cs"),
            "public enum Status {\n    Active,\n    Inactive,\n    Pending\n}\n\npublic enum Priority {\n    Low,\n    Medium,\n    High\n}\n".to_string(),
        ),
        (
            PathBuf::from("Task.cs"),
            "public class Task {\n    public Status CurrentStatus { get; set; }\n    public Priority TaskPriority { get; set; }\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Status", "Enums.cs", "Task.cs"),
        "expected reference to Status enum"
    );
    assert!(
        has_reference(&rows, "Priority", "Enums.cs", "Task.cs"),
        "expected reference to Priority enum"
    );
}

#[test]
fn finds_csharp_record_definitions() {
    let files = vec![
        (
            PathBuf::from("Records.cs"),
            "public record Person(string Name, int Age);\n\npublic record Address(string Street, string City);\n".to_string(),
        ),
        (
            PathBuf::from("Usage.cs"),
            "public class Registry {\n    public Person Owner { get; set; }\n    public Address Location { get; set; }\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Person", "Records.cs", "Usage.cs"),
        "expected reference to Person record"
    );
    assert!(
        has_reference(&rows, "Address", "Records.cs", "Usage.cs"),
        "expected reference to Address record"
    );
}

#[test]
fn finds_csharp_delegate_definitions() {
    let files = vec![
        (
            PathBuf::from("Delegates.cs"),
            "public delegate void EventHandler(object sender);\n\npublic delegate int Calculator(int a, int b);\n".to_string(),
        ),
        (
            PathBuf::from("Usage.cs"),
            "public class Button {\n    public EventHandler OnClick;\n    public Calculator Compute;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "EventHandler", "Delegates.cs", "Usage.cs"),
        "expected reference to EventHandler delegate"
    );
    assert!(
        has_reference(&rows, "Calculator", "Delegates.cs", "Usage.cs"),
        "expected reference to Calculator delegate"
    );
}

#[test]
fn finds_csharp_types_in_namespace() {
    let files = vec![
        (
            PathBuf::from("Models.cs"),
            "namespace MyApp.Models {\n    public class Customer {\n        public string Name { get; set; }\n    }\n}\n".to_string(),
        ),
        (
            PathBuf::from("Service.cs"),
            "namespace MyApp.Services {\n    public class CustomerService {\n        public Customer GetCustomer() { return null; }\n    }\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Customer", "Models.cs", "Service.cs"),
        "expected reference to Customer class inside namespace"
    );
}

#[test]
fn finds_csharp_generic_type_references() {
    let files = vec![
        (
            PathBuf::from("Generic.cs"),
            "public class Repository<T> {\n    public T Get(int id) { return default; }\n}\n".to_string(),
        ),
        (
            PathBuf::from("Usage.cs"),
            "public class UserService {\n    private Repository<User> repo;\n}\n\npublic class User { }\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Repository", "Generic.cs", "Usage.cs"),
        "expected reference to generic Repository class"
    );
}

#[test]
fn ignores_csharp_nested_class_definitions() {
    let files = vec![
        (
            PathBuf::from("Outer.cs"),
            "public class Outer {\n    private class Inner {\n        public int Value;\n    }\n}\n".to_string(),
        ),
        (
            PathBuf::from("Usage.cs"),
            "public class Test {\n    public Outer outer;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    // Outer should be a definition
    assert!(
        has_reference(&rows, "Outer", "Outer.cs", "Usage.cs"),
        "expected Outer to be a definition"
    );
    // Inner should NOT be a top-level definition
    assert!(
        !rows.iter().any(|row| row.definition.name_str() == "Inner"),
        "expected nested Inner class to not be a top-level definition"
    );
}

#[test]
fn finds_php_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/php/fixtures/index.php"),
        read_fixture("src/languages/php/fixtures/Models.php"),
        read_fixture("src/languages/php/fixtures/Services.php"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    // Classes
    assert!(
        has_reference(
            &rows,
            "User",
            "src/languages/php/fixtures/Models.php",
            "src/languages/php/fixtures/index.php"
        ),
        "expected reference to User class from index.php"
    );
    assert!(
        has_reference(
            &rows,
            "Calculator",
            "src/languages/php/fixtures/Services.php",
            "src/languages/php/fixtures/index.php"
        ),
        "expected reference to Calculator class from index.php"
    );

    // Interface
    assert!(
        has_reference(
            &rows,
            "Repository",
            "src/languages/php/fixtures/Models.php",
            "src/languages/php/fixtures/Services.php"
        ),
        "expected reference to Repository interface from Services.php"
    );
}

#[test]
fn finds_php_class_definitions() {
    let files = vec![
        (
            PathBuf::from("Models.php"),
            "<?php\nclass User {\n    public string $name;\n}\n\nclass Order {\n    public int $id;\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.php"),
            "<?php\n$user = new User();\n$order = new Order();\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "User", "Models.php", "main.php"),
        "expected reference to User class"
    );
    assert!(
        has_reference(&rows, "Order", "Models.php", "main.php"),
        "expected reference to Order class"
    );
}

#[test]
fn finds_php_interface_definitions() {
    let files = vec![
        (
            PathBuf::from("Interfaces.php"),
            "<?php\ninterface Repository {\n    public function find(int $id);\n}\n\ninterface Service {\n    public function execute();\n}\n".to_string(),
        ),
        (
            PathBuf::from("Implementation.php"),
            "<?php\nclass UserRepo implements Repository {\n    public function find(int $id) { return null; }\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Repository", "Interfaces.php", "Implementation.php"),
        "expected reference to Repository interface"
    );
}

#[test]
fn finds_php_trait_definitions() {
    let files = vec![
        (
            PathBuf::from("Traits.php"),
            "<?php\ntrait Timestampable {\n    public function touch() {}\n}\n\ntrait Loggable {\n    public function log() {}\n}\n".to_string(),
        ),
        (
            PathBuf::from("Model.php"),
            "<?php\nclass User {\n    use Timestampable;\n    use Loggable;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Timestampable", "Traits.php", "Model.php"),
        "expected reference to Timestampable trait"
    );
    assert!(
        has_reference(&rows, "Loggable", "Traits.php", "Model.php"),
        "expected reference to Loggable trait"
    );
}

#[test]
fn finds_php_enum_definitions() {
    let files = vec![
        (
            PathBuf::from("Enums.php"),
            "<?php\nenum Status {\n    case Active;\n    case Inactive;\n}\n\nenum Priority: int {\n    case Low = 1;\n    case High = 2;\n}\n".to_string(),
        ),
        (
            PathBuf::from("Model.php"),
            "<?php\nclass Task {\n    public Status $status;\n    public Priority $priority;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Status", "Enums.php", "Model.php"),
        "expected reference to Status enum"
    );
    assert!(
        has_reference(&rows, "Priority", "Enums.php", "Model.php"),
        "expected reference to Priority enum"
    );
}

#[test]
fn finds_php_function_definitions() {
    let files = vec![
        (
            PathBuf::from("helpers.php"),
            "<?php\nfunction add(int $a, int $b): int {\n    return $a + $b;\n}\n\nfunction multiply(int $a, int $b): int {\n    return $a * $b;\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.php"),
            "<?php\n$sum = add(1, 2);\n$product = multiply(3, 4);\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "add", "helpers.php", "main.php"),
        "expected reference to add function"
    );
    assert!(
        has_reference(&rows, "multiply", "helpers.php", "main.php"),
        "expected reference to multiply function"
    );
}

#[test]
fn finds_php_const_definitions() {
    let files = vec![
        (
            PathBuf::from("constants.php"),
            "<?php\nconst MAX_SIZE = 100;\nconst DEFAULT_NAME = 'unnamed';\n".to_string(),
        ),
        (
            PathBuf::from("main.php"),
            "<?php\n$size = MAX_SIZE;\n$name = DEFAULT_NAME;\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "MAX_SIZE", "constants.php", "main.php"),
        "expected reference to MAX_SIZE constant"
    );
    assert!(
        has_reference(&rows, "DEFAULT_NAME", "constants.php", "main.php"),
        "expected reference to DEFAULT_NAME constant"
    );
}

#[test]
fn finds_php_types_in_namespace() {
    let files = vec![
        (
            PathBuf::from("Models.php"),
            "<?php\nnamespace App\\Models;\n\nclass Customer {\n    public string $name;\n}\n".to_string(),
        ),
        (
            PathBuf::from("Service.php"),
            "<?php\nnamespace App\\Services;\n\nuse App\\Models\\Customer;\n\nclass CustomerService {\n    public function get(): Customer { return new Customer(); }\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Customer", "Models.php", "Service.php"),
        "expected reference to Customer class in namespace"
    );
}

// C Tests

#[test]
fn finds_c_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/c/fixtures/main.c"),
        read_fixture("src/languages/c/fixtures/types.h"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    // Function references
    assert!(
        has_reference(
            &rows,
            "add",
            "src/languages/c/fixtures/main.c",
            "src/languages/c/fixtures/types.h"
        ),
        "expected reference to add function from types.h"
    );
    assert!(
        has_reference(
            &rows,
            "create_point",
            "src/languages/c/fixtures/main.c",
            "src/languages/c/fixtures/types.h"
        ),
        "expected reference to create_point function from types.h"
    );
}

#[test]
fn finds_c_function_definitions() {
    let files = vec![
        (
            PathBuf::from("math.c"),
            "int add(int a, int b) {\n    return a + b;\n}\n\nint multiply(int a, int b) {\n    return a * b;\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "int main(void) {\n    int x = add(1, 2);\n    int y = multiply(3, 4);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "add", "math.c", "main.c"),
        "expected reference to add function"
    );
    assert!(
        has_reference(&rows, "multiply", "math.c", "main.c"),
        "expected reference to multiply function"
    );
}

#[test]
fn finds_c_struct_definitions() {
    let files = vec![
        (
            PathBuf::from("types.h"),
            "struct Point {\n    int x;\n    int y;\n};\n\nstruct Rectangle {\n    struct Point origin;\n    int width;\n    int height;\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "#include \"types.h\"\nint main() {\n    struct Point p;\n    struct Rectangle r;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Point", "types.h", "main.c"),
        "expected reference to Point struct"
    );
    assert!(
        has_reference(&rows, "Rectangle", "types.h", "main.c"),
        "expected reference to Rectangle struct"
    );
}

#[test]
fn finds_c_enum_definitions() {
    let files = vec![
        (
            PathBuf::from("enums.h"),
            "enum Color {\n    RED,\n    GREEN,\n    BLUE\n};\n\nenum Status {\n    PENDING,\n    ACTIVE,\n    DONE\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "#include \"enums.h\"\nint main() {\n    enum Color c = RED;\n    enum Status s = PENDING;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Color", "enums.h", "main.c"),
        "expected reference to Color enum"
    );
    assert!(
        has_reference(&rows, "Status", "enums.h", "main.c"),
        "expected reference to Status enum"
    );
}

#[test]
fn finds_c_typedef_definitions() {
    let files = vec![
        (
            PathBuf::from("types.h"),
            "typedef struct {\n    int x;\n    int y;\n} Point;\n\ntypedef int (*Callback)(int);\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "#include \"types.h\"\nint main() {\n    Point p;\n    Callback cb;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Point", "types.h", "main.c"),
        "expected reference to Point typedef"
    );
    assert!(
        has_reference(&rows, "Callback", "types.h", "main.c"),
        "expected reference to Callback typedef"
    );
}

#[test]
fn finds_c_union_definitions() {
    let files = vec![
        (
            PathBuf::from("types.h"),
            "union Value {\n    int i;\n    float f;\n    char c;\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "#include \"types.h\"\nint main() {\n    union Value v;\n    v.i = 42;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Value", "types.h", "main.c"),
        "expected reference to Value union"
    );
}

#[test]
fn finds_c_global_variable_definitions() {
    let files = vec![
        (
            PathBuf::from("globals.c"),
            "int counter = 0;\nchar* name = \"test\";\nconst int MAX_SIZE = 100;\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "extern int counter;\nextern const int MAX_SIZE;\nint main() {\n    counter = MAX_SIZE;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "counter", "globals.c", "main.c"),
        "expected reference to counter global variable"
    );
    assert!(
        has_reference(&rows, "MAX_SIZE", "globals.c", "main.c"),
        "expected reference to MAX_SIZE global constant"
    );
}

#[test]
fn finds_c_multiple_declarators() {
    let files = vec![
        (
            PathBuf::from("vars.c"),
            "int width, height, depth;\n".to_string(),
        ),
        (
            PathBuf::from("main.c"),
            "extern int width, height, depth;\nint main() {\n    int vol = width * height * depth;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "width", "vars.c", "main.c"),
        "expected reference to width"
    );
    assert!(
        has_reference(&rows, "height", "vars.c", "main.c"),
        "expected reference to height"
    );
    assert!(
        has_reference(&rows, "depth", "vars.c", "main.c"),
        "expected reference to depth"
    );
}

// C++ Tests

#[test]
fn finds_cpp_cross_file_references() {
    let files = vec![
        read_fixture("src/languages/cpp/fixtures/main.cpp"),
        read_fixture("src/languages/cpp/fixtures/types.hpp"),
    ];

    let rows = cruxlines_from_inputs(files, None);

    // Classes
    assert!(
        has_reference(
            &rows,
            "Point",
            "src/languages/cpp/fixtures/types.hpp",
            "src/languages/cpp/fixtures/main.cpp"
        ),
        "expected reference to Point class from main.cpp"
    );
    assert!(
        has_reference(
            &rows,
            "Rectangle",
            "src/languages/cpp/fixtures/types.hpp",
            "src/languages/cpp/fixtures/main.cpp"
        ),
        "expected reference to Rectangle class from main.cpp"
    );
}

#[test]
fn finds_cpp_class_definitions() {
    let files = vec![
        (
            PathBuf::from("models.hpp"),
            "class User {\npublic:\n    std::string name;\n};\n\nclass Order {\npublic:\n    int id;\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "#include \"models.hpp\"\nint main() {\n    User u;\n    Order o;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "User", "models.hpp", "main.cpp"),
        "expected reference to User class"
    );
    assert!(
        has_reference(&rows, "Order", "models.hpp", "main.cpp"),
        "expected reference to Order class"
    );
}

#[test]
fn finds_cpp_struct_definitions() {
    let files = vec![
        (
            PathBuf::from("types.hpp"),
            "struct Point {\n    int x;\n    int y;\n};\n\nstruct Size {\n    int width;\n    int height;\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "#include \"types.hpp\"\nint main() {\n    Point p{0, 0};\n    Size s{10, 20};\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Point", "types.hpp", "main.cpp"),
        "expected reference to Point struct"
    );
    assert!(
        has_reference(&rows, "Size", "types.hpp", "main.cpp"),
        "expected reference to Size struct"
    );
}

#[test]
fn finds_cpp_enum_class_definitions() {
    let files = vec![
        (
            PathBuf::from("enums.hpp"),
            "enum class Color {\n    Red,\n    Green,\n    Blue\n};\n\nenum class Status {\n    Pending,\n    Active,\n    Done\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "#include \"enums.hpp\"\nint main() {\n    Color c = Color::Red;\n    Status s = Status::Active;\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Color", "enums.hpp", "main.cpp"),
        "expected reference to Color enum class"
    );
    assert!(
        has_reference(&rows, "Status", "enums.hpp", "main.cpp"),
        "expected reference to Status enum class"
    );
}

#[test]
fn finds_cpp_functions_in_namespace() {
    let files = vec![
        (
            PathBuf::from("math.cpp"),
            "namespace math {\n\nint add(int a, int b) {\n    return a + b;\n}\n\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "namespace math { int add(int, int); }\nint main() {\n    int x = add(1, 2);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "add", "math.cpp", "main.cpp"),
        "expected reference to add function in namespace"
    );
}

#[test]
fn finds_cpp_function_definitions() {
    let files = vec![
        (
            PathBuf::from("utils.cpp"),
            "int add(int a, int b) {\n    return a + b;\n}\n\nint multiply(int a, int b) {\n    return a * b;\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "#include \"utils.hpp\"\nint main() {\n    int x = add(1, 2);\n    int y = multiply(3, 4);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "add", "utils.cpp", "main.cpp"),
        "expected reference to add function"
    );
    assert!(
        has_reference(&rows, "multiply", "utils.cpp", "main.cpp"),
        "expected reference to multiply function"
    );
}

#[test]
fn finds_c_and_cpp_cross_language_references() {
    // C and C++ should share the same ecosystem
    let files = vec![
        (
            PathBuf::from("math.c"),
            "int add(int a, int b) {\n    return a + b;\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "extern \"C\" int add(int, int);\nint main() {\n    int x = add(1, 2);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "add", "math.c", "main.cpp"),
        "expected C function referenced from C++ file"
    );
}

#[test]
fn finds_cpp_inheritance_references() {
    let files = vec![
        (
            PathBuf::from("base.hpp"),
            "class Animal {\npublic:\n    virtual void speak() = 0;\n};\n\nclass Mammal : public Animal {\npublic:\n    void breathe() {}\n};\n".to_string(),
        ),
        (
            PathBuf::from("derived.cpp"),
            "#include \"base.hpp\"\nclass Dog : public Mammal {\npublic:\n    void speak() override {}\n};\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Mammal", "base.hpp", "derived.cpp"),
        "expected reference to Mammal base class"
    );
}

#[test]
fn finds_cpp_nested_namespace_definitions() {
    let files = vec![
        (
            PathBuf::from("utils.cpp"),
            "namespace company {\nnamespace project {\nnamespace utils {\n\nint helper(int x) {\n    return x * 2;\n}\n\n}\n}\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "namespace company { namespace project { namespace utils { int helper(int); } } }\nint main() {\n    int x = helper(5);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "helper", "utils.cpp", "main.cpp"),
        "expected reference to helper in nested namespace"
    );
}

#[test]
fn finds_cpp_template_class_definitions() {
    let files = vec![
        (
            PathBuf::from("container.hpp"),
            "template<typename T>\nclass Container {\npublic:\n    T value;\n    Container(T v) : value(v) {}\n};\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "#include \"container.hpp\"\nint main() {\n    Container<int> c(42);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "Container", "container.hpp", "main.cpp"),
        "expected reference to Container template class"
    );
}

#[test]
fn finds_cpp_template_function_definitions() {
    let files = vec![
        (
            PathBuf::from("algorithms.hpp"),
            "template<typename T>\nT maximum(T a, T b) {\n    return (a > b) ? a : b;\n}\n".to_string(),
        ),
        (
            PathBuf::from("main.cpp"),
            "#include \"algorithms.hpp\"\nint main() {\n    int x = maximum(3, 5);\n    return 0;\n}\n".to_string(),
        ),
    ];

    let rows = cruxlines_from_inputs(files, None);

    assert!(
        has_reference(&rows, "maximum", "algorithms.hpp", "main.cpp"),
        "expected reference to maximum template function"
    );
}
