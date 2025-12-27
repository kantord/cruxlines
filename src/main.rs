use std::path::PathBuf;
use std::process;

use clap::Parser;

mod cli_io;

use cruxlines::{cruxlines, OutputRow};
use cli_io::{gather_inputs, CliIoError};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'u', long = "references")]
    references: bool,
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let inputs = match gather_inputs(cli.files) {
        Ok(inputs) => inputs,
        Err(err) => {
            report_error(err);
            process::exit(1);
        }
    };
    let output_rows = cruxlines(inputs);

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

fn format_usage_list(usages: &[cruxlines::Location]) -> String {
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

fn report_error(err: CliIoError) {
    match err {
        CliIoError::CurrentDir(source) => {
            eprintln!("cruxlines: failed to read current dir: {source}");
        }
        CliIoError::ReadFile { path, source } => {
            eprintln!("cruxlines: failed to read {}: {source}", path.display());
        }
    }
}
