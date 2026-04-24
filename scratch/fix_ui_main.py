with open('src/app/ui_main.rs', 'r') as f:
    ui_main = f.read()

# Fix frame_average_ms
ui_main = ui_main.replace("""fn frame_average_ms(times: &VecDeque<f32>) -> Option<f32> {
    if times.is_empty() {
        return None;
    }
    let sum: f32 = times.iter().copied().sum();
    Some(sum / times.len() as f32)

    pub(super) fn monitor_chatgpt_mcp(&mut self) {""", """pub(super) fn frame_average_ms(times: &VecDeque<f32>) -> Option<f32> {
    if times.is_empty() {
        return None;
    }
    let sum: f32 = times.iter().copied().sum();
    Some(sum / times.len() as f32)
}

impl super::BrazenApp {
    pub(super) fn monitor_chatgpt_mcp(&mut self) {""")

with open('src/app/ui_main.rs', 'w') as f:
    f.write(ui_main)

