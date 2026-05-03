# Brazen

Brazen is a Rust browser-platform skeleton built around an `egui`/`eframe` shell and a future Servo-backed content engine.

## Intent

Establish the durable platform layer first: configuration, permissions, runtime wiring, diagnostics, session handling, and the seams needed to swap rendering backends without destabilizing the rest of the app.

## Ambition

Inferred from the Servo integration modules, MCP support, automation hooks, and platform abstractions, the project is aiming beyond a toy browser toward a programmable desktop browsing/runtime environment.

## Current Status

The shell, configuration system, logging, automation scaffolding, profile handling, and engine abstraction are present. The rendering/backend side is still explicitly in-progress.

## Core Capabilities Or Focus Areas

- Desktop app shell with runtime configuration.
- Capability-oriented permission and session model.
- Automation, introspection, and MCP-related integration seams.
- Servo-oriented engine abstraction and supporting resources.
- Tests and scripts for platform bring-up and diagnostics.

## Project Layout

- `config/`: checked-in runtime configuration and configuration examples.
- `docs/`: project documentation, reference material, and roadmap notes.
- `examples/`: sample inputs, example configs, or demonstration workflows.
- `profiles/`: runtime profile data or persisted profile-specific resources.
- `scratch/`: working notes or experimental assets that support ongoing development.
- `scripts/`: helper scripts for development, validation, or release workflows.
- `src/`: Rust source for the main crate or application entrypoint.
- `tests/`: automated tests, fixtures, or parity scenarios.
- `Cargo.toml`: crate or workspace manifest and the first place to check for package structure.

## Setup And Requirements

- Rust toolchain.
- Any system dependencies required by `eframe` and the selected rendering stack.
- Servo-related assets or submodules when working on engine integration.

## Build / Run / Test Commands

```bash
cargo build
cargo test
cargo run
```

## Notes, Limitations, Or Known Gaps

- Several files in `src/` show active merge or recovery artifacts (`.orig`, `.rej`, `.bak`), so the codebase is still under active restructuring.
- Backend evolution is a core part of the project rather than an edge case.

## Next Steps Or Roadmap Hints

- Continue tightening the engine seam so rendering work can land without destabilizing the platform layer.
- Promote the current diagnostics and automation hooks into documented development workflows.
