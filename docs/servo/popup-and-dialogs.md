# Popup And Dialog Mediation

## Popup Policy

- New windows are routed according to `engine.new_window_policy`.
- When multi-tab support is not available, the shell logs deferred popups.

## Dialog Policy

- Alert/confirm/prompt are mediated by the shell.
- Dialogs must be tied to the originating tab and session.

## Context Menus

- Context menus are owned by the shell to avoid privileged actions leaking into content.
