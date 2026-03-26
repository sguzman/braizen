# Automation WebSocket API

Brazen exposes a local automation WebSocket intended for trusted clients. The API is JSON‑based and uses request/response envelopes with optional event subscriptions.

## Connect

The socket binds to `automation.bind` (default `ws://127.0.0.1:7942`). If `require_auth = true`, include a bearer token:

```
Authorization: Bearer <token>
```

You can also provide `?token=<token>` in the URL for local tooling.

## Envelopes

Requests:

```json
{ "id": "1", "type": "tab-list" }
```

Responses:

```json
{ "id": "1", "ok": true, "result": { ... }, "error": null }
```

Events:

```json
{ "id": null, "topic": "navigation", "url": "https://example.com", "title": "Example", "load_status": "complete", "load_progress": 1.0 }
```

## Requests

Tab APIs:

- `tab-list`
- `tab-activate` `{ "index": 0 }` or `{ "tab_id": "<uuid>" }`
- `tab-new` `{ "url": "https://example.com" }`
- `tab-close` `{ "index": 0 }`
- `tab-navigate` `{ "url": "https://example.com" }`
- `tab-reload`
- `tab-stop`
- `tab-back`
- `tab-forward`

Cache APIs:

- `cache-stats`
- `cache-query` `{ "query": { "mime": "text/html" }, "limit": 50 }`
- `cache-body` `{ "asset_id": "<id>" }` (base64 response body)

Subscriptions:

- `subscribe` `{ "topics": ["navigation", "capability"] }`

Not yet implemented (stubs return error):

- `dom-query`
- `rendered-text`
- `article-text`
- `tts-control`
- `tts-enqueue`

## Notes

- Tab and cache APIs are gated by `permissions` and `automation.expose_*` flags.
- Rate limiting and connection caps are enforced by `automation.max_*`.
