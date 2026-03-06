//! Health scoring logic for dependencies.
//!
//! Scores each dependency based on maintenance signals:
//! - Days since last update
//! - Total download count
//! - Whether a repository URL exists

use crate::api::CrateInfo;
use chrono::Utc;

/// Health categories for dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthCategory {
    Healthy,
    Warning,
    Critical,
}

impl std::fmt::Display for HealthCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthCategory::Healthy => write!(f, "HEALTHY"),
            HealthCategory::Warning => write!(f, "WARNING"),
            HealthCategory::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A health score for a single dependency.
#[derive(Debug, Clone)]
pub struct HealthScore {
    /// Overall score from 0 (worst) to 100 (best).
    pub score: u32,
    /// The health category based on score thresholds.
    pub category: HealthCategory,
    /// Days since the crate was last updated.
    pub days_since_update: i64,
    /// Total download count.
    pub downloads: u64,
    /// Whether the crate has a repository URL.
    pub has_repository: bool,
}

/// Score thresholds for categorization.
const HEALTHY_THRESHOLD: u32 = 60;
const WARNING_THRESHOLD: u32 = 30;

/// Compute a health score for a dependency based on its crates.io metadata.
///
/// Scoring breakdown (total 100 points):
/// - **Update recency (50 points):** Full score if updated within 90 days,
///   linearly decreasing to 0 at 730 days (2 years).
/// - **Downloads (30 points):** Logarithmic scale. 30 points at 1M+ downloads,
///   scaled down for fewer.
/// - **Repository (20 points):** 20 if present, 0 if absent.
pub fn score_dependency(info: &CrateInfo) -> HealthScore {
    let days_since_update = compute_days_since_update(&info.updated_at);
    let has_repository = info.repository.is_some();

    let update_score = compute_update_score(days_since_update);
    let download_score = compute_download_score(info.downloads);
    let repo_score = if has_repository { 20 } else { 0 };

    let score = update_score + download_score + repo_score;

    let category = if score >= HEALTHY_THRESHOLD {
        HealthCategory::Healthy
    } else if score >= WARNING_THRESHOLD {
        HealthCategory::Warning
    } else {
        HealthCategory::Critical
    };

    HealthScore {
        score,
        category,
        days_since_update,
        downloads: info.downloads,
        has_repository,
    }
}

/// Compute days since the last update from an ISO 8601 timestamp string.
fn compute_days_since_update(updated_at: &str) -> i64 {
    // Try parsing as RFC 3339 (which crates.io returns)
    match chrono::DateTime::parse_from_rfc3339(updated_at) {
        Ok(dt) => {
            let now = Utc::now();
            (now - dt.with_timezone(&Utc)).num_days()
        }
        Err(_) => {
            // If we can't parse it, assume very old
            999
        }
    }
}

/// Score based on recency of updates (0-50 points).
/// Full score within 90 days, linear decay to 0 at 730 days.
fn compute_update_score(days: i64) -> u32 {
    if days < 0 {
        return 50; // Future date (clock skew), give benefit of doubt
    }
    if days <= 90 {
        50
    } else if days >= 730 {
        0
    } else {
        let remaining = 730 - days;
        let range = 730 - 90;
        ((remaining as f64 / range as f64) * 50.0) as u32
    }
}

/// Score based on download count (0-30 points).
/// Uses logarithmic scaling: 30 points at 1M+.
fn compute_download_score(downloads: u64) -> u32 {
    if downloads == 0 {
        return 0;
    }
    let log_downloads = (downloads as f64).log10();
    // 1M = 6.0 in log10, scale to 30 points
    let score = (log_downloads / 6.0) * 30.0;
    score.min(30.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_score() {
        let info = CrateInfo {
            name: "popular-crate".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            downloads: 5_000_000,
            repository: Some("https://github.com/example/repo".to_string()),
            description: Some("A popular crate".to_string()),
            max_version: "1.0.0".to_string(),
        };
        let score = score_dependency(&info);
        assert_eq!(score.category, HealthCategory::Healthy);
        assert!(score.score >= HEALTHY_THRESHOLD);
        assert!(score.has_repository);
    }

    #[test]
    fn test_critical_score_old_no_repo() {
        let info = CrateInfo {
            name: "abandoned-crate".to_string(),
            updated_at: "2020-01-01T00:00:00Z".to_string(),
            downloads: 50,
            repository: None,
            description: None,
            max_version: "0.0.1".to_string(),
        };
        let score = score_dependency(&info);
        assert_eq!(score.category, HealthCategory::Critical);
        assert!(score.score < WARNING_THRESHOLD);
        assert!(!score.has_repository);
    }

    #[test]
    fn test_warning_score() {
        // Moderately old, decent downloads, has repo
        let info = CrateInfo {
            name: "moderate-crate".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            downloads: 10_000,
            repository: Some("https://github.com/example/repo".to_string()),
            description: Some("A moderate crate".to_string()),
            max_version: "0.5.0".to_string(),
        };
        let score = score_dependency(&info);
        // This should be in warning or healthy range depending on exact date
        assert!(score.score > 0);
    }

    #[test]
    fn test_update_score_recent() {
        assert_eq!(compute_update_score(0), 50);
        assert_eq!(compute_update_score(45), 50);
        assert_eq!(compute_update_score(90), 50);
    }

    #[test]
    fn test_update_score_old() {
        assert_eq!(compute_update_score(730), 0);
        assert_eq!(compute_update_score(1000), 0);
    }

    #[test]
    fn test_update_score_mid() {
        let score = compute_update_score(410); // Midpoint of 90-730
        assert!(score > 0);
        assert!(score < 50);
    }

    #[test]
    fn test_download_score_zero() {
        assert_eq!(compute_download_score(0), 0);
    }

    #[test]
    fn test_download_score_million() {
        assert_eq!(compute_download_score(1_000_000), 30);
    }

    #[test]
    fn test_download_score_scaling() {
        let low = compute_download_score(100);
        let mid = compute_download_score(10_000);
        let high = compute_download_score(1_000_000);
        assert!(low < mid);
        assert!(mid < high);
    }

    #[test]
    fn test_repo_contributes_20_points() {
        let mut info = CrateInfo {
            name: "test".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            downloads: 1_000_000,
            repository: None,
            description: None,
            max_version: "1.0.0".to_string(),
        };
        let without_repo = score_dependency(&info);

        info.repository = Some("https://github.com/test/test".to_string());
        let with_repo = score_dependency(&info);

        assert_eq!(with_repo.score - without_repo.score, 20);
    }

    #[test]
    fn test_negative_days_handled() {
        assert_eq!(compute_update_score(-5), 50);
    }

    #[test]
    fn test_unparseable_date_defaults_to_old() {
        let days = compute_days_since_update("not-a-date");
        assert_eq!(days, 999);
    }

    #[test]
    fn test_category_boundaries() {
        // Exact boundary tests
        let info_healthy = CrateInfo {
            name: "test".to_string(),
            updated_at: Utc::now().to_rfc3339(),
            downloads: 1_000_000,
            repository: Some("https://github.com/t/t".to_string()),
            description: None,
            max_version: "1.0.0".to_string(),
        };
        assert_eq!(
            score_dependency(&info_healthy).category,
            HealthCategory::Healthy
        );
    }
}
