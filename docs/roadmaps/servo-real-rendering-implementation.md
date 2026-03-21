# Servo Real Rendering Implementation Roadmap

Focuses on replacing the current stub renderer with actual Servo output. This is the “make it render pages” plan.

## Servo Runtime Integration

- [ ] Add Servo workspace path dependencies behind a `servo-upstream` feature
- [ ] Introduce a `servo_upstream` module that wraps Servo types and embedder traits
- [ ] Implement an `EventLoopWaker` that integrates with `eframe`’s repaint cadence
- [ ] Build a minimal Servo `Embedder` implementation that can receive `EmbedderMsg`
- [ ] Wire Servo logging to `tracing` with per-target filtering

## Rendering Pipeline

- [ ] Initialize WebRender and compositor with the chosen backend
- [ ] Create a render surface compatible with egui (CPU readback path)
- [ ] Map Servo’s rendered frame into `egui::ColorImage`
- [ ] Add a render loop that drains Servo paint messages each frame
- [ ] Handle viewport resize and reallocate WebRender surfaces
- [ ] Add explicit metrics for frame upload cost

## Navigation + Metadata

- [ ] Instantiate a real Servo browser instance and WebView
- [ ] Wire `navigate/reload/stop` to Servo’s API
- [ ] Translate Servo navigation events into `EngineEvent`
- [ ] Update title, URL, and favicon from Servo metadata
- [ ] Surface load progress / document ready from Servo

## Input + Focus

- [ ] Translate pointer events into Servo embedder input
- [ ] Translate keyboard + modifiers into Servo input
- [ ] Translate scroll/zoom events into Servo input
- [ ] Wire IME composition to Servo
- [ ] Bridge clipboard read/write requests

## Validation

- [ ] Load `about:blank` without crashing and draw a frame
- [ ] Load a basic HTTP URL and show real content
- [ ] Add a smoke test that boots Servo and renders one real frame
