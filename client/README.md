The client workspace provides [terminal](tui/README.md) and [graphical](gui/README.md) entry points to the same
game. Both use one Rust [application core](client-core/README.md), the same Ratatui views and widgets,
and the same [asynchronous TAP library](client-api/README.md). The frontends differ only at the event
and rendering boundaries.

## Components

- [Client API](client-api/README.md) documents connection setup, typed
  requests, responses, events, configuration, and errors.
- [`client-core`](client-core/README.md) owns the shared application, state, networking,
  assets, components, and Ratatui renderer.
- [TUI](tui/README.md) documents the Crossterm event loop, terminal lifecycle,
  and terminal rendering.
- [GUI](gui/README.md) documents Egui-to-Crossterm event translation, the
  software Ratatui backend, desktop rendering, and controls.

The public frames shared with the servers are specified in the root
[TAP protocol reference](../PROTOCOL.md).

## Shared application

The [Client core README](client-core/README.md) explains application ownership,
event flow, state, connection lifecycle, request chains, latency detection,
views, controls, the fight editor, and assets. Both frontends use this logic.

Frontend-specific rendering pipelines and lifecycle details live in the
[TUI](tui/README.md) and [GUI](gui/README.md) READMEs.

## Requirements

- Rust toolchain with Cargo and Rust 2024 edition support
- A Crossterm-compatible terminal for the TUI
- A supported native desktop environment for Eframe/Egui
- A [TAP gateway](../server/go_server/README.md), normally at `127.0.0.1:38800`

## Build and run

From the repository root:

```bash
make install
make build-client-tui
make build-client-gui
```

Launch either frontend:

```bash
make run-client-tui
make run-client-gui
```

Both clients accept these options through `CLIENT_ARGS`:

| Flag | Default | Purpose |
| --- | --- | --- |
| `--ip` | `127.0.0.1` | Go TAP server IP address or hostname. |
| `--port` | `38800` | Public TAP server port. |
| `--assets` | Embedded assets | Optional directory containing `manifest.json` and pictures. |

Examples:

```bash
make run-client-tui CLIENT_ARGS="--ip 192.0.2.10 --port 38800"
make run-client-gui CLIENT_ARGS="--ip 192.0.2.10 --port 38800 --assets ./assets"
```

Both connect to `127.0.0.1:38800` and use their embedded assets by default.

Run client-wide formatting and static analysis with:

```bash
make lint-client
```

Build details, runtime behavior, and controls are documented by the component
READMEs linked above.
