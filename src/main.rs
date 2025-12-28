use std::path::PathBuf;
use std::process;
use clap::{Parser, ValueEnum};

mod cli_io;

use cruxlines::{cruxlines_with_repo_root, Ecosystem, OutputRow};
use cli_io::{gather_inputs, CliIoError};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'u', long = "references")]
    references: bool,
    #[arg(short = 'e', long = "ecosystem", value_enum)]
    ecosystems: Vec<EcosystemArg>,
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
    let inputs = match gather_inputs(&repo_root, &ecosystems) {
        Ok(inputs) => inputs,
        Err(err) => {
            report_error(err);
            process::exit(1);
        }
    };
    let output_rows = cruxlines_with_repo_root(Some(repo_root.clone()), inputs);

    for row in &output_rows {
        print_row(&row, cli.references, &repo_root);
    }

}

fn print_row(row: &OutputRow, include_references: bool, repo_root: &std::path::Path) {
    if include_references {
        println!(
            "{:.6}\t{:.6}\t{:.6}\t{}\t{}:{}:{}{}",
            row.rank,
            row.local_score,
            row.file_rank,
            row.definition.name,
            display_path(&row.definition.path, repo_root),
            row.definition.line,
            row.definition.column,
            format_usage_list(&row.references, repo_root)
        );
    } else {
        println!(
            "{:.6}\t{:.6}\t{:.6}\t{}\t{}:{}:{}",
            row.rank,
            row.local_score,
            row.file_rank,
            row.definition.name,
            display_path(&row.definition.path, repo_root),
            row.definition.line,
            row.definition.column
        );
    }
}

fn format_usage_list(usages: &[cruxlines::Location], repo_root: &std::path::Path) -> String {
    let mut out = String::new();
    for usage in usages {
        out.push('\t');
        out.push_str(&format!(
            "{}:{}:{}",
            display_path(&usage.path, repo_root),
            usage.line,
            usage.column
        ));
    }
    out
}

fn display_path(path: &std::path::Path, repo_root: &std::path::Path) -> String {
    match path.strip_prefix(repo_root) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn report_error(err: CliIoError) {
    match err {
        CliIoError::ReadFile { path, source } => {
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
