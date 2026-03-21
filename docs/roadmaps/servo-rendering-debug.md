# Servo Rendering Debug Roadmap

Focuses on eliminating the current psychedelic render output and establishing a correct, stable pixel pipeline.

## Pixel Format + Color Space

- [x] Confirm Servo readback pixel format (RGBA/BGRA/ARGB) and document it
- [x] Add a runtime pixel-format probe using a known solid-color test frame
- [x] Validate sRGB vs linear conversion expectations for the readback buffer
- [x] Verify alpha premultiplication and adjust if egui expects straight alpha
- [x] Add an explicit pixel format enum to the upstream bridge for clarity

## Surface + Readback Integrity

- [x] Validate render surface dimensions against the egui viewport every frame
- [x] Confirm stride/row alignment from Servo readback and handle padding
- [x] Ensure the readback rect uses correct origin and size (no off-by-one)
- [x] Guard against zero-sized surfaces and skip readback cleanly
- [x] Add a frame checksum to detect repeated or stale buffers

## Pipeline Wiring

- [ ] Verify WebRender pipeline ID and document lifecycle timing
- [x] Confirm we are draining paint messages before readback
- [x] Add a trace span around readback → upload with byte counts
- [x] Add a debug toggle to bypass color conversion for A/B comparison
- [x] Add a single-frame capture to disk (png) for offline inspection

## Input/Viewport Correlation

- [x] Log viewport scale, device pixel ratio, and physical size each resize
- [x] Verify scale factor usage matches Servo’s device pixel ratio expectations
- [x] Validate pointer coordinates with a hit-test overlay
- [x] Confirm scroll delta units match Servo’s expected units

## Validation + Tests

- [x] Add a regression test that renders a known gradient and checks sample pixels
- [ ] Add a basic screenshot comparison test for about:blank
- [x] Add a manual validation checklist to the README for visual sanity checks
