use clap::Parser;
use std::path::PathBuf;
use std::process;

use cargo_health::{parse_lockfile, query_crate_info, score_dependency, print_report, HealthCategory};

/// Cargo subcommand arguments wrapper.
/// When invoked as `cargo health`, cargo passes "health" as the first argument.
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    /// Check the maintenance health of your dependencies
    Health(HealthArgs),
}

#[derive(Parser, Debug)]
#[command(version, about = "Scan your dependency tree and report maintenance health")]
struct HealthArgs {
    /// Path to the Cargo.lock file
    #[arg(short, long, default_value = "Cargo.lock")]
    lockfile: PathBuf,

    /// Only show dependencies with WARNING or CRITICAL status
    #[arg(short, long)]
    warn_only: bool,
}

fn main() {
    let CargoCli::Health(args) = CargoCli::parse();

    let lockfile_content = match std::fs::read_to_string(&args.lockfile) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {}: {}", args.lockfile.display(), e);
            process::exit(1);
        }
    };

    let dependencies = match parse_lockfile(&lockfile_content) {
        Ok(deps) => deps,
        Err(e) => {
            eprintln!("Error parsing lockfile: {}", e);
            process::exit(1);
        }
    };

    let mut results = Vec::new();

    for dep in &dependencies {
        // Skip path/git dependencies and the root package
        if dep.source.is_none() {
            continue;
        }

        match query_crate_info(&dep.name) {
            Ok(info) => {
                let score = score_dependency(&info);
                results.push((dep.clone(), info, score));
            }
            Err(e) => {
                eprintln!("Warning: Could not fetch info for {}: {}", dep.name, e);
            }
        }
    }

    print_report(&results, args.warn_only);

    // Exit with non-zero if any critical dependencies found
    let has_critical = results
        .iter()
        .any(|(_, _, score)| score.category == HealthCategory::Critical);
    if has_critical {
        process::exit(1);
    }
}
