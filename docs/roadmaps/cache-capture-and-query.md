# Cache Capture + Query Roadmap

Focuses on transparent, per-asset caching from real browsing, with clear visibility and query tooling.

## Capture Pipeline (Servo → AssetStore)

- [ ] Tap Servo network events to receive request/response metadata.
- [ ] Map Servo request IDs to tab/session IDs.
- [ ] Capture response headers and status codes for each asset.
- [ ] Normalize MIME types (content-type parsing + fallbacks).
- [ ] Record every request as a distinct asset entry.
- [x] Support empty-body assets (e.g., 204/304) with metadata only.
- [ ] Track request start/finish timestamps for duration metrics.
- [ ] Capture body bytes for HTML, CSS, JS, JSON, images, SVG, fonts, audio, video.
- [ ] Respect cache policy controls for third-party and authenticated assets.
- [ ] Enforce per-asset size caps with truncated flagging.
- [ ] Record redirection chain metadata as separate assets.
- [x] Deduplicate bodies by hash while preserving per-asset entries.
- [x] Persist request/response headers alongside metadata records.
- [x] Add explicit storage mode (memory/disk/archive) per asset record.
- [ ] Emit structured tracing for capture decisions and outcomes.

## Query + Visibility

- [x] Expose cache stats (entries, total bytes, unique blobs, capture ratio).
- [x] Add query filters for URL, MIME, session, tab, and status.
- [x] Add “asset detail” view (headers, timings, hash, storage path).
- [x] Add “recent assets” timeline view.
- [ ] Add search by content hash for dedupe visibility.
- [x] Surface cache state in the status panel (last capture, last error).

## CLI + Export

- [x] Extend `brazen cache` to print capture summary per asset.
- [x] Add `brazen cache --list` with filters (URL/MIME/session).
- [x] Add `brazen cache --show <asset_id|hash>` for full metadata.
- [x] Add export to JSONL and a compact summary report.
- [x] Add import/merge with collision handling.

## Policy + Config

- [x] Add config for capture-all vs selective modes.
- [x] Add explicit MIME allow/deny lists with glob support.
- [x] Add per-host capture policy overrides.
- [x] Add config for “store bodies always” vs “metadata-only”.
- [x] Add config for max total cache size with GC strategy.
- [x] Add config for “no-dedupe” mode for strict per-asset storage.

## Testing

- [x] Unit tests for MIME parsing and policy decisions.
- [x] Unit tests for body dedupe with distinct asset entries.
- [ ] Integration test: local server with HTML + CSS + JS + images; verify per-asset records.
- [ ] Integration test: redirect chain produces multiple assets.
- [x] CLI tests for list/show/export commands.
