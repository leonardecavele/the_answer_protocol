# Clients

The client workspace provides terminal and graphical entry points to the same
game. Both use one Rust application core, the same Ratatui views and widgets,
and the same asynchronous TAP library. The frontends differ only at the event
and rendering boundaries.

## Components

- [API client](api-client/README.md) documents connection setup, typed
  requests, responses, events, configuration, and errors.
- [TUI](tui/README.md) documents the Crossterm event loop, terminal lifecycle,
  controls, commands, and Ratatui rendering.
- [GUI](gui/README.md) documents Egui-to-Crossterm event translation, the
  software Ratatui backend, desktop rendering, and controls.

The public frames shared with the servers are specified in the root
[TAP protocol reference](../PROTOCOL.md).

## Shared application model

The `tui` crate is also a reusable library. It owns `App`, network integration,
screen states, focus, actions, overlays, and all Ratatui drawing code. Each
frontend supplies compatible input events and a Ratatui backend.

### Terminal pipeline

```mermaid
flowchart LR
    CE["Crossterm Event"] --> App["App"]
    App --> Ratatui["Ratatui"]
    Ratatui --> CB["Crossterm Backend"]
    CB --> Terminal["Terminal"]
```

### Graphical pipeline

```mermaid
flowchart LR
    EE["Egui Event"] --> CE["Crossterm Event"]
    CE --> App["App"]
    App --> Ratatui["Ratatui"]
    Ratatui --> SB["Soft Ratatui Backend"]
    SB --> EW["Egui Window"]
```

There is no pseudo-terminal between the GUI and the application. Egui input is
translated directly into Crossterm-compatible events; Ratatui renders into an
in-memory character grid; Egui then paints that grid in its native window.

## Shared assets

`client/assets/manifest.json` associates server identifiers with presentation
metadata:

- NPC display names, roles, contextual actions, and sprites;
- item display names, descriptions, and sprites;
- room illustrations and navigation orientation;
- quest descriptions shown by the interfaces.

Static images use `image_path`. Animated NPCs use an ordered `image_paths`
array and `frame_ms`. The referenced files live below
`client/assets/pictures`.

Assets only affect presentation. The server remains authoritative for rooms,
inventories, NPC state, quests, groups, fights, and every gameplay mutation.

## Requirements

- Rust toolchain with Cargo and Rust 2024 edition support
- A Crossterm-compatible terminal for the TUI
- A supported native desktop environment for Eframe/Egui
- A TAP gateway, normally at `127.0.0.1:38800`

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

The TUI accepts another endpoint through `CLIENT_ARGS`:

```bash
make run-client-tui CLIENT_ARGS="--ip 192.0.2.10 --port 38800"
```

The GUI connects to `127.0.0.1:38800` by default.

Run client-wide formatting and static analysis with:

```bash
make lint-client
```

Build details, runtime behavior, and controls are documented by the component
READMEs linked above.
