# Brazen Roadmap

This roadmap is grouped by feature dimension. Boxes are checked only for capabilities that are implemented in the repo today.

## Shell / Workspace UX

- [x] Desktop shell boots through `eframe` / `egui`
- [x] Single-tab workspace model exists
- [x] Address bar and command controls exist
- [x] Backend status and placeholder content viewport exist
- [x] Log panel and permission panel exist
- [ ] Multi-tab workspace management
- [ ] Bookmarks, history, and downloads UX
- [ ] Keyboard shortcuts and command palette

## Engine Integration

- [x] `BrowserEngine` trait defines the internal engine seam
- [x] Null backend exists for default builds
- [x] Feature-gated Servo scaffold backend exists
- [ ] Real Servo render-surface embedding
- [ ] Input, focus, IME, and popup forwarding
- [ ] Multi-process engine isolation strategy

## Navigation / Session Model

- [x] Navigation command routing exists
- [x] Active tab URL/title state exists
- [x] Engine status events are surfaced into shell state
- [ ] Session restore
- [ ] History stack and navigation actions
- [ ] Window and profile lifecycle management

## Permissions / Capabilities

- [x] Capability-oriented permission model exists
- [x] Default capability grants are configurable in TOML
- [x] Permission state is visible in the shell
- [ ] Runtime permission prompts
- [ ] Grant persistence and revocation UX
- [ ] Fine-grained origin/session scoping

## Automation / API

- [x] Automation endpoint configuration exists
- [ ] Local WebSocket automation server
- [ ] Tab/query/control API
- [ ] DOM and cache query surfaces
- [ ] Authenticated local client policy

## Cache / Asset Plane

- [x] Cache policy configuration surface exists
- [ ] Asset metadata capture
- [ ] Selective response-body capture
- [ ] Archive replay mode
- [ ] Cache inspector UI

## Knowledge / Article Workflows

- [x] Extraction and ontology feature flags exist
- [ ] Article extraction pipeline
- [ ] Save-for-later queue
- [ ] Reading stats and revisit lineage
- [ ] RSS rehydration from browsing data

## Media / TTS

- [x] TTS config surface exists
- [ ] Playback queue
- [ ] Multi-provider TTS integration
- [ ] Reader-mode autoplay rules

## Persistence / Profile Management

- [x] Platform config path resolution exists
- [x] Platform data/logs/cache/profile roots are resolved
- [x] Default config is generated on first run
- [ ] Structured browser-state persistence beyond config/logs
- [ ] Profile switching UX

## Observability

- [x] Tracing bootstrap exists
- [x] Console and rolling file log plans are configurable
- [x] Startup and command activity are logged
- [x] Engine-scaffold tracing exists behind feature flag
- [ ] Metrics and long-running diagnostics panels

## Quality / Testing

- [x] Config parsing and validation tests exist
- [x] Path resolution tests exist
- [x] Command dispatch tests exist
- [x] Engine-state synchronization tests exist
- [x] Bootstrap integration tests exist
- [ ] GUI interaction smoke tests
- [ ] Servo-enabled integration checks on a prepared machine
