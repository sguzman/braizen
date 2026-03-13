use crate::app::ShellState;
use crate::engine::BrowserEngine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommand {
    NavigateTo(String),
    ReloadActiveTab,
    ToggleLogPanel,
    OpenPermissionPanel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    NavigationScheduled,
    ReloadScheduled,
    LogPanelVisibility(bool),
    PermissionPanelVisibility(bool),
}

pub fn dispatch_command(
    state: &mut ShellState,
    engine: &mut dyn BrowserEngine,
    command: AppCommand,
) -> CommandOutcome {
    match command {
        AppCommand::NavigateTo(url) => {
            state.address_bar_input = url.clone();
            state.record_event(format!("queued navigation to {url}"));
            engine.navigate(&url);
            CommandOutcome::NavigationScheduled
        }
        AppCommand::ReloadActiveTab => {
            state.record_event("queued reload for active tab");
            engine.reload();
            CommandOutcome::ReloadScheduled
        }
        AppCommand::ToggleLogPanel => {
            state.log_panel_open = !state.log_panel_open;
            state.record_event(format!(
                "log panel {}",
                if state.log_panel_open {
                    "opened"
                } else {
                    "closed"
                }
            ));
            CommandOutcome::LogPanelVisibility(state.log_panel_open)
        }
        AppCommand::OpenPermissionPanel => {
            state.permission_panel_open = true;
            state.record_event("permission panel opened");
            CommandOutcome::PermissionPanelVisibility(true)
        }
    }
}
