#!/usr/bin/env bash
set -euo pipefail

# Example usage (requires websocat):
#   websocat -H "Authorization: Bearer <token>" ws://127.0.0.1:7942/ws

cat <<'EOF'
Example requests:

{"id":"1","type":"tab-list"}
{"id":"2","type":"tab-new","url":"https://example.com"}
{"id":"3","type":"subscribe","topics":["navigation","capability"]}
{"id":"4","type":"cache-stats"}
EOF
