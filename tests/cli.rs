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
        let _local = parts.next().unwrap_or_default();
        let _file_rank = parts.next().unwrap_or_default();
        let symbol = parts.next().unwrap_or_default();
        if symbol == "add" {
            add_lines += 1;
            add_line = line.to_string();
        }
    }
    assert_eq!(add_lines, 1, "expected one line for add, got {add_lines}");
    let refs: Vec<_> = add_line.split('\t').skip(5).collect();
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
            5,
            "expected 5 columns without references flag, got {parts:?}"
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
        if parts.len() > 5 {
            has_refs = true;
            break;
        }
    }
    assert!(has_refs, "expected at least one line with references");
}

#[test]
fn cli_skips_directory_inputs() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["fixtures", "fixtures/python/main.py"]);
    cmd.assert().success();
}

#[test]
fn cli_skips_unknown_extension_inputs() {
    let tmp_path = temp_file_path("cruxlines-ignore.txt");
    std::fs::write(&tmp_path, "not source").expect("write temp file");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args([tmp_path.to_str().unwrap(), "fixtures/python/main.py"]);
    cmd.assert().success();

    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn cli_skips_non_utf8_inputs() {
    let tmp_path = temp_file_path("cruxlines-binary.py");
    std::fs::write(&tmp_path, [0xff, 0xfe, 0xfd]).expect("write temp file");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args([tmp_path.to_str().unwrap(), "fixtures/python/main.py"]);
    cmd.assert().success();

    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn cli_respects_gitignore_like_ripgrep() {
    let dir = temp_dir_path("cruxlines-ignore");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::create_dir_all(dir.join(".git")).expect("create git dir");
    std::fs::write(dir.join(".gitignore"), "ignored.py\n").expect("write gitignore");
    std::fs::write(
        dir.join("utils.py"),
        "def add(a, b):\n    return a + b\n",
    )
    .expect("write utils");
    std::fs::write(
        dir.join("main.py"),
        "from utils import add\nfrom ignored import ignored\n\nprint(add(1, 2))\nprint(ignored())\n",
    )
    .expect("write main");
    std::fs::write(
        dir.join("ignored.py"),
        "def ignored():\n    return 0\n",
    )
    .expect("write ignored");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.current_dir(&dir);
    cmd.args(["-u", "main.py", "utils.py", "ignored.py"]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("utils.py"),
        "expected output to include utils.py, got: {output}"
    );
    assert!(
        output.contains("ignored.py"),
        "expected ignored.py to be included when explicitly passed, got: {output}"
    );

    let _ = std::fs::remove_file(dir.join("ignored.py"));
    let _ = std::fs::remove_file(dir.join("main.py"));
    let _ = std::fs::remove_file(dir.join("utils.py"));
    let _ = std::fs::remove_file(dir.join(".gitignore"));
    let _ = std::fs::remove_dir(dir.join(".git"));
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn cli_skips_gitignored_when_scanning_directory() {
    let dir = temp_dir_path("cruxlines-ignore-dir");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::create_dir_all(dir.join(".git")).expect("create git dir");
    std::fs::write(dir.join(".gitignore"), "ignored.py\n").expect("write gitignore");
    std::fs::write(
        dir.join("utils.py"),
        "def add(a, b):\n    return a + b\n",
    )
    .expect("write utils");
    std::fs::write(
        dir.join("main.py"),
        "from utils import add\nfrom ignored import ignored\n\nprint(add(1, 2))\nprint(ignored())\n",
    )
    .expect("write main");
    std::fs::write(
        dir.join("ignored.py"),
        "def ignored():\n    return 0\n",
    )
    .expect("write ignored");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.current_dir(&dir);
    cmd.args(["-u", "."]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("utils.py"),
        "expected output to include utils.py, got: {output}"
    );
    assert!(
        !output.contains("ignored.py"),
        "expected ignored.py to be skipped when scanning dir, got: {output}"
    );

    let _ = std::fs::remove_file(dir.join("ignored.py"));
    let _ = std::fs::remove_file(dir.join("main.py"));
    let _ = std::fs::remove_file(dir.join("utils.py"));
    let _ = std::fs::remove_file(dir.join(".gitignore"));
    let _ = std::fs::remove_dir(dir.join(".git"));
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn cli_profile_flag_outputs_stats() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["--profile", "fixtures/python/main.py"]);
    let output = cmd.assert().success().get_output().stderr.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("profile:"),
        "expected profile output, got: {output}"
    );
}

fn temp_file_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    path.push(format!("{name}-{nanos}"));
    path
}

fn temp_dir_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    path.push(format!("{name}-{nanos}"));
    path
}
