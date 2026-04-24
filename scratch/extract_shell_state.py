import re

with open('src/app/mod.rs', 'r') as f:
    mod_rs = f.read()

# Extract ShellState struct and impl
struct_match = re.search(r'#\[derive\([^)]*\)\]\s*pub struct ShellState \{.*?\}', mod_rs, flags=re.DOTALL)
impl_match = re.search(r'impl ShellState \{.*?^\}', mod_rs, flags=re.MULTILINE|re.DOTALL)

struct_str = struct_match.group(0) if struct_match else ""
impl_str = impl_match.group(0) if impl_match else ""

if struct_str:
    mod_rs = mod_rs.replace(struct_str, "")
if impl_str:
    mod_rs = mod_rs.replace(impl_str, "")

with open('src/app/mod.rs', 'w') as f:
    f.write(mod_rs)

with open('src/app/state.rs', 'r') as f:
    state_rs = f.read()

state_rs += "\n\n" + struct_str + "\n\n" + impl_str

with open('src/app/state.rs', 'w') as f:
    f.write(state_rs)
