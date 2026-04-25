#[cfg(feature = "servo")]
mod tests {
    use brazen::config::EngineConfig;
    use brazen::engine::RenderSurfaceMetadata;
    use brazen::servo_embedder::{ServoEmbedder, ServoEmbedderConfig};

    #[test]
    fn servo_embedder_renders_frame_after_surface_attach() {
        let config = ServoEmbedderConfig::from_brazen_config(&brazen::config::BrazenConfig::default());
        let mount_manager = brazen::mounts::MountManager::new();
        let session = std::sync::Arc::new(std::sync::RwLock::new(brazen::session::SessionSnapshot::new("test".to_string(), "now".to_string())));
        let paths = brazen::platform_paths::RuntimePaths {
            config_path: "brazen.toml".into(),
            data_dir: "data".into(),
            logs_dir: "logs".into(),
            profiles_dir: "profiles".into(),
            cache_dir: "cache".into(),
            downloads_dir: "downloads".into(),
            crash_dumps_dir: "crash-dumps".into(),
            active_profile_dir: "profiles/default".into(),
            session_path: "profiles/default/session.json".into(),
            audit_log_path: "logs/audit.jsonl".into(),
        };
        let mut embedder = ServoEmbedder::new(config, mount_manager, session, paths);
        embedder.init().unwrap();
        embedder.attach_surface(
            brazen::engine::RenderSurfaceHandle {
                id: 1,
                label: "test".to_string(),
            },
            RenderSurfaceMetadata {
                viewport_width: 64,
                viewport_height: 64,
                scale_factor_basis_points: 100,
            },
        );
        let frame = embedder.render_frame();
        assert!(frame.is_some());
        let frame = frame.unwrap();
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 64);
        assert_eq!(frame.pixels.len(), 64 * 64 * 4);
        assert_eq!(frame.stride_bytes, 64 * 4);
    }
}
