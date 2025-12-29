use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use clap::{Parser, ValueEnum};

use cruxlines::{cruxlines, CruxlinesError, Ecosystem, OutputRow};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'e', long = "ecosystem", value_enum)]
    ecosystems: Vec<EcosystemArg>,
    #[arg(short = 'm', long = "metadata")]
    metadata: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum EcosystemArg {
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

    let mut line_cache: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for row in &output_rows {
        print_row(&row, &repo_root, &mut line_cache, cli.metadata);
    }

}

fn print_row(
    row: &OutputRow,
    repo_root: &std::path::Path,
    line_cache: &mut HashMap<PathBuf, Vec<String>>,
    include_metadata: bool,
) {
    let line_text = line_text(&row.definition.path, row.definition.line, line_cache);
    if include_metadata {
        println!(
            "{}:{}:{}: rank={:.6} local={:.6} file={:.6} name={} | {}",
            display_path(&row.definition.path, repo_root),
            row.definition.line,
            row.definition.column,
            row.rank,
            row.local_score,
            row.file_rank,
            row.definition.name,
            line_text
        );
    } else {
        println!(
            "{}:{}:{}: {}",
            display_path(&row.definition.path, repo_root),
            row.definition.line,
            row.definition.column,
            line_text
        );
    }
}

fn display_path(path: &std::path::Path, repo_root: &std::path::Path) -> String {
    match path.strip_prefix(repo_root) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn line_text(
    path: &std::path::Path,
    line: usize,
    cache: &mut HashMap<PathBuf, Vec<String>>,
) -> String {
    let lines = cache.entry(path.to_path_buf()).or_insert_with(|| {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        contents.lines().map(|line| line.to_string()).collect()
    });
    lines
        .get(line.saturating_sub(1))
        .map(|line| line.trim_end().to_string())
        .unwrap_or_default()
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
        ecosystems.insert(Ecosystem::Python);
        ecosystems.insert(Ecosystem::JavaScript);
        ecosystems.insert(Ecosystem::Rust);
        return ecosystems;
    }
    for value in values {
        let ecosystem = match value {
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
