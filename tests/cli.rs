use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use predicates::Predicate;
use predicates::str::contains;

fn run_cli_output() -> String {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["--ecosystem", "python"])
        .current_dir(repo_root());
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("utf8 output")
}

#[test]
fn cli_outputs_reference_edges_for_python_files() {
    let output = run_cli_output();
    assert!(
        contains("\tadd\t")
            .and(contains("src/languages/python/fixtures/utils.py"))
            .and(contains("src/languages/python/fixtures/main.py"))
            .eval(&output),
        "output did not include expected edge: {output}"
    );
}

#[test]
fn library_cruxlines_scans_repo_root() {
    let dir = temp_dir_path("cruxlines-lib-root");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    git_init(&dir);
    std::fs::write(dir.join("main.py"), "def add():\n    return 1\n\nadd()\n")
        .expect("write main");
    git_commit(&dir, "init", "2001-01-01T00:00:00Z");

    let ecosystems = std::collections::HashSet::from([cruxlines::Ecosystem::Python]);
    let rows = cruxlines::cruxlines(&dir, &ecosystems).expect("cruxlines");
    assert!(
        rows.iter().any(|row| row.definition.name == "add"),
        "expected add definition from repo scan"
    );

    let _ = std::fs::remove_dir_all(&dir);
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
fn cli_hides_references_without_flag() {
    let output = run_cli_output();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let parts: Vec<_> = line.split('\t').collect();
        assert_eq!(
            parts.len(),
            5,
            "expected 5 columns, got {parts:?}"
        );
    }
}

#[test]
fn cli_filters_by_ecosystem() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["--ecosystem", "python"])
        .current_dir(repo_root());
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("src/languages/python/fixtures"),
        "expected python fixtures in output, got: {output}"
    );
    assert!(
        !output.contains("src/languages/javascript/fixtures"),
        "expected javascript fixtures to be filtered out, got: {output}"
    );
}

#[test]
fn cli_supports_ecosystem_short_flag() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["-e", "python"])
        .current_dir(repo_root());
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("src/languages/python/fixtures"),
        "expected python fixtures in output, got: {output}"
    );
    assert!(
        !output.contains("src/languages/javascript/fixtures"),
        "expected javascript fixtures to be filtered out, got: {output}"
    );
}

#[test]
fn cli_accepts_ecosystem_aliases() {
    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["--ecosystem", "py"])
        .current_dir(repo_root());
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("src/languages/python/fixtures"),
        "expected python fixtures in output, got: {output}"
    );
    assert!(
        !output.contains("src/languages/javascript/fixtures"),
        "expected javascript fixtures to be filtered out, got: {output}"
    );
}

#[test]
fn cli_outputs_paths_relative_to_repo_root() {
    let dir = temp_dir_path("cruxlines-relpath");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    git_init(&dir);
    std::fs::write(dir.join("main.py"), "def add():\n    return 1\n\nadd()\n")
        .expect("write main");
    git_commit(&dir, "init", "2001-01-01T00:00:00Z");

    let subdir = dir.join("sub");
    std::fs::create_dir_all(&subdir).expect("create subdir");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["--ecosystem", "py"]).current_dir(&subdir);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("\tmain.py:1:5"),
        "expected output paths relative to repo root, got: {output}"
    );
    assert!(
        !output.contains(&dir.display().to_string()),
        "expected output paths to avoid absolute repo root, got: {output}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_skips_unknown_extension_inputs() {
    let dir = temp_dir_path("cruxlines-ignore-ext");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::create_dir_all(dir.join(".git")).expect("create git dir");
    std::fs::write(
        dir.join("main.py"),
        "def add(a, b):\n    return a + b\n\nadd(1, 2)\n",
    )
        .expect("write main");
    std::fs::write(dir.join("ignore.txt"), "not source").expect("write temp file");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.current_dir(&dir);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("main.py"),
        "expected output to include main.py, got: {output}"
    );
    assert!(
        !output.contains("ignore.txt"),
        "expected ignore.txt to be skipped, got: {output}"
    );

    let _ = std::fs::remove_file(dir.join("ignore.txt"));
    let _ = std::fs::remove_file(dir.join("main.py"));
    let _ = std::fs::remove_dir(dir.join(".git"));
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn cli_skips_non_utf8_inputs() {
    let dir = temp_dir_path("cruxlines-binary");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::create_dir_all(dir.join(".git")).expect("create git dir");
    std::fs::write(
        dir.join("main.py"),
        "def add(a, b):\n    return a + b\n\nadd(1, 2)\n",
    )
        .expect("write main");
    std::fs::write(dir.join("binary.py"), [0xff, 0xfe, 0xfd]).expect("write temp file");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.current_dir(&dir);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");
    assert!(
        output.contains("main.py"),
        "expected output to include main.py, got: {output}"
    );
    assert!(
        !output.contains("binary.py"),
        "expected binary.py to be skipped, got: {output}"
    );

    let _ = std::fs::remove_file(dir.join("binary.py"));
    let _ = std::fs::remove_file(dir.join("main.py"));
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
fn cli_uses_repo_root_for_frecency() {
    let dir = temp_dir_path("cruxlines-frecency");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    git_init(&dir);

    std::fs::write(dir.join("defs_a.py"), "def alpha():\n    return 1\n")
        .expect("write defs_a");
    std::fs::write(dir.join("defs_b.py"), "def beta():\n    return 1\n")
        .expect("write defs_b");
    let alpha_calls = "    alpha()\n".repeat(50);
    let beta_calls = "    beta()\n".repeat(50);
    std::fs::write(
        dir.join("use_alpha.py"),
        format!("from defs_a import alpha\nfrom main import anchor\n\n\
def helper_alpha():\n{alpha_calls}    anchor()\n"),
    )
    .expect("write use_alpha");
    std::fs::write(
        dir.join("use_beta.py"),
        format!("from defs_b import beta\nfrom main import anchor\n\n\
def helper_beta():\n{beta_calls}    anchor()\n"),
    )
    .expect("write use_beta");
    std::fs::write(
        dir.join("main.py"),
        "from use_alpha import helper_alpha\nfrom use_beta import helper_beta\n\n\
def anchor():\n    return None\n\n\
helper_alpha()\nhelper_beta()\n",
    )
    .expect("write main");

    git_commit(&dir, "initial", "2001-01-01T00:00:00Z");

    for day in 2..=11 {
        std::fs::write(
            dir.join("use_alpha.py"),
            format!(
                "from defs_a import alpha\nfrom main import anchor\n\n\
def helper_alpha():\n{alpha_calls}    anchor()\n\n# touch {day}\n"
            ),
        )
        .expect("update use_alpha");
        let date = format!("2001-01-{day:02}T00:00:00Z");
        git_commit(&dir, "touch alpha", &date);
    }

    let subdir = dir.join("sub");
    std::fs::create_dir_all(&subdir).expect("create subdir");

    let mut cmd = cargo_bin_cmd!("cruxlines");
    cmd.args(["--ecosystem", "py"]).current_dir(&subdir);
    let output = cmd.assert().success().get_output().stdout.clone();
    let output = String::from_utf8(output).expect("utf8 output");

    let alpha_score = local_score_for_symbol(&output, "alpha").expect("alpha score");
    let beta_score = local_score_for_symbol(&output, "beta").expect("beta score");
    assert!(
        alpha_score > beta_score,
        "expected alpha to score higher due to frecency, got alpha={alpha_score} beta={beta_score}\n{output}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn local_score_for_symbol(output: &str, symbol: &str) -> Option<f64> {
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split('\t');
        let _score_str = parts.next()?;
        let local_str = parts.next()?;
        let _file_rank = parts.next()?;
        let name = parts.next()?;
        if name == symbol {
            return local_str.parse().ok();
        }
    }
    None
}

fn git_init(dir: &std::path::Path) {
    let status = git_command(dir)
        .arg("init")
        .status()
        .expect("git init");
    assert!(status.success(), "git init failed");
    let status = git_command(dir)
        .args(["config", "user.name", "Test User"])
        .status()
        .expect("git config user.name");
    assert!(status.success(), "git config user.name failed");
    let status = git_command(dir)
        .args(["config", "user.email", "test@example.com"])
        .status()
        .expect("git config user.email");
    assert!(status.success(), "git config user.email failed");
}

fn git_commit(dir: &std::path::Path, message: &str, date: &str) {
    let status = git_command(dir).arg("add").arg(".").status().expect("git add");
    assert!(status.success(), "git add failed");
    let status = git_command(dir)
        .args(["-c", "commit.gpgsign=false", "commit", "-m", message])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("git commit");
    assert!(status.success(), "git commit failed");
}

fn git_command(dir: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C").arg(dir);
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd.env("GIT_CONFIG_SYSTEM", "/dev/null");
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd
}
