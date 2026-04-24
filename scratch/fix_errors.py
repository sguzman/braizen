import re

# Fix panels.rs
with open('src/app/panels.rs', 'r') as f:
    panels_rs = f.read()

panels_rs = panels_rs.replace("crate::app::frame_average_ms", "crate::app::ui_main::frame_average_ms")

with open('src/app/panels.rs', 'w') as f:
    f.write(panels_rs)


# Fix ui_components.rs
with open('src/app/ui_components.rs', 'r') as f:
    ui_comp = f.read()

ui_comp = ui_comp.replace("use super::state::*;", "use super::state::*;\nuse crate::app::PaletteCommand;\nuse crate::engine::EngineStatus;")

with open('src/app/ui_components.rs', 'w') as f:
    f.write(ui_comp)

# Fix monitor_chatgpt_mcp in ui_main.rs
# I'll just append it to impl BrazenApp in ui_main.rs
monitor_code = """
    pub(super) fn monitor_chatgpt_mcp(&mut self) {
        if !self.shell_state.observe_dom || !self.shell_state.control_terminal {
            return;
        }
        
        let Some(snapshot) = &self.shell_state.dom_snapshot else { return; };
        
        // Use scraper to find <client mcp="terminal">...</client>
        let fragment = scraper::Html::parse_fragment(snapshot);
        let selector = scraper::Selector::parse("client[mcp=\\"terminal\\"]").unwrap();
        
        for element in fragment.select(&selector) {
            let command = element.text().collect::<String>();
            let command = command.trim();
            if !command.is_empty() && !self.processed_mcp_commands.contains(command) {
                tracing::info!(target: "brazen::automation", command = %command, "Found new MCP terminal command in DOM");
                
                // Run command
                if let Some(tx) = &self.terminal_tx {
                    let _ = tx.send(command.to_string());
                }
                
                // Mark as processed
                self.processed_mcp_commands.insert(command.to_string());
            }
        }
    }
"""

with open('src/app/ui_main.rs', 'r') as f:
    ui_main_rs = f.read()

# insert before last } of impl BrazenApp
last_brace = ui_main_rs.find("impl eframe::App")
if last_brace != -1:
    # go back to find the } of impl BrazenApp
    idx = ui_main_rs.rfind("}", 0, last_brace)
    ui_main_rs = ui_main_rs[:idx] + monitor_code + ui_main_rs[idx:]

with open('src/app/ui_main.rs', 'w') as f:
    f.write(ui_main_rs)

print("Errors fixed")
