with open('src/app/state.rs', 'r') as f:
    state_rs = f.read()

imports = """
use chrono::Utc;
use crate::session::{NavigationEntry, SessionSnapshot, load_session};
use crate::engine::EngineLoadStatus;
use crate::profile_db::ProfileDb;
use std::time::Instant;
"""

state_rs = imports + state_rs

with open('src/app/state.rs', 'w') as f:
    f.write(state_rs)
