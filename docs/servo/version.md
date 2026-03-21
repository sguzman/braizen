# Servo Source Pin

- Tag: `v0.0.4`
- Revision: `b73ae02`

Brazen uses a pinned Servo source checkout rather than the crates.io package. The helper script `scripts/fetch_servo.sh` will clone the pinned tag into `vendor/servo` by default, and will optionally `git checkout` `BRAZEN_SERVO_REV` when provided.
