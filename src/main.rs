use std::path::PathBuf;
use std::process;

use clap::Parser;

use cruxlines::analysis::{analyze_paths, AnalyzeError, OutputRow};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'u', long = "references")]
    references: bool,
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let output_rows = match analyze_paths(cli.files) {
        Ok(rows) => rows,
        Err(err) => {
            report_error(err);
            process::exit(1);
        }
    };

    for row in output_rows {
        print_row(&row, cli.references);
    }
}

fn print_row(row: &OutputRow, include_references: bool) {
    if include_references {
        println!(
            "{:.6}\t{}\t{}:{}:{}{}",
            row.rank,
            row.definition.name,
            row.definition.path.display(),
            row.definition.line,
            row.definition.column,
            format_usage_list(&row.references)
        );
    } else {
        println!(
            "{:.6}\t{}\t{}:{}:{}",
            row.rank,
            row.definition.name,
            row.definition.path.display(),
            row.definition.line,
            row.definition.column
        );
    }
}

fn format_usage_list(usages: &[cruxlines::find_references::Location]) -> String {
    let mut out = String::new();
    for usage in usages {
        out.push('\t');
        out.push_str(&format!(
            "{}:{}:{}",
            usage.path.display(),
            usage.line,
            usage.column
        ));
    }
    out
}

fn report_error(err: AnalyzeError) {
    match err {
        AnalyzeError::CurrentDir(source) => {
            eprintln!("cruxlines: failed to read current dir: {source}");
        }
        AnalyzeError::ReadFile { path, source } => {
            eprintln!("cruxlines: failed to read {}: {source}", path.display());
        }
    }
}
