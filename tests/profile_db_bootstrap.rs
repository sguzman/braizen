use brazen::app::{ReadingQueueItem, build_shell_state};
use brazen::config::BrazenConfig;
use brazen::platform_paths::RuntimePaths;
use brazen::profile_db::ProfileDb;
use brazen::ServoEngineFactory;

#[test]
fn build_shell_state_loads_profile_sqlite_state() {
    let dir = tempfile::tempdir().unwrap();
    let paths = RuntimePaths {
        config_path: dir.path().join("brazen.toml"),
        data_dir: dir.path().join("data"),
        logs_dir: dir.path().join("logs"),
        profiles_dir: dir.path().join("profiles"),
        cache_dir: dir.path().join("cache"),
        downloads_dir: dir.path().join("downloads"),
        crash_dumps_dir: dir.path().join("crash"),
        active_profile_dir: dir.path().join("profiles/default"),
        session_path: dir.path().join("profiles/default/session.json"),
        audit_log_path: dir.path().join("logs/audit.jsonl"),
    };
    std::fs::create_dir_all(&paths.active_profile_dir).unwrap();

    let db = ProfileDb::open(paths.active_profile_dir.join("state.sqlite")).unwrap();
    db.save_tts_state(true, &["hello".to_string()]).unwrap();
    db.upsert_reading_item(&ReadingQueueItem {
        url: "https://example.com/article".to_string(),
        title: Some("Example".to_string()),
        kind: "article".to_string(),
        saved_at: "now".to_string(),
        progress: 0.25,
        article_text: Some("text".to_string()),
    })
    .unwrap();
    let mut counts = std::collections::HashMap::new();
    counts.insert("https://example.com/".to_string(), 2);
    db.save_visit_stats(2, 1, &counts).unwrap();
    db.append_history("https://example.com/", Some("Example"), "now")
        .unwrap();

    let config = BrazenConfig::default();
    let engine_factory = ServoEngineFactory;
    let shell = build_shell_state(&config, &paths, &engine_factory);

    assert!(shell.tts_playing);
    assert_eq!(shell.tts_queue.front().map(|s| s.as_str()), Some("hello"));
    assert_eq!(shell.reading_queue.len(), 1);
    assert_eq!(shell.visit_total, 2);
    assert_eq!(shell.revisit_total, 1);
    assert!(!shell.history.is_empty());
}

