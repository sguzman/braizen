pub mod state;
pub mod panels;
pub mod ui_components;
pub mod secondary_ui;
pub mod ui_main;
pub mod input;
pub mod capture;
pub use state::*;


use chrono::Utc;
use serde::{Deserialize, Serialize};
use eframe::egui::Color32;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::automation::{
    AutomationCapabilityEvent, AutomationCommand, AutomationHandle, AutomationNavigationEvent,
    AutomationActivityStatus, drain_automation_commands,
};
use crate::cache::{AssetQuery, AssetStore};
use crate::commands::{AppCommand, dispatch_command};
use crate::config::BrazenConfig;
use crate::engine::{
    AlphaMode, BrowserEngine, BrowserTab, DialogKind, EngineEvent, EngineFactory, EngineLoadStatus,
    EngineStatus, FocusState, InputEvent, RenderSurfaceHandle, RenderSurfaceMetadata,
    SecurityWarningKind, WindowDisposition,
};
use crate::navigation::{normalize_url_input, resolve_startup_url};
use crate::permissions::Capability;
use crate::platform_paths::RuntimePaths;
use crate::rendering::{normalize_pixels, probe_frame_stats};
use crate::session::{NavigationEntry, SessionSnapshot, load_session};
use crate::profile_db::ProfileDb;
use tokio::sync::mpsc;

const _PLACEHOLDER: &str = "";









pub fn build_shell_state(
    config: &BrazenConfig,
    paths: &RuntimePaths,
    engine_factory: &dyn EngineFactory,
) -> ShellState {
    let session_data = load_session(&paths.session_path).unwrap_or_else(|_| {
        SessionSnapshot::new(
            config.profiles.active_profile.clone(),
            Utc::now().to_rfc3339(),
        )
    });
    let session = Arc::new(RwLock::new(session_data));

    let mount_manager = crate::mounts::MountManager::new();
    let mut engine = engine_factory.create(config, paths, mount_manager.clone(), session.clone());
    engine.set_render_surface(RenderSurfaceMetadata {
        viewport_width: config.window.initial_width as u32,
        viewport_height: config.window.initial_height as u32,
        scale_factor_basis_points: 100,
    });

    let capabilities_snapshot = vec![
        (
            Capability::TerminalExec.label().to_string(),
            format!(
                "{:?}",
                config.permissions.decision_for(&Capability::TerminalExec)
            ),
        ),
        (
            Capability::DomRead.label().to_string(),
            format!(
                "{:?}",
                config.permissions.decision_for(&Capability::DomRead)
            ),
        ),
        (
            Capability::CacheRead.label().to_string(),
            format!(
                "{:?}",
                config.permissions.decision_for(&Capability::CacheRead)
            ),
        ),
        (
            Capability::TabInspect.label().to_string(),
            format!(
                "{:?}",
                config.permissions.decision_for(&Capability::TabInspect)
            ),
        ),
        (
            Capability::AiToolUse.label().to_string(),
            format!(
                "{:?}",
                config.permissions.decision_for(&Capability::AiToolUse)
            ),
        ),
        (
            Capability::VirtualResourceMount.label().to_string(),
            format!(
                "{:?}",
                config.permissions.decision_for(&Capability::VirtualResourceMount)
            ),
        ),
    ];

    let startup_url = resolve_startup_url(&config.engine.startup_url)
        .ok()
        .flatten();

    let profile_db_path = paths.active_profile_dir.join("state.sqlite");
    let profile_db = ProfileDb::open(&profile_db_path).ok();

    let (tts_playing, tts_queue) = profile_db
        .as_ref()
        .and_then(|db| db.load_tts_state().ok())
        .unwrap_or((false, Vec::new()));
    let reading_queue = profile_db
        .as_ref()
        .and_then(|db| db.load_reading_queue(512).ok())
        .unwrap_or_default();
    let (visit_total, revisit_total, visit_counts) = profile_db
        .as_ref()
        .and_then(|db| db.load_visit_stats().ok())
        .unwrap_or((0, 0, HashMap::new()));
    let history = profile_db
        .as_ref()
        .and_then(|db| db.load_history(256).ok())
        .unwrap_or_default();

    let mut shell_state = ShellState {
        app_name: config.app.name.clone(),
        backend_name: engine.backend_name().to_string(),
        engine_instance_id: engine.instance_id(),
        engine_status: engine.status(),
        active_tab: engine.active_tab().clone(),
        address_bar_input: startup_url
            .clone()
            .unwrap_or_else(|| config.app.homepage.clone()),
        page_title: engine.active_tab().title.clone(),
        load_progress: 0.0,
        can_go_back: false,
        can_go_forward: false,
        document_ready: false,
        load_status: None,
        favicon_url: None,
        metadata_summary: None,
        history,
        last_committed_url: None,
        extracted_entities: Vec::new(),
        active_tab_zoom: {
            let s = session.read().unwrap();
            s.active_tab().map(|t| t.zoom_level).unwrap_or(config.engine.zoom_default)
        },
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
        engine_verbose_logging: config.engine.verbose_logging,
        resource_reader_ready: None,
        resource_reader_path: None,
        upstream_active: false,
        upstream_last_error: None,
        render_warning: None,
        session,
        event_log: vec![
            format!("loaded config for {}", config.app.name),
            format!("data dir: {}", paths.data_dir.display()),
            format!("logs dir: {}", paths.logs_dir.display()),
            format!("session path: {}", paths.session_path.display()),
            format!(
                "engine policy: new-window={}, devtools={}, transport={}",
                config.engine.new_window_policy,
                config.engine.devtools_enabled,
                config.engine.devtools_transport
            ),
            format!(
                "engine limits: memory={}mb cpu={}ms max_tabs={}",
                config.engine.resource_limits.memory_mb,
                config.engine.resource_limits.cpu_ms,
                config.engine.resource_limits.max_tabs
            ),
            format!(
                "startup url: {}",
                startup_url.as_deref().unwrap_or("about:blank (disabled)")
            ),
        ],
        log_panel_open: config.window.show_log_panel_on_startup,
        permission_panel_open: config.window.show_permission_panel_on_startup,
        find_panel_open: false,
        find_query: String::new(),
        capabilities_snapshot,
        automation_activities: Vec::new(),
        tts_queue: tts_queue.into(),
        tts_playing,
        reading_queue: reading_queue.into(),
        reader_mode_open: false,
        reader_mode_source_url: None,
        reader_mode_text: String::new(),
        visit_counts,
        visit_total,
        revisit_total,
        mount_manager: crate::mounts::MountManager::new(),
        runtime_paths: paths.clone(),
        pending_window_screenshot: Arc::new(std::sync::Mutex::new(None)),
        dom_snapshot: None,
        network_log: VecDeque::with_capacity(512),
        terminal_history: Vec::new(),
        terminal_input: String::new(),
        terminal_busy: false,
        observe_dom: false,
        control_terminal: true,
        use_mcp_tools: true,
    };

    shell_state.sync_from_engine(engine.as_mut());
    shell_state
}

pub struct BrazenApp {
    config: BrazenConfig,
    shell_state: ShellState,
    engine: Box<dyn BrowserEngine>,
    engine_factory: crate::engine::ServoEngineFactory,
    surface_handle: RenderSurfaceHandle,
    last_surface: Option<RenderSurfaceMetadata>,
    render_texture: Option<eframe::egui::TextureHandle>,
    render_frame_number: Option<u64>,
    render_frame_size: Option<(u32, u32)>,
    render_frame_format: Option<(
        crate::engine::PixelFormat,
        AlphaMode,
        crate::engine::ColorSpace,
    )>,
    last_frame_pixels: Option<Vec<u8>>,
    render_viewport_rect: Option<eframe::egui::Rect>,
    frame_probe: Option<crate::rendering::FrameProbeStats>,
    blank_frame_streak: u32,
    blank_frame_warned: bool,
    render_capture_logged: bool,
    load_status_started_at: Option<Instant>,
    load_status_warned: bool,
    last_nav_url: Option<String>,
    frame_times: VecDeque<f32>,
    last_frame_instant: Option<Instant>,
    last_frame_ms: Option<f32>,
    upload_times: VecDeque<f32>,
    last_upload_ms: Option<f32>,
    last_pointer_pos: Option<eframe::egui::Pos2>,
    last_pointer_local: Option<eframe::egui::Pos2>,
    pointer_captured: bool,
    pointer_inside: bool,
    last_click_at: Option<Instant>,
    last_click_pos: Option<eframe::egui::Pos2>,
    last_click_button: Option<eframe::egui::PointerButton>,
    click_count: u8,
    capture_next_frame: bool,
    pending_restart_at: Option<chrono::DateTime<Utc>>,
    crash_count: u32,
    cache_store: AssetStore,
    cache_query_url: String,
    cache_query_mime: String,
    cache_query_hash: String,
    cache_query_session: String,
    cache_query_tab: String,
    cache_query_status: String,
    cache_selected_asset: Option<String>,
    cache_export_path: String,
    cache_import_path: String,
    cache_manifest_path: String,
    panels: WorkspacePanels,
    bookmarks: Vec<String>,
    downloads: Vec<String>,
    ui_theme: UiTheme,
    ui_density: UiDensity,
    command_palette_open: bool,
    command_palette_query: String,
    command_palette_focus_pending: bool,
    address_bar_focus_pending: bool,
    pending_startup_url: Option<String>,
    automation_handle: Option<AutomationHandle>,
    automation_rx: Option<tokio::sync::mpsc::UnboundedReceiver<AutomationCommand>>,
    last_nav_event: Option<(String, Option<EngineLoadStatus>, f32)>,
    last_event_log_len: usize,
    pending_screenshot_tx: Option<tokio::sync::oneshot::Sender<Result<crate::engine::EngineFrame, String>>>,
    profile_db: Option<ProfileDb>,
    last_profile_persist_at: Instant,
    terminal_tx: Option<mpsc::UnboundedSender<String>>,
    terminal_rx: Option<mpsc::UnboundedReceiver<crate::terminal::TerminalLine>>,
    last_dom_observation: Instant,
    settings_tab: SettingsTab,
    left_panel_tab: LeftPanelTab,
    processed_mcp_commands: std::collections::HashSet<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PaletteCommand {
    NewTab,
    CloseTab,
    Reload,
    StopLoading,
    GoBack,
    GoForward,
    FocusAddressBar,
    ToggleLogs,
    TogglePermissions,
}

struct PaletteEntry {
    label: &'static str,
    action: PaletteCommand,
}



















impl BrazenApp {
    pub fn new(
        config: BrazenConfig,
        shell_state: ShellState,
        automation: Option<crate::automation::AutomationRuntime>,
    ) -> Self {
        let engine_factory = crate::engine::ServoEngineFactory;
        let mut engine = engine_factory.create(&config, &shell_state.runtime_paths, shell_state.mount_manager.clone(), shell_state.session.clone());
        engine.set_verbose_logging(config.engine.verbose_logging);
        engine.configure_devtools(
            config.engine.devtools_enabled,
            &config.engine.devtools_transport,
        );
        let surface_handle = RenderSurfaceHandle {
            id: 1,
            label: "primary-surface".to_string(),
        };
        let cache_store = AssetStore::load(
            config.cache.clone(),
            &shell_state.runtime_paths,
            config.profiles.active_profile.clone(),
        );
        let capture_next_frame = config.engine.debug_capture_next_frame;
        let pending_startup_url = resolve_startup_url(&config.engine.startup_url)
            .ok()
            .flatten();
        let mut panels = WorkspacePanels::default();
        let mut ui_theme = UiTheme::System;
        let mut ui_density = UiDensity::Comfortable;

        let profile_db = ProfileDb::open(shell_state.runtime_paths.active_profile_dir.join("state.sqlite")).ok();

        if let Some(db) = &profile_db {
            if let Ok(Some(layout_json)) = db.load_workspace_layout() {
                if let Ok(layout) = serde_json::from_str::<WorkspaceLayout>(&layout_json) {
                    panels = layout.panels;
                    ui_theme = layout.theme;
                    ui_density = layout.density;
                }
            }
        }

        let (automation_handle, automation_rx) = automation
            .map(|runtime| (Some(runtime.handle), Some(runtime.command_rx)))
            .unwrap_or((None, None));

        // Register external MCP servers from config
        for (name, srv_config) in &config.mcp.servers {
            match crate::mcp_stdio::StdioMcpServer::spawn(
                name.clone(),
                srv_config.command.clone(),
                srv_config.args.clone(),
                srv_config.env.clone(),
            ) {
                Ok(server) => {
                    crate::mcp::McpBroker::register_server(Box::new(server));
                }
                Err(e) => {
                    tracing::error!("Failed to spawn MCP server {}: {}", name, e);
                }
            }
        }

        let (term_tx, mut term_rx) = mpsc::unbounded_channel::<String>();
        let (term_out_tx, term_out_rx) = mpsc::unbounded_channel::<crate::terminal::TerminalLine>();

        let terminal_config = config.terminal.clone();
        tokio::spawn(async move {
            while let Some(cmd_line) = term_rx.recv().await {
                let parts: Vec<String> = cmd_line.split_whitespace().map(|s| s.to_string()).collect();
                if parts.is_empty() {
                    continue;
                }

                let cmd = parts[0].clone();
                let args = parts[1..].to_vec();

                let _ = term_out_tx.send(crate::terminal::TerminalLine::Status(format!(
                    "Running: {}...",
                    cmd
                )));

                let request = crate::terminal::TerminalRequest {
                    cmd,
                    args,
                    cwd: None,
                };

                let response =
                    crate::terminal::TerminalBroker::execute(&terminal_config, request).await;

                if !response.stdout.is_empty() {
                    let _ = term_out_tx.send(crate::terminal::TerminalLine::Stdout(response.stdout));
                }
                if !response.stderr.is_empty() {
                    let _ = term_out_tx.send(crate::terminal::TerminalLine::Stderr(response.stderr));
                }
                if let Some(err) = response.error {
                    let _ = term_out_tx.send(crate::terminal::TerminalLine::Stderr(err));
                }

                let _ = term_out_tx.send(crate::terminal::TerminalLine::Done(response.success));
            }
        });

        Self {
            config,
            shell_state,
            engine,
            engine_factory,
            surface_handle,
            last_surface: None,
            render_texture: None,
            render_frame_number: None,
            render_frame_size: None,
            render_frame_format: None,
            last_frame_pixels: None,
            render_viewport_rect: None,
            frame_probe: None,
            blank_frame_streak: 0,
            blank_frame_warned: false,
            render_capture_logged: false,
            load_status_started_at: None,
            load_status_warned: false,
            last_nav_url: None,
            frame_times: VecDeque::with_capacity(120),
            last_frame_instant: None,
            last_frame_ms: None,
            upload_times: VecDeque::with_capacity(120),
            last_upload_ms: None,
            last_pointer_pos: None,
            last_pointer_local: None,
            pointer_captured: false,
            pointer_inside: false,
            last_click_at: None,
            last_click_pos: None,
            last_click_button: None,
            click_count: 0,
            capture_next_frame,
            pending_restart_at: None,
            crash_count: 0,
            cache_store,
            cache_query_url: String::new(),
            cache_query_mime: String::new(),
            cache_query_hash: String::new(),
            cache_query_session: String::new(),
            cache_query_tab: String::new(),
            cache_query_status: String::new(),
            cache_selected_asset: None,
            cache_export_path: "cache-export.json".to_string(),
            cache_import_path: "cache-import.json".to_string(),
            cache_manifest_path: "cache-manifest.json".to_string(),
            panels,
            bookmarks: Vec::new(),
            downloads: Vec::new(),
            ui_theme,
            ui_density,
            command_palette_open: false,
            command_palette_query: String::new(),
            command_palette_focus_pending: false,
            address_bar_focus_pending: false,
            pending_startup_url,
            automation_handle,
            automation_rx,
            last_nav_event: None,
            last_event_log_len: 0,
            pending_screenshot_tx: None,
            profile_db,
            last_profile_persist_at: Instant::now(),
            terminal_tx: Some(term_tx),
            terminal_rx: Some(term_out_rx),
            last_dom_observation: Instant::now(),
            settings_tab: SettingsTab::Layout,
            left_panel_tab: LeftPanelTab::Workspace,
            processed_mcp_commands: std::collections::HashSet::new(),
        }
    }

    fn persist_profile_state_if_due(&mut self) {
        if self.last_profile_persist_at.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.last_profile_persist_at = Instant::now();
        let Some(db) = &self.profile_db else {
            return;
        };

        let queue: Vec<String> = self.shell_state.tts_queue.iter().cloned().collect();
        let _ = db.save_tts_state(self.shell_state.tts_playing, &queue);
        let _ = db.save_visit_stats(
            self.shell_state.visit_total,
            self.shell_state.revisit_total,
            &self.shell_state.visit_counts,
        );
        for item in self.shell_state.reading_queue.iter() {
            let _ = db.upsert_reading_item(item);
        }
        self.save_workspace_layout();
    }

    fn frame_probe_enabled(&self) -> bool {
        self.config.engine.debug_frame_probe
    }

    fn update_render_health(&mut self) {
        if self.shell_state.last_committed_url != self.last_nav_url {
            self.last_nav_url = self.shell_state.last_committed_url.clone();
            self.blank_frame_streak = 0;
            self.blank_frame_warned = false;
            self.render_capture_logged = false;
            self.load_status_started_at = None;
            self.load_status_warned = false;
            self.shell_state.render_warning = None;
        }

        match self.shell_state.load_status {
            Some(EngineLoadStatus::Started) => {
                if self.load_status_started_at.is_none() {
                    self.load_status_started_at = Some(Instant::now());
                }
                if let Some(started_at) = self.load_status_started_at
                    && started_at.elapsed() >= Duration::from_secs(10)
                    && !self.load_status_warned
                {
                    self.load_status_warned = true;
                    let warning = "load status stuck at Started for 10s".to_string();
                    tracing::warn!(target: "brazen::render", "{warning}");
                    self.shell_state.record_event(warning.clone());
                    self.shell_state.render_warning = Some(warning);
                }
            }
            Some(_) | None => {
                self.load_status_started_at = None;
                self.load_status_warned = false;
                self.shell_state.render_warning = None;
            }
        }
    }

    fn update_automation(&mut self, ctx: &eframe::egui::Context) {
        if let Some(receiver) = &mut self.automation_rx {
            drain_automation_commands(receiver, &mut self.shell_state, self.engine.as_mut(), &mut self.cache_store);
        }

        if let Some(tx) = self.shell_state.pending_window_screenshot.lock().unwrap().take() {
            tracing::info!(target: "brazen::automation", "received screenshot request, triggering viewport command");
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Screenshot(eframe::egui::UserData::new(0usize)));
            self.pending_screenshot_tx = Some(tx);
        }

        let screenshot = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let eframe::egui::Event::Screenshot { image, .. } = e {
                    tracing::info!(target: "brazen::automation", "received screenshot event from egui");
                    Some(image.clone())
                } else {
                    None
                }
            })
        });

        if let Some(image) = screenshot {
            if let Some(tx) = self.pending_screenshot_tx.take() {
                tracing::info!(target: "brazen::automation", "sending screenshot frame to automation client");
                let frame = crate::engine::EngineFrame {
                    width: image.width() as u32,
                    height: image.height() as u32,
                    stride_bytes: image.width() * 4,
                    pixels: image.as_raw().to_vec(),
                    pixel_format: crate::engine::PixelFormat::Rgba8,
                    alpha_mode: crate::engine::AlphaMode::Premultiplied,
                    color_space: crate::engine::ColorSpace::Srgb,
                    frame_number: 0,
                };
                let _ = tx.send(Ok(frame));
            } else {
                tracing::warn!(target: "brazen::automation", "received screenshot event but no pending tx");
            }
        }

        if let Some(handle) = &self.automation_handle {
            handle.update_snapshot(&self.shell_state, &self.cache_store);
            let automation_snapshot = handle.snapshot();
            self.shell_state.automation_activities = automation_snapshot.activities.into_iter().collect();
            let nav_key = (
                self.shell_state.active_tab.current_url.clone(),
                self.shell_state.load_status,
                (self.shell_state.load_progress * 100.0).round() / 100.0,
            );
            if self.last_nav_event.as_ref() != Some(&nav_key) {
                let event = AutomationNavigationEvent {
                    url: self.shell_state.active_tab.current_url.clone(),
                    title: self.shell_state.page_title.clone(),
                    load_status: self
                        .shell_state
                        .load_status
                        .map(|status| status.as_str().to_string()),
                    load_progress: self.shell_state.load_progress,
                };
                handle.publish_navigation(event);
                self.last_nav_event = Some(nav_key);
            }
            if self.shell_state.event_log.len() > self.last_event_log_len {
                for line in self.shell_state.event_log[self.last_event_log_len..].iter() {
                    let lower = line.to_lowercase();
                    if lower.contains("permission")
                        || lower.contains("capability")
                        || lower.contains("security warning")
                    {
                        handle.publish_capability(AutomationCapabilityEvent {
                            message: line.clone(),
                        });
                    }
                }
                self.last_event_log_len = self.shell_state.event_log.len();
            }
        }

        self.persist_profile_state_if_due();
        
        // Periodic DOM observation
        if self.shell_state.observe_dom && self.last_dom_observation.elapsed() > Duration::from_secs(5) {
            self.last_dom_observation = Instant::now();
            let script = "document.documentElement.innerText".to_string();
            self.engine.evaluate_javascript(script, Box::new(|_result| {
                // The engine's evaluate_javascript implementation now injects 
                // EngineEvent::DomSnapshotUpdated directly if it's successful.
            }));
            // Actually, I should probably just make the engine's evaluate_javascript implementation
            // responsible for injecting the event if it wants to.
            // Or I can use a channel.
        }
    }

    fn update_terminal(&mut self, ctx: &eframe::egui::Context) {
        if let Some(rx) = &mut self.terminal_rx {
            while let Ok(line) = rx.try_recv() {
                match line {
                    crate::terminal::TerminalLine::Stdout(s) => {
                        for l in s.lines() {
                            self.shell_state.terminal_history.push(l.to_string());
                        }
                    }
                    crate::terminal::TerminalLine::Stderr(s) => {
                        for l in s.lines() {
                            self.shell_state.terminal_history.push(format!("[ERR] {}", l));
                        }
                    }
                    crate::terminal::TerminalLine::Status(s) => {
                        self.shell_state.terminal_history.push(format!("-- {}", s));
                    }
                    crate::terminal::TerminalLine::Done(_) => {
                        self.shell_state.terminal_busy = false;
                    }
                }
                ctx.request_repaint();
                // Limit history
                if self.shell_state.terminal_history.len() > 1000 {
                    self.shell_state.terminal_history.remove(0);
                }
            }
        }
    }

    fn apply_cursor_icon(&self, ctx: &eframe::egui::Context) {
        let Some(cursor) = self.shell_state.cursor_icon.as_deref() else {
            return;
        };
        let inside = self
            .last_pointer_pos
            .and_then(|pos| self.render_viewport_rect.map(|rect| rect.contains(pos)))
            .unwrap_or(false);
        if !inside {
            return;
        }
        let icon = match cursor {
            "Pointer" => eframe::egui::CursorIcon::PointingHand,
            "Text" | "VerticalText" => eframe::egui::CursorIcon::Text,
            "Crosshair" => eframe::egui::CursorIcon::Crosshair,
            "Move" | "AllScroll" => eframe::egui::CursorIcon::Move,
            "Grab" => eframe::egui::CursorIcon::Grab,
            "Grabbing" => eframe::egui::CursorIcon::Grabbing,
            "NotAllowed" | "NoDrop" => eframe::egui::CursorIcon::NotAllowed,
            "EResize" | "EwResize" => eframe::egui::CursorIcon::ResizeHorizontal,
            "NResize" | "SResize" | "NsResize" => eframe::egui::CursorIcon::ResizeVertical,
            "NeResize" | "SwResize" | "NeswResize" => eframe::egui::CursorIcon::ResizeNeSw,
            "NwResize" | "SeResize" | "NwseResize" => eframe::egui::CursorIcon::ResizeNwSe,
            "RowResize" => eframe::egui::CursorIcon::ResizeRow,
            "ColResize" => eframe::egui::CursorIcon::ResizeColumn,
            "ZoomIn" => eframe::egui::CursorIcon::ZoomIn,
            "ZoomOut" => eframe::egui::CursorIcon::ZoomOut,
            "Wait" | "Progress" => eframe::egui::CursorIcon::Wait,
            _ => eframe::egui::CursorIcon::Default,
        };
        ctx.output_mut(|output| output.cursor_icon = icon);
    }

    pub fn shell_state(&self) -> &ShellState {
        &self.shell_state
    }

    fn handle_navigation(&mut self) {
        let input = self.shell_state.address_bar_input.trim().to_string();
        self.shell_state
            .session
            .write()
            .unwrap()
            .mark_pending_navigation(&input, Utc::now().to_rfc3339());
        let _ = dispatch_command(
            &mut self.shell_state,
            self.engine.as_mut(),
            AppCommand::NavigateTo(input),
        );
        self.shell_state.sync_from_engine(self.engine.as_mut());
    }

    fn map_pointer_to_viewport(
        &self,
        ctx: &eframe::egui::Context,
        pos: eframe::egui::Pos2,
        allow_outside: bool,
    ) -> Option<eframe::egui::Pos2> {
        let rect = self.render_viewport_rect?;
        if !allow_outside && !rect.contains(pos) {
            return None;
        }
        let mut local = pos - rect.min;
        if let Some(surface) = &self.last_surface {
            let pixels_per_point = ctx.pixels_per_point();
            let max_x = surface.viewport_width as f32 / pixels_per_point;
            let max_y = surface.viewport_height as f32 / pixels_per_point;
            local.x = local.x.clamp(0.0, max_x);
            local.y = local.y.clamp(0.0, max_y);
        }
        Some(eframe::egui::pos2(local.x, local.y))
    }

    fn clamp_zoom(&self, zoom: f32) -> f32 {
        zoom.clamp(self.config.engine.zoom_min, self.config.engine.zoom_max)
    }

    fn set_active_tab_zoom(&mut self, zoom: f32, reason: &str) {
        let clamped = self.clamp_zoom(zoom);
        {
            let mut session = self.shell_state.session.write().unwrap();
            session.active_tab_mut().zoom_level = clamped;
        }
        self.shell_state.active_tab_zoom = clamped;
        self.engine.set_page_zoom(clamped);
        self.shell_state
            .record_event(format!("zoom {reason}: {clamped:.2}x"));
    }

    fn apply_zoom_steps(&mut self, steps: i32, reason: &str) {
        let current = self.shell_state.active_tab_zoom;
        let step = self.config.engine.zoom_step;
        let next = current + step * steps as f32;
        self.set_active_tab_zoom(next, reason);
    }

    fn apply_zoom_factor(&mut self, factor: f32, reason: &str) {
        let current = self.shell_state.active_tab_zoom;
        self.set_active_tab_zoom(current * factor, reason);
    }

    fn update_click_count(
        &mut self,
        button: eframe::egui::PointerButton,
        pos: eframe::egui::Pos2,
    ) -> u8 {
        let now = Instant::now();
        let within_time = self
            .last_click_at
            .map(|at| now.duration_since(at) <= Duration::from_millis(500))
            .unwrap_or(false);
        let within_distance = self
            .last_click_pos
            .map(|last| last.distance(pos) <= 4.0)
            .unwrap_or(false);
        if within_time && within_distance && self.last_click_button == Some(button) {
            self.click_count = (self.click_count + 1).min(3);
        } else {
            self.click_count = 1;
        }
        self.last_click_at = Some(now);
        self.last_click_pos = Some(pos);
        self.last_click_button = Some(button);
        self.click_count
    }














    fn sync_active_tab_from_session(&mut self) {
        let tab = self.shell_state.session.read().unwrap().active_tab().unwrap().clone();
        if self.shell_state.active_tab.current_url != tab.url
            || self.shell_state.active_tab.title != tab.title
        {
            self.shell_state.active_tab.title = tab.title;
            self.shell_state.active_tab.current_url = tab.url;
        }
        if (self.shell_state.active_tab_zoom - tab.zoom_level).abs() > f32::EPSILON {
            self.shell_state.active_tab_zoom = tab.zoom_level;
            self.engine.set_page_zoom(tab.zoom_level);
        }
    }

    fn apply_new_window_policy(&mut self) {
        let Some((url, disposition)) = self.shell_state.pending_new_window.take() else {
            return;
        };
        let policy = self.config.engine.new_window_policy.as_str();
        let decision = match policy {
            "new-tab" => WindowDisposition::BackgroundTab,
            "same-tab" => WindowDisposition::ForegroundTab,
            "block" => WindowDisposition::Blocked,
            _ => disposition.clone(),
        };

        match decision {
            WindowDisposition::ForegroundTab => {
                if let Ok(normalized) = normalize_url_input(&url) {
                    self.engine.navigate(&normalized);
                    self.shell_state
                        .record_event(format!("new window routed to current tab: {normalized}"));
                } else {
                    self.shell_state
                        .record_event(format!("new window navigation failed: {url}"));
                }
            }
            WindowDisposition::BackgroundTab | WindowDisposition::NewWindow => {
                if let Ok(normalized) = normalize_url_input(&url) {
                    {
                        let mut session = self.shell_state.session.write().unwrap();
                        session.open_new_tab(&normalized, "New Tab");
                        session.active_tab_mut().zoom_level = self.config.engine.zoom_default;
                    }
                    self.shell_state.active_tab_zoom = self.config.engine.zoom_default;
                    self.shell_state
                        .record_event(format!("new window opened as tab: {normalized}"));
                } else {
                    self.shell_state
                        .record_event(format!("new window tab open failed: {url}"));
                }
            }
            WindowDisposition::Blocked => {
                self.shell_state
                    .record_event(format!("new window blocked: {url}"));
            }
        }
    }

    fn write_crash_dump(&mut self, reason: &str) {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        let filename = format!("crash-{timestamp}.log");
        let path = self
            .shell_state
            .runtime_paths
            .crash_dumps_dir
            .join(filename);
        let _ = std::fs::create_dir_all(&self.shell_state.runtime_paths.crash_dumps_dir);
        let payload = format!(
            "timestamp={}\nreason={}\nsession_id={}\nprofile={}\nactive_url={}\n",
            timestamp,
            reason,
            self.shell_state.session.read().unwrap().session_id.0,
            self.shell_state.session.read().unwrap().profile_id.clone(),
            self.shell_state.active_tab.current_url
        );
        let _ = std::fs::write(&path, payload.as_bytes());
        self.shell_state.last_crash_dump = Some(path.display().to_string());
    }

    fn restart_engine(&mut self) {
        self.engine.shutdown();
        self.engine = self.engine_factory.create(
            &self.config,
            &self.shell_state.runtime_paths,
            self.shell_state.mount_manager.clone(),
            self.shell_state.session.clone(),
        );
        self.engine
            .set_verbose_logging(self.shell_state.engine_verbose_logging);
        self.engine.configure_devtools(
            self.config.engine.devtools_enabled,
            &self.config.engine.devtools_transport,
        );
        self.last_surface = None;
        self.render_texture = None;
        self.shell_state.record_event("engine restarted");
    }

    fn schedule_restart(&mut self) {
        if self.pending_restart_at.is_some() {
            return;
        }
        self.crash_count = self.crash_count.saturating_add(1);
        let exponent = self.crash_count.min(5);
        let backoff = 2u64.pow(exponent);
        let delay = chrono::Duration::seconds(backoff as i64);
        let scheduled = Utc::now() + delay;
        self.pending_restart_at = Some(scheduled);
        self.shell_state
            .record_event(format!("engine restart scheduled in {backoff}s"));
    }

    fn handle_crash_recovery(&mut self) {
        if self.shell_state.last_crash.is_some() {
            self.schedule_restart();
        }
        if let Some(scheduled) = self.pending_restart_at
            && Utc::now() >= scheduled
        {
            self.restart_engine();
            self.shell_state.last_crash = None;
            self.shell_state.session.write().unwrap().crash_recovery_pending = false;
            self.pending_restart_at = None;
        }
    }

    fn palette_entries() -> &'static [PaletteEntry] {
        &[
            PaletteEntry {
                label: "New Tab",
                action: PaletteCommand::NewTab,
            },
            PaletteEntry {
                label: "Close Tab",
                action: PaletteCommand::CloseTab,
            },
            PaletteEntry {
                label: "Reload",
                action: PaletteCommand::Reload,
            },
            PaletteEntry {
                label: "Stop Loading",
                action: PaletteCommand::StopLoading,
            },
            PaletteEntry {
                label: "Go Back",
                action: PaletteCommand::GoBack,
            },
            PaletteEntry {
                label: "Go Forward",
                action: PaletteCommand::GoForward,
            },
            PaletteEntry {
                label: "Focus Address Bar",
                action: PaletteCommand::FocusAddressBar,
            },
            PaletteEntry {
                label: "Toggle Logs Panel",
                action: PaletteCommand::ToggleLogs,
            },
            PaletteEntry {
                label: "Toggle Permissions Panel",
                action: PaletteCommand::TogglePermissions,
            },
        ]
    }

    fn open_command_palette(&mut self) {
        self.command_palette_open = true;
        self.command_palette_focus_pending = true;
        self.command_palette_query.clear();
    }

    fn apply_palette_command(&mut self, action: PaletteCommand) {
        match action {
            PaletteCommand::NewTab => {
                {
                    let mut session = self.shell_state.session.write().unwrap();
                    session.open_new_tab("about:blank", "New Tab");
                    session.active_tab_mut().zoom_level = self.config.engine.zoom_default;
                }
                self.shell_state.active_tab_zoom = self.config.engine.zoom_default;
                self.sync_active_tab_from_session();
                self.shell_state.address_bar_input =
                    self.shell_state.active_tab.current_url.clone();
                self.shell_state.record_event("palette: new tab");
            }
            PaletteCommand::CloseTab => {
                self.shell_state.session.write().unwrap().close_active_tab();
                self.sync_active_tab_from_session();
                self.shell_state.address_bar_input =
                    self.shell_state.active_tab.current_url.clone();
                self.shell_state.record_event("palette: close tab");
            }
            PaletteCommand::Reload => {
                let _ = dispatch_command(
                    &mut self.shell_state,
                    self.engine.as_mut(),
                    AppCommand::ReloadActiveTab,
                );
                self.shell_state.record_event("palette: reload");
            }
            PaletteCommand::StopLoading => {
                let _ = dispatch_command(
                    &mut self.shell_state,
                    self.engine.as_mut(),
                    AppCommand::StopLoading,
                );
                self.shell_state.record_event("palette: stop loading");
            }
            PaletteCommand::GoBack => {
                let _ = dispatch_command(
                    &mut self.shell_state,
                    self.engine.as_mut(),
                    AppCommand::GoBack,
                );
                self.shell_state.session.write().unwrap().go_back(Utc::now().to_rfc3339());
                self.shell_state.record_event("palette: go back");
            }
            PaletteCommand::GoForward => {
                let _ = dispatch_command(
                    &mut self.shell_state,
                    self.engine.as_mut(),
                    AppCommand::GoForward,
                );
                self.shell_state.session.write().unwrap().go_forward(Utc::now().to_rfc3339());
                self.shell_state.record_event("palette: go forward");
            }
            PaletteCommand::FocusAddressBar => {
                self.address_bar_focus_pending = true;
                self.shell_state.record_event("palette: focus address bar");
            }
            PaletteCommand::ToggleLogs => {
                let _ = dispatch_command(
                    &mut self.shell_state,
                    self.engine.as_mut(),
                    AppCommand::ToggleLogPanel,
                );
            }
            PaletteCommand::TogglePermissions => {
                self.shell_state.permission_panel_open = !self.shell_state.permission_panel_open;
                self.shell_state.record_event(format!(
                    "permission panel {}",
                    if self.shell_state.permission_panel_open {
                        "opened"
                    } else {
                        "closed"
                    }
                ));
            }
        }
    }

    fn load_workspace_layout(path: &PathBuf) -> Option<WorkspaceLayout> {
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    fn save_workspace_layout(&self) {
        let Some(db) = &self.profile_db else {
            return;
        };
        let payload = WorkspaceLayout {
            panels: self.panels,
            theme: self.ui_theme,
            density: self.ui_density,
        };
        if let Ok(data) = serde_json::to_string_pretty(&payload) {
            let _ = db.save_workspace_layout(&data);
        }
    }

    fn apply_layout_preset(&mut self, preset: LayoutPreset) {
        self.panels = match preset {
            LayoutPreset::Default => WorkspacePanels {
                sidebar_visible: true,
                bookmarks: false,
                history: false,
                reading_queue: false,
                reader_mode: false,
                tts_controls: false,
                workspace_settings: false,
                terminal: false,
                dashboard: true,
                find_panel_open: false,
                bottom_panel_visible: false,
                active_diagnostic_tab: DiagnosticTab::Logs,
                bottom_panel_height: 250.0,
            },
            LayoutPreset::Developer => WorkspacePanels {
                sidebar_visible: true,
                bookmarks: false,
                history: false,
                reading_queue: false,
                reader_mode: false,
                tts_controls: false,
                workspace_settings: false,
                terminal: true,
                dashboard: false,
                find_panel_open: false,
                bottom_panel_visible: true,
                active_diagnostic_tab: DiagnosticTab::Network,
                bottom_panel_height: 300.0,
            },
            LayoutPreset::Archive => WorkspacePanels {
                sidebar_visible: true,
                terminal: false,
                dashboard: false,
                reading_queue: true,
                reader_mode: true,
                tts_controls: true,
                bookmarks: true,
                history: true,
                workspace_settings: true,
                find_panel_open: false,
                bottom_panel_visible: true,
                active_diagnostic_tab: DiagnosticTab::KnowledgeGraph,
                bottom_panel_height: 350.0,
            },
        };
        self.shell_state
            .record_event(format!("layout preset applied: {preset:?}"));
        self.save_workspace_layout();
    }

    fn apply_ui_settings(&self, ctx: &eframe::egui::Context) {
        match self.ui_theme {
            UiTheme::System => {}
            UiTheme::Light => ctx.set_visuals(eframe::egui::Visuals::light()),
            UiTheme::Dark => ctx.set_visuals(eframe::egui::Visuals::dark()),
            UiTheme::Brazen => crate::ui_theme::apply_brazen_style(ctx),
        }
        let mut style = (*ctx.style()).clone();
        match self.ui_density {
            UiDensity::Compact => {
                style.spacing.item_spacing = eframe::egui::vec2(6.0, 4.0);
                style.spacing.button_padding = eframe::egui::vec2(6.0, 4.0);
            }
            UiDensity::Comfortable => {
                style.spacing.item_spacing = eframe::egui::vec2(10.0, 8.0);
                style.spacing.button_padding = eframe::egui::vec2(10.0, 6.0);
            }
        }
        ctx.set_style(style);
    }
}




#[cfg(test)]
mod tests;


fn extract_entities(html: &str) -> Vec<ExtractedEntity> {
    crate::extraction::extract_entities(html)
}
