# Servo Devtools Integration

## Plan

- Prefer an explicit devtools transport that is disabled by default.
- Scope devtools to localhost or a trusted socket.
- Require explicit user enablement in config.

## Transport Selection

- `none`: devtools disabled.
- `local-socket`: Unix domain or named pipe (preferred for security).
- `tcp`: localhost-only, with explicit token.

## Security Constraints

- No remote binding by default.
- Authentication required for non-local transports.
- Separate devtools from automation endpoints to avoid privilege escalation.
