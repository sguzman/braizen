use crate::commands::{AppCommand, dispatch_command};
use crate::config::BrazenConfig;
use crate::engine::{
    BrowserEngine, BrowserTab, EngineFactory, EngineStatus, RenderSurfaceMetadata,
};
use crate::permissions::Capability;
use crate::platform_paths::RuntimePaths;

#[derive(Debug, Clone)]
pub struct ShellState {
    pub app_name: String,
    pub backend_name: String,
    pub engine_status: EngineStatus,
    pub active_tab: BrowserTab,
    pub address_bar_input: String,
    pub event_log: Vec<String>,
    pub log_panel_open: bool,
    pub permission_panel_open: bool,
    pub capabilities_snapshot: Vec<(String, String)>,
    pub runtime_paths: RuntimePaths,
}

impl ShellState {
    pub fn record_event(&mut self, event: impl Into<String>) {
        self.event_log.push(event.into());
    }

    pub fn sync_from_engine(&mut self, engine: &mut dyn BrowserEngine) {
        self.engine_status = engine.status();
        self.active_tab = engine.active_tab().clone();
        for event in engine.take_events() {
            self.record_event(format!("engine event: {event:?}"));
        }
    }
}

pub fn build_shell_state(
    config: &BrazenConfig,
    paths: &RuntimePaths,
    engine_factory: &dyn EngineFactory,
) -> ShellState {
    let mut engine = engine_factory.create(config, paths);
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
    ];

    let mut shell_state = ShellState {
        app_name: config.app.name.clone(),
        backend_name: engine.backend_name().to_string(),
        engine_status: engine.status(),
        active_tab: engine.active_tab().clone(),
        address_bar_input: config.app.homepage.clone(),
        event_log: vec![
            format!("loaded config for {}", config.app.name),
            format!("data dir: {}", paths.data_dir.display()),
            format!("logs dir: {}", paths.logs_dir.display()),
        ],
        log_panel_open: config.window.show_log_panel_on_startup,
        permission_panel_open: config.window.show_permission_panel_on_startup,
        capabilities_snapshot,
        runtime_paths: paths.clone(),
    };

    shell_state.sync_from_engine(engine.as_mut());
    shell_state
}

pub struct BrazenApp {
    config: BrazenConfig,
    shell_state: ShellState,
    engine: Box<dyn BrowserEngine>,
}

impl BrazenApp {
    pub fn new(config: BrazenConfig, shell_state: ShellState) -> Self {
        let factory = crate::engine::ServoEngineFactory;
        let engine = factory.create(&config, &shell_state.runtime_paths);

        Self {
            config,
            shell_state,
            engine,
        }
    }

    pub fn shell_state(&self) -> &ShellState {
        &self.shell_state
    }

    fn handle_navigation(&mut self) {
        let input = self.shell_state.address_bar_input.trim().to_string();
        let _ = dispatch_command(
            &mut self.shell_state,
            self.engine.as_mut(),
            AppCommand::NavigateTo(input),
        );
        self.shell_state.sync_from_engine(self.engine.as_mut());
    }
}

impl eframe::App for BrazenApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.shell_state.sync_from_engine(self.engine.as_mut());

        eframe::egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(&self.config.app.name);
                ui.label(format!("backend: {}", self.shell_state.backend_name));
                ui.separator();
                ui.label(format!("status: {}", self.shell_state.engine_status));
            });

            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.shell_state.address_bar_input);
                let enter_pressed = ui.input(|input| input.key_pressed(eframe::egui::Key::Enter));
                if response.lost_focus() && enter_pressed {
                    self.handle_navigation();
                }
                if ui.button("Go").clicked() {
                    self.handle_navigation();
                }
                if ui.button("Reload").clicked() {
                    let _ = dispatch_command(
                        &mut self.shell_state,
                        self.engine.as_mut(),
                        AppCommand::ReloadActiveTab,
                    );
                }
                if ui.button("Logs").clicked() {
                    let _ = dispatch_command(
                        &mut self.shell_state,
                        self.engine.as_mut(),
                        AppCommand::ToggleLogPanel,
                    );
                }
                if ui.button("Permissions").clicked() {
                    let _ = dispatch_command(
                        &mut self.shell_state,
                        self.engine.as_mut(),
                        AppCommand::OpenPermissionPanel,
                    );
                }
            });
        });

        eframe::egui::SidePanel::left("tab_sidebar")
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.heading("Workspace");
                ui.label("Tab 1");
                ui.separator();
                ui.label(format!("Title: {}", self.shell_state.active_tab.title));
                ui.label(format!("URL: {}", self.shell_state.active_tab.current_url));
                ui.label(format!(
                    "Profiles: {}",
                    self.shell_state.runtime_paths.profiles_dir.display()
                ));
                ui.label(format!(
                    "Cache: {}",
                    self.shell_state.runtime_paths.cache_dir.display()
                ));
            });

        if self.shell_state.permission_panel_open {
            eframe::egui::SidePanel::right("permissions")
                .default_width(260.0)
                .show(ctx, |ui| {
                    ui.heading("Capability Grants");
                    for (capability, decision) in &self.shell_state.capabilities_snapshot {
                        ui.horizontal(|ui| {
                            ui.monospace(capability);
                            ui.label(decision);
                        });
                    }
                });
        }

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Browser Backend View");
            ui.separator();
            ui.label(format!("Engine state: {}", self.shell_state.engine_status));
            ui.label("This viewport is reserved for Servo-backed rendering surfaces.");
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.label("Current target");
                ui.monospace(&self.shell_state.active_tab.current_url);
            });
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.label("Future dimensions");
                ui.label("Permissions, automation, cache introspection, article workflows, and local-tool routing hang off this shell.");
            });
        });

        if self.shell_state.log_panel_open {
            eframe::egui::TopBottomPanel::bottom("log_panel")
                .resizable(true)
                .default_height(180.0)
                .show(ctx, |ui| {
                    ui.heading("Startup and Command Log");
                    eframe::egui::ScrollArea::vertical().show(ui, |ui| {
                        for event in self.shell_state.event_log.iter().rev().take(128) {
                            ui.monospace(event);
                        }
                    });
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{BrowserEngine, BrowserTab, EngineEvent, EngineStatus};

    struct MockEngine {
        status: EngineStatus,
        tab: BrowserTab,
        events: Vec<EngineEvent>,
    }

    impl BrowserEngine for MockEngine {
        fn backend_name(&self) -> &'static str {
            "mock"
        }

        fn status(&self) -> EngineStatus {
            self.status.clone()
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

        fn set_render_surface(&mut self, _metadata: RenderSurfaceMetadata) {}

        fn take_events(&mut self) -> Vec<EngineEvent> {
            std::mem::take(&mut self.events)
        }
    }

    #[test]
    fn shell_state_sync_handles_ready_and_error_statuses() {
        let paths = RuntimePaths {
            config_path: "brazen.toml".into(),
            data_dir: "data".into(),
            logs_dir: "logs".into(),
            profiles_dir: "profiles".into(),
            cache_dir: "cache".into(),
        };
        let mut shell = ShellState {
            app_name: "Brazen".to_string(),
            backend_name: "mock".to_string(),
            engine_status: EngineStatus::Initializing,
            active_tab: BrowserTab {
                id: 1,
                title: "Loading".to_string(),
                current_url: "about:blank".to_string(),
            },
            address_bar_input: "https://example.com".to_string(),
            event_log: Vec::new(),
            log_panel_open: true,
            permission_panel_open: false,
            capabilities_snapshot: Vec::new(),
            runtime_paths: paths,
        };

        let mut ready_engine = MockEngine {
            status: EngineStatus::Ready,
            tab: BrowserTab {
                id: 1,
                title: "Example".to_string(),
                current_url: "https://example.com".to_string(),
            },
            events: vec![EngineEvent::StatusChanged(EngineStatus::Ready)],
        };
        shell.sync_from_engine(&mut ready_engine);
        assert_eq!(shell.engine_status, EngineStatus::Ready);
        assert!(
            shell
                .event_log
                .iter()
                .any(|line| line.contains("StatusChanged"))
        );

        let mut failing_engine = MockEngine {
            status: EngineStatus::Error("boot failed".to_string()),
            tab: shell.active_tab.clone(),
            events: vec![EngineEvent::StatusChanged(EngineStatus::Error(
                "boot failed".to_string(),
            ))],
        };
        shell.sync_from_engine(&mut failing_engine);
        assert_eq!(
            shell.engine_status,
            EngineStatus::Error("boot failed".to_string())
        );
    }
}
