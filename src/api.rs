//! crates.io API client.
//!
//! Queries the crates.io API for crate metadata including last update time,
//! download counts, and repository URLs.

use serde::Deserialize;

const USER_AGENT: &str = "cargo-health/0.1.0 (https://github.com/Philamentous/cargo-health)";
const CRATES_IO_API: &str = "https://crates.io/api/v1/crates";

/// Metadata about a crate fetched from crates.io.
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub updated_at: String,
    pub downloads: u64,
    pub repository: Option<String>,
    pub description: Option<String>,
    pub max_version: String,
}

/// Raw API response structures for deserialization.
#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    #[serde(rename = "crate")]
    krate: CrateData,
}

#[derive(Debug, Deserialize)]
struct CrateData {
    name: String,
    updated_at: String,
    downloads: u64,
    repository: Option<String>,
    description: Option<String>,
    max_version: String,
}

/// Query crates.io for metadata about a specific crate.
///
/// Sets the required User-Agent header per crates.io API policy.
/// Returns `CrateInfo` on success, or an error string on failure.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the response status
/// is not 200, or the response body cannot be deserialized.
pub fn query_crate_info(name: &str) -> Result<CrateInfo, String> {
    let url = format!("{}/{}", CRATES_IO_API, name);

    let response = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("HTTP request failed for {}: {}", name, e))?;

    if response.status() != 200 {
        return Err(format!(
            "crates.io returned status {} for {}",
            response.status(),
            name
        ));
    }

    let api_response: CratesIoResponse = response
        .into_json()
        .map_err(|e| format!("Failed to parse response for {}: {}", name, e))?;

    Ok(CrateInfo {
        name: api_response.krate.name,
        updated_at: api_response.krate.updated_at,
        downloads: api_response.krate.downloads,
        repository: api_response.krate.repository,
        description: api_response.krate.description,
        max_version: api_response.krate.max_version,
    })
}
