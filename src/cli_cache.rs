use std::collections::BTreeMap;
use std::io::Read;
use std::time::Duration;

use chrono::Utc;

use crate::cache::{AssetMetadata, AssetStore, StorageMode};
use crate::config::BrazenConfig;
use crate::logging::init_tracing;
use crate::platform_paths::{PlatformPaths, RuntimePaths};

#[derive(Debug, Clone)]
pub struct CacheCliOptions {
    pub url: String,
    pub profile: Option<String>,
    pub timeout_secs: u64,
    pub stats: bool,
}

#[derive(Debug, Clone)]
pub struct CacheFetchResult {
    pub metadata: AssetMetadata,
    pub entry_count: usize,
    pub storage_mode: StorageMode,
}

pub fn parse_cache_args(args: &[String]) -> Result<CacheCliOptions, String> {
    if args.is_empty() || args[0] != "cache" {
        return Err("expected `cache <url>`".to_string());
    }
    let mut url = None;
    let mut profile = None;
    let mut timeout_secs = 30u64;
    let mut stats = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--profile" => {
                i += 1;
                profile = args.get(i).cloned();
            }
            "--timeout" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "missing timeout value".to_string())?;
                timeout_secs = value
                    .parse::<u64>()
                    .map_err(|_| "timeout must be integer seconds".to_string())?;
            }
            "--stats" => {
                stats = true;
            }
            value if url.is_none() => {
                url = Some(value.to_string());
            }
            value => {
                return Err(format!("unrecognized argument `{value}`"));
            }
        }
        i += 1;
    }

    let url = url.ok_or_else(|| "missing url".to_string())?;
    Ok(CacheCliOptions {
        url,
        profile,
        timeout_secs,
        stats,
    })
}

pub fn run_cache_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_cache_args(args).map_err(|error| format!("cache args error: {error}"))?;
    let platform = PlatformPaths::detect()?;
    let config_path = platform.default_config_path();
    let mut config = BrazenConfig::load_with_defaults(&config_path)?;
    if let Some(profile) = options.profile.clone() {
        config.profiles.active_profile = profile;
    }
    let runtime = platform.resolve_runtime_paths(&config, &config_path)?;
    init_tracing(&config.logging, &runtime.logs_dir)?;

    let result = fetch_and_store(&config, &runtime, &options)?;
    println!(
        "cached url={} mime={} size={} hash={} storage={:?} entries={}",
        result.metadata.url,
        result.metadata.mime,
        result.metadata.size_bytes,
        result
            .metadata
            .hash
            .clone()
            .unwrap_or_else(|| "-".to_string()),
        result.storage_mode,
        result.entry_count
    );
    if options.stats {
        println!(
            "capture_mode={:?} truncated={} third_party={} authenticated={}",
            result.metadata.capture_mode,
            result.metadata.truncated,
            result.metadata.is_third_party,
            result.metadata.authenticated
        );
    }

    tracing::info!(
        target: "brazen::cache::cli",
        url = %result.metadata.url,
        mime = %result.metadata.mime,
        size_bytes = result.metadata.size_bytes,
        hash = ?result.metadata.hash,
        storage_mode = ?result.storage_mode,
        entries = result.entry_count,
        "cache cli fetch complete"
    );

    Ok(())
}

pub fn fetch_and_store(
    config: &BrazenConfig,
    runtime: &RuntimePaths,
    options: &CacheCliOptions,
) -> Result<CacheFetchResult, Box<dyn std::error::Error>> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(options.timeout_secs))
        .redirects(10)
        .build();

    let started_at = Utc::now();
    let response = agent
        .get(&options.url)
        .call()
        .map_err(|error| format!("cache fetch failed: {error}"))?;
    let status = response.status();
    if status >= 400 {
        return Err(format!("cache fetch returned status {status}").into());
    }

    let final_url = response.get_url().to_string();
    let mut headers = BTreeMap::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            headers.insert(name.to_lowercase(), value.to_string());
        }
    }
    let mime = response
        .header("content-type")
        .and_then(|value| value.split(';').next())
        .unwrap_or("application/octet-stream")
        .trim()
        .to_string();

    let mut body = Vec::new();
    let mut reader = response.into_reader();
    reader.read_to_end(&mut body)?;
    let finished_at = Utc::now();

    let mut store = AssetStore::load(
        config.cache.clone(),
        runtime,
        config.profiles.active_profile.clone(),
    );

    let metadata = store.record_asset_with_timing(
        &final_url,
        &mime,
        Some(&body),
        headers,
        false,
        false,
        Some("cli".to_string()),
        None,
        None,
        Some(started_at.to_rfc3339()),
        Some(finished_at.to_rfc3339()),
    )?;

    Ok(CacheFetchResult {
        metadata,
        entry_count: store.entries().len(),
        storage_mode: store.storage_mode(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cache_args_accepts_url_and_flags() {
        let args = vec![
            "cache".to_string(),
            "https://example.com".to_string(),
            "--profile".to_string(),
            "dev".to_string(),
            "--timeout".to_string(),
            "12".to_string(),
            "--stats".to_string(),
        ];
        let options = parse_cache_args(&args).unwrap();
        assert_eq!(options.url, "https://example.com");
        assert_eq!(options.profile.as_deref(), Some("dev"));
        assert_eq!(options.timeout_secs, 12);
        assert!(options.stats);
    }
}
