use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationEntry {
    pub url: String,
    pub title: String,
    pub timestamp: String,
    pub redirect_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingNavigation {
    pub url: String,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabLineage {
    pub created_from: Option<TabId>,
    pub reopened_from: Option<TabId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub zoom_level: f32,
    pub pending: Option<PendingNavigation>,
    pub back_stack: Vec<NavigationEntry>,
    pub forward_stack: Vec<NavigationEntry>,
    pub history: Vec<NavigationEntry>,
    pub pinned: bool,
    pub muted: bool,
    pub closed: bool,
    pub focused_element: Option<String>,
    pub selection_text: Option<String>,
    pub downloads: Vec<String>,
    pub permission_grants: Vec<String>,
    pub lineage: TabLineage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub id: WindowId,
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub version: u32,
    pub session_id: SessionId,
    pub profile_id: String,
    pub created_at: String,
    pub windows: Vec<WindowState>,
    pub active_window: usize,
    pub crash_recovery_pending: bool,
}

impl SessionSnapshot {
    pub fn new(profile_id: String, now: String) -> Self {
        let tab_id = TabId(Uuid::new_v4());
        let tab = TabState {
            id: tab_id,
            title: "New Tab".to_string(),
            url: "about:blank".to_string(),
            zoom_level: 1.0,
            pending: None,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            history: Vec::new(),
            pinned: false,
            muted: false,
            closed: false,
            focused_element: None,
            selection_text: None,
            downloads: Vec::new(),
            permission_grants: Vec::new(),
            lineage: TabLineage {
                created_from: None,
                reopened_from: None,
            },
        };
        let window = WindowState {
            id: WindowId(Uuid::new_v4()),
            tabs: vec![tab],
            active_tab: 0,
        };

        Self {
            version: 1,
            session_id: SessionId(Uuid::new_v4()),
            profile_id,
            created_at: now,
            windows: vec![window],
            active_window: 0,
            crash_recovery_pending: false,
        }
    }

    pub fn active_window_mut(&mut self) -> &mut WindowState {
        &mut self.windows[self.active_window]
    }

    pub fn active_tab(&self) -> Option<&TabState> {
        self.windows
            .get(self.active_window)
            .and_then(|window| window.tabs.get(window.active_tab))
    }

    pub fn active_tab_mut(&mut self) -> &mut TabState {
        let window = &mut self.windows[self.active_window];
        &mut window.tabs[window.active_tab]
    }

    pub fn set_active_tab(&mut self, index: usize) {
        if let Some(window) = self.windows.get_mut(self.active_window)
            && index < window.tabs.len()
        {
            window.active_tab = index;
        }
    }

    pub fn open_new_tab(&mut self, url: &str, title: &str) {
        let lineage = TabLineage {
            created_from: Some(self.active_tab_mut().id.clone()),
            reopened_from: None,
        };
        let tab = TabState {
            id: TabId(Uuid::new_v4()),
            title: title.to_string(),
            url: url.to_string(),
            zoom_level: 1.0,
            pending: None,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            history: Vec::new(),
            pinned: false,
            muted: false,
            closed: false,
            focused_element: None,
            selection_text: None,
            downloads: Vec::new(),
            permission_grants: Vec::new(),
            lineage,
        };
        let window = self.active_window_mut();
        window.tabs.push(tab);
        window.active_tab = window.tabs.len() - 1;
    }

    pub fn duplicate_active_tab(&mut self) {
        let active = self.active_tab_mut().clone();
        let mut duplicate = active.clone();
        duplicate.id = TabId(Uuid::new_v4());
        duplicate.lineage = TabLineage {
            created_from: Some(active.id),
            reopened_from: None,
        };
        let window = self.active_window_mut();
        window.tabs.push(duplicate);
        window.active_tab = window.tabs.len() - 1;
    }

    pub fn close_active_tab(&mut self) {
        let window = self.active_window_mut();
        if window.tabs.len() <= 1 {
            return;
        }
        window.tabs.remove(window.active_tab);
        if window.active_tab >= window.tabs.len() {
            window.active_tab = window.tabs.len().saturating_sub(1);
        }
    }

    pub fn toggle_pin_active_tab(&mut self) {
        let tab = self.active_tab_mut();
        tab.pinned = !tab.pinned;
    }

    pub fn toggle_mute_active_tab(&mut self) {
        let tab = self.active_tab_mut();
        tab.muted = !tab.muted;
    }

    pub fn mark_pending_navigation(&mut self, url: &str, now: String) {
        let tab = self.active_tab_mut();
        tab.pending = Some(PendingNavigation {
            url: url.to_string(),
            started_at: now,
        });
    }

    pub fn commit_navigation(&mut self, entry: NavigationEntry) {
        let tab = self.active_tab_mut();
        if tab.url != entry.url {
            if !tab.url.is_empty() {
                tab.back_stack.push(NavigationEntry {
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                    timestamp: entry.timestamp.clone(),
                    redirect_chain: Vec::new(),
                });
            }
            tab.forward_stack.clear();
        }
        tab.url = entry.url.clone();
        tab.title = entry.title.clone();
        tab.history.push(entry);
        tab.pending = None;
    }

    pub fn go_back(&mut self, now: String) {
        let tab = self.active_tab_mut();
        if let Some(entry) = tab.back_stack.pop() {
            tab.forward_stack.push(NavigationEntry {
                url: tab.url.clone(),
                title: tab.title.clone(),
                timestamp: now,
                redirect_chain: Vec::new(),
            });
            tab.url = entry.url;
            tab.title = entry.title;
        }
    }

    pub fn go_forward(&mut self, now: String) {
        let tab = self.active_tab_mut();
        if let Some(entry) = tab.forward_stack.pop() {
            tab.back_stack.push(NavigationEntry {
                url: tab.url.clone(),
                title: tab.title.clone(),
                timestamp: now,
                redirect_chain: Vec::new(),
            });
            tab.url = entry.url;
            tab.title = entry.title;
        }
    }
}

pub fn save_session(path: &Path, session: &SessionSnapshot) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec_pretty(session).map_err(std::io::Error::other)?;
    std::fs::write(path, data)
}

pub fn load_session(path: &Path) -> std::io::Result<SessionSnapshot> {
    let data = std::fs::read(path)?;
    let session = serde_json::from_slice(&data).map_err(std::io::Error::other)?;
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_navigation_updates_history() {
        let mut session = SessionSnapshot::new("default".to_string(), "now".to_string());
        session.commit_navigation(NavigationEntry {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            timestamp: "t1".to_string(),
            redirect_chain: Vec::new(),
        });
        assert_eq!(session.active_tab_mut().url, "https://example.com");
        assert_eq!(session.active_tab_mut().history.len(), 1);
    }

    #[test]
    fn pending_navigation_is_cleared_on_commit() {
        let mut session = SessionSnapshot::new("default".to_string(), "now".to_string());
        session.mark_pending_navigation("https://example.com", "t0".to_string());
        assert!(session.active_tab().unwrap().pending.is_some());
        session.commit_navigation(NavigationEntry {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            timestamp: "t1".to_string(),
            redirect_chain: vec!["https://example.com".to_string()],
        });
        assert!(session.active_tab().unwrap().pending.is_none());
    }

    #[test]
    fn back_and_forward_stacks_move_entries() {
        let mut session = SessionSnapshot::new("default".to_string(), "now".to_string());
        session.commit_navigation(NavigationEntry {
            url: "https://a.test".to_string(),
            title: "A".to_string(),
            timestamp: "t1".to_string(),
            redirect_chain: Vec::new(),
        });
        session.commit_navigation(NavigationEntry {
            url: "https://b.test".to_string(),
            title: "B".to_string(),
            timestamp: "t2".to_string(),
            redirect_chain: Vec::new(),
        });
        assert_eq!(session.active_tab().unwrap().url, "https://b.test");
        assert_eq!(session.active_tab().unwrap().back_stack.len(), 2);
        assert!(session.active_tab().unwrap().forward_stack.is_empty());

        session.go_back("t3".to_string());
        assert_eq!(session.active_tab().unwrap().url, "https://a.test");
        assert_eq!(session.active_tab().unwrap().forward_stack.len(), 1);

        session.go_forward("t4".to_string());
        assert_eq!(session.active_tab().unwrap().url, "https://b.test");
    }

    #[test]
    fn open_new_tab_sets_lineage_from_active() {
        let mut session = SessionSnapshot::new("default".to_string(), "now".to_string());
        let origin = session.active_tab().unwrap().id.clone();
        session.open_new_tab("https://example.com", "Example");
        let tab = session.active_tab().unwrap();
        assert_eq!(tab.url, "https://example.com");
        assert_eq!(tab.lineage.created_from.as_ref(), Some(&origin));
    }

    #[test]
    fn duplicate_tab_sets_lineage_and_new_id() {
        let mut session = SessionSnapshot::new("default".to_string(), "now".to_string());
        let origin = session.active_tab().unwrap().id.clone();
        session.duplicate_active_tab();
        let tab = session.active_tab().unwrap();
        assert_ne!(tab.id, origin);
        assert_eq!(tab.lineage.created_from.as_ref(), Some(&origin));
    }

    #[test]
    fn pin_mute_close_and_switch_behaviors() {
        let mut session = SessionSnapshot::new("default".to_string(), "now".to_string());
        session.open_new_tab("https://a.test", "A");
        session.open_new_tab("https://b.test", "B");
        assert_eq!(session.active_tab().unwrap().title, "B");

        session.set_active_tab(0);
        assert_eq!(session.active_tab().unwrap().title, "New Tab");
        session.toggle_pin_active_tab();
        session.toggle_mute_active_tab();
        assert!(session.active_tab().unwrap().pinned);
        assert!(session.active_tab().unwrap().muted);

        session.close_active_tab();
        assert_ne!(session.active_tab().unwrap().title, "New Tab");
    }

    #[test]
    fn session_json_persistence_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let mut session = SessionSnapshot::new("p".to_string(), "now".to_string());
        session.crash_recovery_pending = true;
        session.open_new_tab("https://example.com", "Example");
        save_session(&path, &session).unwrap();
        let loaded = load_session(&path).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.profile_id, "p");
        assert!(loaded.crash_recovery_pending);
        assert!(loaded.windows[0].tabs.len() >= 2);
    }
}
