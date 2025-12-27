use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::Parser;

mod cli_io;

use cruxlines::{cruxlines, cruxlines_profiled, OutputRow, ProfileStats};
use cli_io::{gather_inputs, CliIoError};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'u', long = "references")]
    references: bool,
    #[arg(long = "profile")]
    profile: bool,
    #[arg(required = true)]
    files: Vec<PathBuf>,
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
    let start_inputs = Instant::now();
    let inputs = match gather_inputs(cli.files) {
        Ok(inputs) => inputs,
        Err(err) => {
            report_error(err);
            process::exit(1);
        }
    };
    let inputs_duration = start_inputs.elapsed();
    let start_analysis = Instant::now();
    let (output_rows, profile_stats) = if cli.profile {
        let (rows, stats) = cruxlines_profiled(inputs);
        (rows, Some(stats))
    } else {
        (cruxlines(inputs), None)
    };
    let analysis_duration = start_analysis.elapsed();

    for row in &output_rows {
        print_row(&row, cli.references, &cwd);
    }

    if let Some(stats) = profile_stats {
        report_profile(inputs_duration.as_millis(), analysis_duration.as_millis(), &stats);
    }
}

fn print_row(row: &OutputRow, include_references: bool, cwd: &std::path::Path) {
    if include_references {
        println!(
            "{:.6}\t{:.6}\t{:.6}\t{}\t{}:{}:{}{}",
            row.rank,
            row.local_score,
            row.file_rank,
            row.definition.name,
            display_path(&row.definition.path, cwd),
            row.definition.line,
            row.definition.column,
            format_usage_list(&row.references, cwd)
        );
    } else {
        println!(
            "{:.6}\t{:.6}\t{:.6}\t{}\t{}:{}:{}",
            row.rank,
            row.local_score,
            row.file_rank,
            row.definition.name,
            display_path(&row.definition.path, cwd),
            row.definition.line,
            row.definition.column
        );
    }
}

fn format_usage_list(usages: &[cruxlines::Location], cwd: &std::path::Path) -> String {
    let mut out = String::new();
    for usage in usages {
        out.push('\t');
        out.push_str(&format!(
            "{}:{}:{}",
            display_path(&usage.path, cwd),
            usage.line,
            usage.column
        ));
    }
    out
}

fn display_path(path: &std::path::Path, cwd: &std::path::Path) -> String {
    match path.strip_prefix(cwd) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => path.display().to_string(),
    }
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

fn report_profile(inputs_ms: u128, analysis_ms: u128, stats: &ProfileStats) {
    eprintln!(
        "profile: inputs={}ms analysis={}ms frecency={}ms parse={}ms file_rank={}ms score={}ms definitions={} references={}",
        inputs_ms,
        analysis_ms,
        stats.frecency_ms,
        stats.parse_ms,
        stats.file_rank_ms,
        stats.score_ms,
        stats.definitions,
        stats.references
    );
}
