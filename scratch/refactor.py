import re
import sys

def extract_method(content, method_name, is_impl_trait=False):
    # Find the start of the method
    pattern = rf"(\s*(?:pub |pub\(super\) )?fn {method_name}\s*\(.*?\)\s*{{)"
    if is_impl_trait:
        pattern = rf"(\s*impl.*for.*{{\s*fn {method_name}\s*\(.*?\)\s*{{)"
        # Actually it's simpler: find "impl eframe::App for BrazenApp"
        
    start_match = re.search(pattern, content, re.DOTALL)
    if not start_match:
        return None, content
        
    # We need a proper brace matcher
    start_idx = start_match.start()
    
    # Simple brace matching
    brace_count = 0
    in_string = False
    in_char = False
    escape = False
    
    for i in range(start_idx, len(content)):
        char = content[i]
        
        if escape:
            escape = False
            continue
            
        if char == '\\':
            escape = True
            continue
            
        if char == '"' and not in_char:
            in_string = not in_string
            continue
            
        if char == "'" and not in_string:
            in_char = not in_char
            continue
            
        if not in_string and not in_char:
            if char == '{':
                brace_count += 1
            elif char == '}':
                brace_count -= 1
                if brace_count == 0:
                    end_idx = i + 1
                    method_content = content[start_idx:end_idx]
                    new_content = content[:start_idx] + content[end_idx:]
                    return method_content, new_content
                    
    return None, content

with open('src/app.rs', 'r') as f:
    app_rs = f.read()

# We need to extract these from app.rs and put into ui_components.rs
methods_to_ui_comp = [
    "render_top_menu",
    "render_command_palette",
    "render_context_menu"
]

extracted_ui_comps = []
for m in methods_to_ui_comp:
    method_code, app_rs = extract_method(app_rs, m)
    if method_code:
        # ensure it is 'pub fn'
        method_code = re.sub(r'^\s*fn ' + m, f'    pub fn {m}', method_code, flags=re.MULTILINE)
        extracted_ui_comps.append(method_code)

if extracted_ui_comps:
    with open('src/app/ui_components.rs', 'r') as f:
        ui_comp = f.read()
    
    # insert before the last closing brace
    last_brace = ui_comp.rfind('}')
    ui_comp = ui_comp[:last_brace] + "\n" + "\n\n".join(extracted_ui_comps) + "\n}\n"
    
    with open('src/app/ui_components.rs', 'w') as f:
        f.write(ui_comp)

# Extract eframe::App and Drop for BrazenApp
# Instead of precise matching, let's just use regex for the whole block if possible, or string split
# `impl eframe::App for BrazenApp`
start_eframe = app_rs.find("impl eframe::App for BrazenApp {")
if start_eframe != -1:
    brace_count = 0
    end_eframe = -1
    for i in range(start_eframe, len(app_rs)):
        if app_rs[i] == '{': brace_count += 1
        elif app_rs[i] == '}':
            brace_count -= 1
            if brace_count == 0:
                end_eframe = i + 1
                break
    
    eframe_block = app_rs[start_eframe:end_eframe]
    app_rs = app_rs[:start_eframe] + app_rs[end_eframe:]
else:
    eframe_block = ""

# `impl Drop for BrazenApp`
start_drop = app_rs.find("impl Drop for BrazenApp {")
if start_drop != -1:
    brace_count = 0
    end_drop = -1
    for i in range(start_drop, len(app_rs)):
        if app_rs[i] == '{': brace_count += 1
        elif app_rs[i] == '}':
            brace_count -= 1
            if brace_count == 0:
                end_drop = i + 1
                break
    
    drop_block = app_rs[start_drop:end_drop]
    app_rs = app_rs[:start_drop] + app_rs[end_drop:]
else:
    drop_block = ""

# Also extract ensure_engine_initialized
ensure_engine_code, app_rs = extract_method(app_rs, "ensure_engine_initialized")

# Extract empty_to_none
empty_to_none_code, app_rs = extract_method(app_rs, "empty_to_none")

# Extract frame_average_ms
frame_avg_code, app_rs = extract_method(app_rs, "frame_average_ms")

# Write ui_main.rs
ui_main_content = f"""use super::state::*;
use crate::engine::RenderSurfaceMetadata;
use crate::navigation::normalize_url_input;
use std::collections::VecDeque;

impl super::BrazenApp {{
{ensure_engine_code if ensure_engine_code else ''}
}}

{empty_to_none_code if empty_to_none_code else ''}

{frame_avg_code if frame_avg_code else ''}

{eframe_block.replace("impl eframe::App for BrazenApp", "impl eframe::App for super::BrazenApp")}

{drop_block.replace("impl Drop for BrazenApp", "impl Drop for super::BrazenApp")}
"""

with open('src/app/ui_main.rs', 'w') as f:
    f.write(ui_main_content)

# We need to remove the already-copied methods from app.rs!
# The ones that were copied to panels.rs, secondary_ui.rs, ui_components.rs
methods_to_remove = [
    "render_workspace_settings",
    "render_bookmarks_panel",
    "render_history_panel",
    "render_bottom_panel",
    "render_log_tab",
    "render_health_tab",
    "render_downloads_tab",
    "render_dom_tab",
    "render_network_tab",
    "render_capabilities_tab",
    "render_automation_tab",
    "render_knowledge_graph_tab",
    "render_reading_queue_panel",
    "render_reader_mode_panel",
    "render_tts_controls_panel",
    "render_browser_view",
    "render_header",
    "render_left_sidebar",
    "render_right_sidebar",
    "render_dashboard",
    "render_resources_content",
    "render_telemetry_content",
    "render_terminal_content",
    "render_cache_panel",
    "monitor_chatgpt_mcp",
]

for m in methods_to_remove:
    _, app_rs = extract_method(app_rs, m)

with open('src/app.rs', 'w') as f:
    f.write(app_rs)

print("Refactoring script completed")
