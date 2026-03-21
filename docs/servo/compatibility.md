# Servo Compatibility And Hardening

## Media Playback

- Define minimum media formats and codecs to support.
- Establish playback policy for autoplay and user gesture requirements.

## Audio Output

- Default to the system device.
- Allow explicit device selection when supported.

## Clipboard

- Clipboard reads are gated by capability grants.
- Writes must respect user-initiated actions.

## Downloads

- Engine emits download requests with suggested paths.
- Shell owns final path selection and confirmation UI.

## Cookies And Storage

- Storage is profile-scoped by default.
- Explicitly bind storage to the active profile context.

## Service Workers

- Align service-worker persistence with cache policy.
- Expose policy overrides in config when needed.

## Security Warnings

- Surface mixed-content warnings and TLS errors in the shell.
- Provide a clear override UX with audit logging.
