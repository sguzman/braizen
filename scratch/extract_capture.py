import re

with open('src/app/mod.rs', 'r') as f:
    mod_rs = f.read()

funcs_to_extract = [
    "fn update_render_frame",
    "fn capture_frame_to_disk",
    "fn save_snapshot_to_disk",
    "fn resolve_capture_dir",
    "fn sample_pixel_rgba"
]

extracted_funcs = []

for func in funcs_to_extract:
    start_idx = mod_rs.find(f"    {func}")
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
        
        extracted = mod_rs[start_idx:end_idx]
        extracted = extracted.replace("fn ", "pub(super) fn ")
        extracted_funcs.append(extracted)
        mod_rs = mod_rs[:start_idx] + mod_rs[end_idx:]

with open('src/app/capture.rs', 'w') as f:
    f.write("use super::*;\nuse crate::engine::{RenderSurfaceMetadata};\nuse eframe::egui::ColorImage;\n\nimpl BrazenApp {\n" + "\n\n".join(extracted_funcs) + "\n}\n")

with open('src/app/mod.rs', 'w') as f:
    f.write(mod_rs)
