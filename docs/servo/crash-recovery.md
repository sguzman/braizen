# Servo Crash And Recovery

## Detection

- The engine emits `Crashed` events with a reason string.
- The shell records the crash and surfaces recovery controls.

## Crash Dumps

- Crash artifacts are stored in the configured crash-dumps directory.
- The shell writes a minimal crash log on first crash receipt.

## Recovery

- The shell can restart the engine and reattach the render surface.
- Future work will restore session state after restart.
