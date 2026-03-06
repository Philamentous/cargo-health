//! cargo-health: Scan your dependency tree and report maintenance health.
//!
//! This library provides the core functionality for parsing Cargo.lock files,
//! querying crates.io for dependency metadata, scoring dependencies on
//! maintenance signals, and generating health reports.

mod lockfile;
mod api;
mod scorer;
mod report;

pub use lockfile::{parse_lockfile, Dependency};
pub use api::{query_crate_info, CrateInfo};
pub use scorer::{score_dependency, HealthScore, HealthCategory};
pub use report::print_report;
