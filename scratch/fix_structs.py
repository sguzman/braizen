import re

with open('src/app/mod.rs', 'r') as f:
    mod_rs = f.read()

# Delete duplicate structs/enums from mod.rs
structs_to_delete = [
    "WorkspaceLayout",
    "UiTheme",
    "UiDensity",
    "LayoutPreset",
    "SettingsTab",
    "LeftPanelTab",
    "WorkspacePanels",
    "ReadingQueueItem",
    "ExtractedEntity"
]

for s in structs_to_delete:
    # regex to delete `enum X { ... }` or `struct X { ... }`
    mod_rs = re.sub(r'#\[derive\([^)]*\)\]\s*(pub )?(enum|struct)\s+' + s + r'\s*\{[^}]*\}', '', mod_rs, flags=re.MULTILINE)
    # also remove `impl Default for WorkspacePanels`
    if s == "WorkspacePanels":
        mod_rs = re.sub(r'impl Default for WorkspacePanels\s*\{.*?(?=\n\n|\Z)', '', mod_rs, flags=re.MULTILINE|re.DOTALL)

# Find DiagnosticTab and move it
diag_tab_match = re.search(r'#\[derive\([^)]*\)\]\s*(pub )?enum DiagnosticTab\s*\{[^}]*\}', mod_rs)
diag_tab_str = ""
if diag_tab_match:
    diag_tab_str = diag_tab_match.group(0)
    # make it pub
    diag_tab_str = diag_tab_str.replace("enum DiagnosticTab", "pub enum DiagnosticTab")
    mod_rs = mod_rs[:diag_tab_match.start()] + mod_rs[diag_tab_match.end():]

# Make PaletteCommand pub
mod_rs = mod_rs.replace("enum PaletteCommand", "pub enum PaletteCommand")

with open('src/app/mod.rs', 'w') as f:
    f.write(mod_rs)

with open('src/app/state.rs', 'r') as f:
    state_rs = f.read()

# Remove `use super::DiagnosticTab;` from state.rs
state_rs = state_rs.replace("use super::DiagnosticTab;", "")

# add DiagnosticTab
if diag_tab_str:
    state_rs += "\n" + diag_tab_str + "\n"

with open('src/app/state.rs', 'w') as f:
    f.write(state_rs)

print("Fixed structs in mod.rs and state.rs")
