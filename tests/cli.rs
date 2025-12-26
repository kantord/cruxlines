use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::Predicate;
use predicates::str::contains;

fn run_cli_output() -> String {
    let mut cmd = Command::cargo_bin("cruxlines").expect("binary exists");
    cmd.args([
        "fixtures/python/main.py",
        "fixtures/python/utils.py",
        "fixtures/python/models.py",
    ]);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("utf8 output")
}

#[test]
fn cli_outputs_reference_edges_for_python_files() {
    let output = run_cli_output();
    assert!(
        contains("\tadd\t")
            .and(contains("fixtures/python/utils.py"))
            .and(contains("fixtures/python/main.py"))
            .eval(&output),
        "output did not include expected edge: {output}"
    );
}

#[test]
fn cli_outputs_non_uniform_pagerank_scores() {
    let output = run_cli_output();
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let Some(score_str) = line.split('\t').next() else {
            continue;
        };
        let score: f64 = score_str.parse().expect("score is f64");
        if score < min {
            min = score;
        }
        if score > max {
            max = score;
        }
    }
    assert!(
        (max - min) > 1e-6,
        "expected non-uniform pagerank scores, got min={min} max={max}"
    );
}
