//! Terminal report output with colored formatting.

use colored::Colorize;

use crate::api::CrateInfo;
use crate::lockfile::Dependency;
use crate::scorer::{HealthCategory, HealthScore};

/// Print a colored health report to the terminal.
///
/// If `warn_only` is true, only dependencies with WARNING or CRITICAL
/// status will be shown.
pub fn print_report(results: &[(Dependency, CrateInfo, HealthScore)], warn_only: bool) {
    println!(
        "\n{}",
        "=== Dependency Health Report ===".bold().underline()
    );
    println!();

    let mut healthy_count = 0u32;
    let mut warning_count = 0u32;
    let mut critical_count = 0u32;

    for (dep, _info, score) in results {
        match score.category {
            HealthCategory::Healthy => healthy_count += 1,
            HealthCategory::Warning => warning_count += 1,
            HealthCategory::Critical => critical_count += 1,
        }

        if warn_only && score.category == HealthCategory::Healthy {
            continue;
        }

        let status = match score.category {
            HealthCategory::Healthy => format!("[{}]", "HEALTHY".green().bold()),
            HealthCategory::Warning => format!("[{}]", "WARNING".yellow().bold()),
            HealthCategory::Critical => format!("[{}]", "CRITICAL".red().bold()),
        };

        let repo_indicator = if score.has_repository {
            "repo".green().to_string()
        } else {
            "no repo".red().to_string()
        };

        println!(
            "  {} {} {} (score: {}, updated: {} days ago, downloads: {}, {})",
            status,
            dep.name.bold(),
            format!("v{}", dep.version).dimmed(),
            score.score,
            score.days_since_update,
            format_downloads(score.downloads),
            repo_indicator,
        );
    }

    println!();
    println!("{}", "--- Summary ---".bold());
    println!(
        "  {} {} | {} {} | {} {}",
        healthy_count.to_string().green().bold(),
        "healthy".green(),
        warning_count.to_string().yellow().bold(),
        "warning".yellow(),
        critical_count.to_string().red().bold(),
        "critical".red(),
    );
    println!(
        "  {} total dependencies scanned",
        results.len().to_string().bold()
    );
    println!();
}

/// Format download count with K/M suffixes for readability.
fn format_downloads(downloads: u64) -> String {
    if downloads >= 1_000_000 {
        format!("{:.1}M", downloads as f64 / 1_000_000.0)
    } else if downloads >= 1_000 {
        format!("{:.1}K", downloads as f64 / 1_000.0)
    } else {
        downloads.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_downloads_small() {
        assert_eq!(format_downloads(42), "42");
        assert_eq!(format_downloads(999), "999");
    }

    #[test]
    fn test_format_downloads_thousands() {
        assert_eq!(format_downloads(1_000), "1.0K");
        assert_eq!(format_downloads(15_500), "15.5K");
    }

    #[test]
    fn test_format_downloads_millions() {
        assert_eq!(format_downloads(1_000_000), "1.0M");
        assert_eq!(format_downloads(2_500_000), "2.5M");
    }
}
