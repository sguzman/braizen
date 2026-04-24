import re

with open('src/app/mod.rs', 'r') as f:
    mod_rs = f.read()

# Find forward_input_events
start_idx = mod_rs.find("    fn forward_input_events(&mut self, ctx: &eframe::egui::Context) {")
if start_idx != -1:
    brace_count = 0
    end_idx = -1
    for i in range(start_idx, len(mod_rs)):
        if mod_rs[i] == '{': brace_count += 1
        elif mod_rs[i] == '}':
            brace_count -= 1
            if brace_count == 0:
                end_idx = i + 1
                break
    
    input_fn = mod_rs[start_idx:end_idx]
    # change from `fn forward_input_events` to `pub(super) fn forward_input_events`
    input_fn = input_fn.replace("fn forward_input_events", "pub(super) fn forward_input_events")
    
    # Write to input.rs
    with open('src/app/input.rs', 'w') as f:
        f.write("use super::*;\nuse crate::engine::{InputEvent, AlphaMode};\n\nimpl BrazenApp {\n" + input_fn + "\n}\n")
        
    # Remove from mod.rs
    mod_rs = mod_rs[:start_idx] + mod_rs[end_idx:]

with open('src/app/mod.rs', 'w') as f:
    f.write(mod_rs)
