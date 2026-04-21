# Brazen Automation Examples

This directory contains examples of how to interact with the Brazen Automation API using different languages and tools.

## Prerequisites

The Brazen browser must be running with automation enabled. By default, it listens on `ws://127.0.0.1:7942/ws`.

## Examples

### Python

A high-level client using the `websockets` library.

- [brazen_client.py](./python/brazen_client.py)
- **Usage**:
  ```bash
  pip install websockets
  python python/brazen_client.py
  ```

### Bash

A simple script using `websocat` and `jq` to perform a common task (taking a screenshot).

- [take_screenshot.sh](./bash/take_screenshot.sh)
- **Usage**:
  ```bash
  ./bash/take_screenshot.sh
  ```

## API Documentation

For a full reference of the protocol, commands, and events, see:
- [Automation API Guide](../../docs/automation/api.md)
- [JSON Schema](../../docs/automation/schema.json)
