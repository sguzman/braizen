use super::*;

    use super::*;
    use crate::engine::{BrowserEngine, BrowserTab, EngineEvent, EngineFrame, EngineStatus};

    fn test_paths() -> RuntimePaths {
        RuntimePaths {
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
        }
    }

    fn build_test_app() -> BrazenApp {
        let config = BrazenConfig::default();
        let paths = test_paths();
        let engine_factory = crate::engine::ServoEngineFactory;
        let shell_state = build_shell_state(&config, &paths, &engine_factory);
        BrazenApp::new(config, shell_state, None)
    }

    struct MockEngine {
        status: EngineStatus,
        tab: BrowserTab,
        events: Vec<EngineEvent>,
        zoom: f32,
    }

    impl BrowserEngine for MockEngine {
        fn backend_name(&self) -> &'static str {
            "mock"
        }

        fn instance_id(&self) -> crate::engine::EngineInstanceId {
            1
        }

        fn status(&self) -> EngineStatus {
            self.status.clone()
        }

        fn health(&self) -> crate::engine::RenderHealth {
            crate::engine::RenderHealth {
                resource_reader_ready: Some(true),
                resource_reader_path: Some("/mock/resources".to_string()),
                upstream_active: true,
                last_error: None,
            }
        }

        fn active_tab(&self) -> &BrowserTab {
            &self.tab
        }

        fn navigate(&mut self, url: &str) {
            self.tab.current_url = url.to_string();
            self.events
                .push(EngineEvent::NavigationRequested(url.to_string()));
        }

        fn reload(&mut self) {}

        fn stop(&mut self) {}

        fn go_back(&mut self) {}

        fn go_forward(&mut self) {}

        fn attach_surface(&mut self, _surface: crate::engine::RenderSurfaceHandle) {}

        fn set_render_surface(&mut self, _metadata: RenderSurfaceMetadata) {}

        fn render_frame(&mut self) -> Option<EngineFrame> {
            None
        }

        fn set_focus(&mut self, _focus: crate::engine::FocusState) {}

        fn handle_input(&mut self, _event: crate::engine::InputEvent) {}

        fn handle_ime(&mut self, _event: crate::engine::ImeEvent) {}

        fn handle_clipboard(&mut self, _request: crate::engine::ClipboardRequest) {}

        fn set_page_zoom(&mut self, zoom: f32) {
            self.zoom = zoom;
        }

        fn page_zoom(&self) -> f32 {
            self.zoom
        }

        fn set_verbose_logging(&mut self, _enabled: bool) {}

        fn configure_devtools(&mut self, _enabled: bool, _transport: &str) {}

        fn suspend(&mut self) {}

        fn resume(&mut self) {}

        fn shutdown(&mut self) {}

        fn inject_event(&mut self, event: EngineEvent) {
            self.events.push(event);
        }

        fn take_events(&mut self) -> Vec<EngineEvent> {
            std::mem::take(&mut self.events)
        }

        fn evaluate_javascript(&mut self, _script: String, callback: Box<dyn FnOnce(Result<serde_json::Value, String>) + Send + 'static>) {
            callback(Ok(serde_json::Value::Null));
        }

        fn take_screenshot(&mut self) -> Result<EngineFrame, String> {
            Err("MockEngine does not support screenshots".to_string())
        }

        fn interact_dom(
            &mut self,
            _selector: String,
            _event: String,
            _value: Option<String>,
            callback: Box<dyn FnOnce(Result<(), String>) + Send + 'static>,
        ) {
            callback(Ok(()));
        }
    }

    #[test]
    fn shell_state_sync_handles_ready_and_error_statuses() {
        let paths = test_paths();
        let mut shell = ShellState {
            app_name: "Brazen".to_string(),
            backend_name: "mock".to_string(),
            engine_instance_id: 1,
            engine_status: EngineStatus::Initializing,
            active_tab: BrowserTab {
                id: 1,
                title: "Loading".to_string(),
                current_url: "about:blank".to_string(),
            },
            address_bar_input: "https://example.com".to_string(),
            page_title: "Loading".to_string(),
            load_progress: 0.0,
            can_go_back: false,
            can_go_forward: false,
            document_ready: false,
            load_status: None,
            favicon_url: None,
            metadata_summary: None,
            history: Vec::new(),
            last_committed_url: None,
            active_tab_zoom: 1.0,
            cursor_icon: None,
            was_minimized: false,
            pending_popup: None,
            pending_dialog: None,
            pending_context_menu: None,
            pending_new_window: None,
            last_download: None,
            last_security_warning: None,
            last_crash: None,
            last_crash_dump: None,
            devtools_endpoint: None,
            engine_verbose_logging: false,
            resource_reader_ready: None,
            resource_reader_path: None,
            upstream_active: false,
            upstream_last_error: None,
            render_warning: None,
            session: Arc::new(RwLock::new(SessionSnapshot::new(
                "default".to_string(),
                "now".to_string(),
            ))),
            event_log: Vec::new(),
            log_panel_open: true,
            permission_panel_open: false,
            find_panel_open: false,
            find_query: String::new(),
            capabilities_snapshot: Vec::new(),
            automation_activities: Vec::new(),
            tts_queue: VecDeque::new(),
            tts_playing: false,
            reading_queue: VecDeque::new(),
            reader_mode_open: false,
            reader_mode_source_url: None,
            reader_mode_text: String::new(),
            visit_counts: HashMap::new(),
            visit_total: 0,
            revisit_total: 0,
            runtime_paths: paths,
            mount_manager: crate::mounts::MountManager::new(),
            pending_window_screenshot: Arc::new(std::sync::Mutex::new(None)),
            dom_snapshot: None,
            network_log: VecDeque::new(),
            extracted_entities: Vec::new(),
        };

        let mut ready_engine = MockEngine {
            status: EngineStatus::Ready,
            tab: BrowserTab {
                id: 1,
                title: "Example".to_string(),
                current_url: "https://example.com".to_string(),
            },
            events: vec![EngineEvent::StatusChanged(EngineStatus::Ready)],
            zoom: 1.0,
        };
        shell.sync_from_engine(&mut ready_engine);
        assert_eq!(shell.engine_status, EngineStatus::Ready);
        assert!(shell.event_log.iter().any(|line| line.contains("status:")));

        let mut failing_engine = MockEngine {
            status: EngineStatus::Error("boot failed".to_string()),
            tab: shell.active_tab.clone(),
            events: vec![EngineEvent::StatusChanged(EngineStatus::Error(
                "boot failed".to_string(),
            ))],
            zoom: 1.0,
        };
        shell.sync_from_engine(&mut failing_engine);
        assert_eq!(
            shell.engine_status,
            EngineStatus::Error("boot failed".to_string())
        );
    }

    #[test]
    fn zoom_steps_clamp_to_config_bounds() {
        let mut app = build_test_app();
        app.apply_zoom_steps(10, "test");
        assert!((app.shell_state.active_tab_zoom - 2.0).abs() < f32::EPSILON);
        app.apply_zoom_steps(200, "test");
        assert!((app.shell_state.active_tab_zoom - app.config.engine.zoom_max).abs() < 0.001);
        app.apply_zoom_steps(-200, "test");
        assert!((app.shell_state.active_tab_zoom - app.config.engine.zoom_min).abs() < 0.001);
    }

    #[test]
    fn context_menu_event_sets_pending_state() {
        let paths = test_paths();
        let mut shell = ShellState {
            app_name: "Brazen".to_string(),
            backend_name: "mock".to_string(),
            engine_instance_id: 1,
            engine_status: EngineStatus::Initializing,
            active_tab: BrowserTab {
                id: 1,
                title: "Loading".to_string(),
                current_url: "about:blank".to_string(),
            },
            address_bar_input: "https://example.com".to_string(),
            page_title: "Loading".to_string(),
            load_progress: 0.0,
            can_go_back: false,
            can_go_forward: false,
            document_ready: false,
            load_status: None,
            favicon_url: None,
            metadata_summary: None,
            history: Vec::new(),
            last_committed_url: None,
            active_tab_zoom: 1.0,
            cursor_icon: None,
            was_minimized: false,
            pending_popup: None,
            pending_dialog: None,
            pending_context_menu: None,
            pending_new_window: None,
            last_download: None,
            last_security_warning: None,
            last_crash: None,
            last_crash_dump: None,
            devtools_endpoint: None,
            engine_verbose_logging: false,
            resource_reader_ready: None,
            resource_reader_path: None,
            upstream_active: false,
            upstream_last_error: None,
            render_warning: None,
            session: Arc::new(RwLock::new(SessionSnapshot::new(
                "default".to_string(),
                "now".to_string(),
            ))),
            event_log: Vec::new(),
            log_panel_open: true,
            permission_panel_open: false,
            find_panel_open: false,
            find_query: String::new(),
            capabilities_snapshot: Vec::new(),
            automation_activities: Vec::new(),
            tts_queue: VecDeque::new(),
            tts_playing: false,
            reading_queue: VecDeque::new(),
            reader_mode_open: false,
            reader_mode_source_url: None,
            reader_mode_text: String::new(),
            visit_total: 0,
            revisit_total: 0,
            runtime_paths: paths,
            dom_snapshot: None,
            network_log: VecDeque::new(),
            extracted_entities: Vec::new(),
            mount_manager: crate::mounts::MountManager::new(),
            pending_window_screenshot: Arc::new(std::sync::Mutex::new(None)),
            terminal_history: Vec::new(),
            terminal_input: String::new(),
            terminal_busy: false,
            observe_dom: false,
            control_terminal: false,
            use_mcp_tools: false,
            visit_counts: std::collections::HashMap::new(),
        };

        let mut engine = MockEngine {
            status: EngineStatus::Ready,
            tab: shell.active_tab.clone(),
            events: vec![EngineEvent::ContextMenuRequested { x: 120.0, y: 88.0 }],
            zoom: 1.0,
        };

        shell.sync_from_engine(&mut engine);
        assert!(shell.pending_context_menu.is_some());
    }
