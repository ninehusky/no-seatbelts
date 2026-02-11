use serde::{Deserialize, Serialize};
use std::error::Error;

// grab the thing from src/experiments/survey.rs.
// TODO: this is separate now, it shouldn't be.

const TOP_K: usize = 1000; // how many crates to scan
const PER_PAGE: usize = 100; // how many crates per page to request from the API
const WANT: usize = 10; // how many binaries to keep

#[derive(Deserialize)]
struct CratesResponse {
    crates: Vec<CrateSummary>,
}

#[derive(Deserialize)]
struct CrateSummary {
    name: String,
    max_version: String,
    repository: Option<String>,
}

#[derive(Deserialize)]
struct VersionsResponse {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    num: String,
    bin_names: Option<Vec<String>>,
}

#[derive(Serialize)]
struct BinaryCrate {
    name: String,
    version: String,
    repository: Option<String>,
}

fn is_binary_version(v: &Version) -> bool {
    match &v.bin_names {
        Some(bin_names) => !bin_names.is_empty(),
        None => false,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("no-seatbelts-eval/0.1 (contact: acheung)")
        .build()?;

    // 1. Get top crates by downloads
    let mut page = 1;
    let mut binaries = Vec::new();

    let mut all_crates = Vec::new();
    while all_crates.len() < TOP_K {
        let url = format!(
            "https://crates.io/api/v1/crates?sort=downloads&per_page={}&page={}",
            PER_PAGE, page
        );

        let resp: CratesResponse = client.get(&url).send()?.json()?;

        if resp.crates.is_empty() {
            break;
        }

        all_crates.extend(resp.crates);

        page += 1;
    }

    all_crates.truncate(TOP_K);

    // 2. For each crate, check if latest version defines a binary
    for krate in all_crates {
        let version_url = format!("https://crates.io/api/v1/crates/{}/versions", krate.name);

        let resp = client.get(&version_url).send();
        if resp.is_err() {
            continue;
        }

        let version_resp: VersionsResponse = match resp?.json() {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "Failed to parse version response for crate {}: {}",
                    krate.name, e
                );
                continue;
            }
        };

        if is_binary_version(&version_resp.versions.last().unwrap()) {
            binaries.push(BinaryCrate {
                name: krate.name,
                version: krate.max_version,
                repository: krate.repository,
            });
        }

        if binaries.len() >= WANT {
            break;
        }
    }

    // 4. Write the JSON file to disk.
    let output_path = std::env::current_dir()?.join("top_bins.json");
    std::fs::write(&output_path, serde_json::to_string_pretty(&binaries)?)?;
    println!("Wrote top binaries to {}", output_path.display());

    Ok(())
}
