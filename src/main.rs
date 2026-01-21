use std::path::PathBuf;
use std::process;

use clap::{Parser, ValueEnum};

use cruxlines::{CruxlinesError, Ecosystem, OutputRow, cruxlines};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'e', long = "ecosystem", value_enum)]
    ecosystems: Vec<EcosystemArg>,
    #[arg(short = 'm', long = "metadata")]
    metadata: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum EcosystemArg {
    #[value(name = "java", alias = "jvm")]
    Java,
    #[value(name = "python", alias = "py")]
    Python,
    #[value(name = "javascript", alias = "js", alias = "ts", alias = "tsx")]
    JavaScript,
    #[value(name = "rust", alias = "rs")]
    Rust,
}

fn main() {
    let cli = Cli::parse();
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            eprintln!("cruxlines: failed to read current dir: {err}");
            process::exit(1);
        }
    };
    let Some(repo_root) = find_repo_root(&cwd) else {
        eprintln!("cruxlines: current dir is not inside a git repository");
        process::exit(1);
    };
    let ecosystems = selected_ecosystems(&cli.ecosystems);

    let output_rows = match cruxlines(&repo_root, &ecosystems) {
        Ok(rows) => rows,
        Err(err) => {
            report_error(err);
            process::exit(1);
        }
    };

    // Test-only hook to coordinate snapshot timing in integration tests.
    if let Ok(ready_path) = std::env::var("CRUXLINES_TEST_READY_FILE") {
        let _ = std::fs::write(ready_path, "ready\n");
    }
    if let Ok(pause_ms) = std::env::var("CRUXLINES_TEST_PAUSE_MS")
        && let Ok(pause_ms) = pause_ms.parse::<u64>()
    {
        std::thread::sleep(std::time::Duration::from_millis(pause_ms));
    }

    for row in &output_rows {
        print_row(row, &repo_root, cli.metadata);
    }
}

fn print_row(row: &OutputRow, repo_root: &std::path::Path, include_metadata: bool) {
    let line_text = row.definition_line.as_str();
    if include_metadata {
        println!(
            "{}:{}:{}: rank={:.6} local={:.6} file={:.6} name={} | {}",
            display_path(row.definition.path_str(), repo_root),
            row.definition.line,
            row.definition.column,
            row.rank,
            row.local_score,
            row.file_rank,
            row.definition.name_str(),
            line_text
        );
    } else {
        println!(
            "{}:{}:{}: {}",
            display_path(row.definition.path_str(), repo_root),
            row.definition.line,
            row.definition.column,
            line_text
        );
    }
}

fn display_path(path: &str, repo_root: &std::path::Path) -> String {
    let path = std::path::Path::new(path);
    match path.strip_prefix(repo_root) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn report_error(err: CruxlinesError) {
    match err {
        CruxlinesError::ReadFile { path, source } => {
            eprintln!("cruxlines: failed to read {}: {source}", path.display());
        }
    }
}

fn selected_ecosystems(values: &[EcosystemArg]) -> std::collections::HashSet<Ecosystem> {
    let mut ecosystems = std::collections::HashSet::new();
    if values.is_empty() {
        ecosystems.insert(Ecosystem::Java);
        ecosystems.insert(Ecosystem::Python);
        ecosystems.insert(Ecosystem::JavaScript);
        ecosystems.insert(Ecosystem::Rust);
        return ecosystems;
    }
    for value in values {
        let ecosystem = match value {
            EcosystemArg::Java => Ecosystem::Java,
            EcosystemArg::Python => Ecosystem::Python,
            EcosystemArg::JavaScript => Ecosystem::JavaScript,
            EcosystemArg::Rust => Ecosystem::Rust,
        };
        ecosystems.insert(ecosystem);
    }
    ecosystems
}

fn find_repo_root(start: &std::path::Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        if ancestor.join(".git").is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}
