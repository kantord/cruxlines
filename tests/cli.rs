use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn cli_outputs_reference_edges_for_python_files() {
    let mut cmd = Command::cargo_bin("cruxlines").expect("binary exists");
    cmd.args([
        "fixtures/python/main.py",
        "fixtures/python/utils.py",
        "fixtures/python/models.py",
    ]);

    cmd.assert()
        .success()
        .stdout(
            contains("\tadd\t")
                .and(contains("fixtures/python/utils.py"))
                .and(contains("fixtures/python/main.py")),
        );
}
