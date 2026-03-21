# Servo Live Integration Roadmap

Focuses on turning the current Servo scaffold into an actual embedded renderer that loads and displays pages.

## Build And Sources

- [x] Pin a specific Servo source revision with a reproducible fetch script
- [x] Add Servo as a git dependency or workspace submodule
- [x] Document platform-specific build prerequisites for the pinned revision
- [x] Establish a dedicated build profile for Servo artifacts
- [x] Add CI target that builds Servo in the same environment

## Embedder Runtime

- [x] Create a Servo embedder crate/module wired to the pinned source
- [x] Implement real init/shutdown and error propagation
- [x] Initialize Servo’s renderer and compositor
- [x] Allocate a render surface compatible with `egui` textures
- [x] Upload rendered frames to the `egui` surface each frame
- [x] Implement window resize handling with framebuffer reallocation

## Input And Event Loop

- [x] Forward mouse/pointer events to Servo correctly
- [x] Forward keyboard events to Servo correctly
- [x] Forward scroll/zoom events to Servo correctly
- [x] Wire IME composition into Servo text input
- [x] Add focus/blur events for window activation
- [x] Integrate Servo’s event loop with `eframe`’s update cadence

## Navigation And State

- [x] Translate Servo navigation events into shell events
- [x] Update title/favicon from Servo
- [x] Wire back/forward stack to Servo history
- [x] Implement reload/stop commands
- [x] Surface load progress updates

## Diagnostics And Debugging

- [x] Add a render debug overlay in the shell
- [x] Capture Servo logs and pipe into `tracing`
- [x] Add a runtime toggle for verbose Servo logging
- [x] Implement a minimal devtools transport for local use

## Stability

- [x] Crash detection with retry/backoff
- [x] Persist crash dumps with session context
- [x] Memory and GPU resource cleanup on shutdown
