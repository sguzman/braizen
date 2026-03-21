# Servo Source Pin

- Tag: `v0.0.4`
- Revision: `b73ae02`

Brazen uses a pinned Servo source checkout rather than the crates.io package. The repository is tracked as a submodule at `vendor/servo`, and the helper script `scripts/fetch_servo.sh` can clone the pinned tag into `vendor/servo` by default, optionally checking out `BRAZEN_SERVO_REV` when provided.
