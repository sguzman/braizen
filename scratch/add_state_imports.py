with open('src/app/state.rs', 'r') as f:
    state_rs = f.read()

imports = """
use crate::engine::{BrowserEngine, EngineEvent, EngineStatus, FocusState, RenderSurfaceMetadata, SecurityWarningKind};
use crate::extraction::extract_entities;
use std::time::Duration;
use std::sync::{Arc, RwLock};
"""

state_rs = imports + state_rs

with open('src/app/state.rs', 'w') as f:
    f.write(state_rs)
