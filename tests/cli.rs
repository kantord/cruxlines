use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use predicates::Predicate;
use predicates::str::contains;

fn run_cli_output() -> String {
    let mut cmd = cargo_bin_cmd!("cruxlines");
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

#[test]
fn cli_outputs_scores_in_descending_order() {
    let output = run_cli_output();
    let mut prev = f64::INFINITY;
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let Some(score_str) = line.split('\t').next() else {
            continue;
        };
        let score: f64 = score_str.parse().expect("score is f64");
        assert!(
            score <= prev + 1e-12,
            "scores are not in descending order: {score} after {prev}"
        );
        prev = score;
    }
}

#[test]
fn cli_groups_references_per_definition() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args([
        "-u",
        "fixtures/python/main.py",
        "fixtures/python/utils.py",
        "fixtures/python/models.py",
    ]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    let mut add_lines = 0;
    let mut add_line = String::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split('\t');
        let _score = parts.next().unwrap_or_default();
        let symbol = parts.next().unwrap_or_default();
        if symbol == "add" {
            add_lines += 1;
            add_line = line.to_string();
        }
    }
    assert_eq!(add_lines, 1, "expected one line for add, got {add_lines}");
    let refs: Vec<_> = add_line.split('\t').skip(3).collect();
    assert!(
        refs.len() >= 2,
        "expected at least two references for add, got {refs:?}"
    );
}

#[test]
fn cli_hides_references_without_flag() {
    let output = run_cli_output();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let parts: Vec<_> = line.split('\t').collect();
        assert_eq!(
            parts.len(),
            3,
            "expected 3 columns without references flag, got {parts:?}"
        );
    }
}

#[test]
fn cli_shows_references_with_flag() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args([
        "-u",
        "fixtures/python/main.py",
        "fixtures/python/utils.py",
        "fixtures/python/models.py",
    ]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    let mut has_refs = false;
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() > 3 {
            has_refs = true;
            break;
        }
    }
    assert!(has_refs, "expected at least one line with references");
}
