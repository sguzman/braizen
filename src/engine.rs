use std::fmt;

use crate::config::BrazenConfig;
use crate::platform_paths::RuntimePaths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineStatus {
    NoEngine,
    Initializing,
    Ready,
    Error(String),
}

impl fmt::Display for EngineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEngine => write!(f, "No engine"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Ready => write!(f, "Ready"),
            Self::Error(message) => write!(f, "Error: {message}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfaceMetadata {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub scale_factor_basis_points: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    StatusChanged(EngineStatus),
    NavigationRequested(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTab {
    pub id: u64,
    pub title: String,
    pub current_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    Unsupported(&'static str),
    Startup(String),
}

pub trait BrowserEngine: Send {
    fn backend_name(&self) -> &'static str;
    fn status(&self) -> EngineStatus;
    fn active_tab(&self) -> &BrowserTab;
    fn navigate(&mut self, url: &str);
    fn reload(&mut self);
    fn set_render_surface(&mut self, metadata: RenderSurfaceMetadata);
    fn take_events(&mut self) -> Vec<EngineEvent>;
}

pub trait EngineFactory {
    fn create(&self, config: &BrazenConfig, paths: &RuntimePaths) -> Box<dyn BrowserEngine>;
}

pub struct NullEngine {
    status: EngineStatus,
    active_tab: BrowserTab,
    events: Vec<EngineEvent>,
    _surface: Option<RenderSurfaceMetadata>,
}

impl NullEngine {
    pub fn new() -> Self {
        Self {
            status: EngineStatus::NoEngine,
            active_tab: BrowserTab {
                id: 1,
                title: "Platform Skeleton".to_string(),
                current_url: "about:blank".to_string(),
            },
            events: Vec::new(),
            _surface: None,
        }
    }
}

impl Default for NullEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserEngine for NullEngine {
    fn backend_name(&self) -> &'static str {
        "null"
    }

    fn status(&self) -> EngineStatus {
        self.status.clone()
    }

    fn active_tab(&self) -> &BrowserTab {
        &self.active_tab
    }

    fn navigate(&mut self, url: &str) {
        self.active_tab.current_url = url.to_string();
        self.events
            .push(EngineEvent::NavigationRequested(url.to_string()));
    }

    fn reload(&mut self) {
        self.events.push(EngineEvent::StatusChanged(self.status()));
    }

    fn set_render_surface(&mut self, metadata: RenderSurfaceMetadata) {
        self._surface = Some(metadata);
    }

    fn take_events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }
}

pub struct ServoEngineFactory;

impl EngineFactory for ServoEngineFactory {
    fn create(&self, _config: &BrazenConfig, _paths: &RuntimePaths) -> Box<dyn BrowserEngine> {
        #[cfg(feature = "servo")]
        {
            Box::new(ServoEngine::new())
        }

        #[cfg(not(feature = "servo"))]
        {
            Box::new(NullEngine::new())
        }
    }
}

#[cfg(feature = "servo")]
pub struct ServoEngine {
    status: EngineStatus,
    active_tab: BrowserTab,
    events: Vec<EngineEvent>,
    surface: Option<RenderSurfaceMetadata>,
}

#[cfg(feature = "servo")]
impl ServoEngine {
    pub fn new() -> Self {
        tracing::info!("servo feature enabled with scaffold backend");
        let events = vec![
            EngineEvent::StatusChanged(EngineStatus::Initializing),
            EngineEvent::StatusChanged(EngineStatus::Ready),
        ];

        Self {
            status: EngineStatus::Ready,
            active_tab: BrowserTab {
                id: 1,
                title: "Servo Scaffold".to_string(),
                current_url: "about:blank".to_string(),
            },
            events,
            surface: None,
        }
    }
}

#[cfg(feature = "servo")]
impl Default for ServoEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "servo")]
impl BrowserEngine for ServoEngine {
    fn backend_name(&self) -> &'static str {
        "servo-scaffold"
    }

    fn status(&self) -> EngineStatus {
        self.status.clone()
    }

    fn active_tab(&self) -> &BrowserTab {
        &self.active_tab
    }

    fn navigate(&mut self, url: &str) {
        tracing::info!(target: "brazen::engine::servo", %url, "servo scaffold navigate");
        self.active_tab.current_url = url.to_string();
        self.events
            .push(EngineEvent::NavigationRequested(url.to_string()));
    }

    fn reload(&mut self) {
        tracing::info!(target: "brazen::engine::servo", "servo scaffold reload");
    }

    fn set_render_surface(&mut self, metadata: RenderSurfaceMetadata) {
        tracing::debug!(
            target: "brazen::engine::servo",
            width = metadata.viewport_width,
            height = metadata.viewport_height,
            scale = metadata.scale_factor_basis_points,
            "updated render surface metadata"
        );
        self.surface = Some(metadata);
    }

    fn take_events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }
}
