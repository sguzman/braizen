# Navigation And Session Model Roadmap

Tracks tabs, windows, browsing sessions, lifecycle state, and the browser’s internal model of navigation.

## Current State

- [x] Active tab model exists
- [x] URL/title state exists
- [x] Navigation command dispatch exists
- [x] Reload command exists
- [x] Engine-originated events are surfaced into shell state
- [x] Back/forward commands are modeled in the shell

## Core Session Model

- [x] Back/forward navigation stacks
- [x] Pending vs committed navigation state
- [x] Redirect chain capture
- [x] Window and tab lineage model
- [x] Session restore
- [x] Crash recovery state
- [x] Profile-bound session separation
- [x] Session file format versioning
- [x] Session JSON persistence

## Browser Data Model

- [x] Structured models for windows, tabs, frames, and documents
- [x] Selection and focused-element state
- [x] Download and permission-grant linkage to sessions
- [x] Browsing-session identifiers stable across subsystems
- [x] Revisit history and tab lineage metadata
- [x] Navigation history stored per tab

## User-Facing Flows

- [x] Open in new tab/window flows
- [x] Duplicate, pin, mute, and close behaviors
- [x] Session snapshot export/import
- [x] Active tab switching UI
- [x] Profile-based session path usage
- [x] Crash recovery flag persisted in snapshots
