use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Serialize, Deserialize)]
struct VersionCache {
    last_check: SystemTime,
    latest_version: String,
}

pub fn get_latest_version(force_check: bool) -> Result<Option<String>> {
    let current_version = env!("CARGO_PKG_VERSION");

    if !force_check {
        if let Some(cached) = read_version_cache()? {
            if cached.last_check + CHECK_INTERVAL > SystemTime::now() {
                if cached.latest_version != current_version {
                    return Ok(Some(cached.latest_version));
                }
                return Ok(None);
            }
        }
    }

    // Perform fresh check
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.github.com/repos/osteele/stitch-sync/releases/latest")
        .header("User-Agent", "stitch-sync")
        .send()?;

    let release_info: serde_json::Value = response.json()?;
    let latest_version = release_info["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid release info"))?
        .trim_start_matches('v')
        .to_string();

    // Cache the result
    cache_version_check(&latest_version)?;

    if latest_version != current_version {
        Ok(Some(latest_version))
    } else {
        Ok(None)
    }
}

fn get_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("stitch-sync")
        .join("version-cache.json")
}

fn read_version_cache() -> Result<Option<VersionCache>> {
    let path = get_cache_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn cache_version_check(latest_version: &str) -> Result<()> {
    let cache = VersionCache {
        last_check: SystemTime::now(),
        latest_version: latest_version.to_string(),
    };

    let path = get_cache_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, serde_json::to_string(&cache)?)?;
    Ok(())
}
