use std::path::PathBuf;

use brazen::cache::{AssetQuery, AssetStore};
use brazen::{BrazenConfig, PlatformPaths};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut url = None;
    let mut mime = None;
    let mut hash = None;
    let mut session_id = None;
    let mut tab_id = None;
    let mut status_code = None;
    let mut export_path = None;
    let mut import_path = None;
    let mut manifest_path = None;
    let mut profile = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" => {
                i += 1;
                url = args.get(i).cloned();
            }
            "--mime" => {
                i += 1;
                mime = args.get(i).cloned();
            }
            "--hash" => {
                i += 1;
                hash = args.get(i).cloned();
            }
            "--session" => {
                i += 1;
                session_id = args.get(i).cloned();
            }
            "--tab" => {
                i += 1;
                tab_id = args.get(i).cloned();
            }
            "--status" => {
                i += 1;
                status_code = args.get(i).and_then(|value| value.parse::<u16>().ok());
            }
            "--export" => {
                i += 1;
                export_path = args.get(i).cloned();
            }
            "--import" => {
                i += 1;
                import_path = args.get(i).cloned();
            }
            "--manifest" => {
                i += 1;
                manifest_path = args.get(i).cloned();
            }
            "--profile" => {
                i += 1;
                profile = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    let platform = PlatformPaths::detect()?;
    let config_path = platform.default_config_path();
    let mut config = BrazenConfig::load_with_defaults(&config_path)?;
    if let Some(profile) = profile {
        config.profiles.active_profile = profile;
    }
    let runtime = platform.resolve_runtime_paths(&config, &config_path)?;

    let mut store = AssetStore::load(
        config.cache.clone(),
        &runtime,
        config.profiles.active_profile.clone(),
    );

    if let Some(path) = import_path {
        store.import_json(PathBuf::from(path).as_path())?;
    }
    if let Some(path) = export_path {
        store.export_json(PathBuf::from(path).as_path())?;
    }
    if let Some(path) = manifest_path {
        store.build_replay_manifest(PathBuf::from(path).as_path())?;
    }

    let query = AssetQuery {
        url,
        mime,
        hash,
        session_id,
        tab_id,
        status_code,
    };
    let results = store.query(query);
    for entry in results {
        println!(
            "{} {} {} {}",
            entry.created_at,
            entry.mime,
            entry.url,
            entry.hash.clone().unwrap_or_else(|| "-".to_string())
        );
    }

    Ok(())
}
