The `tui` crate is the terminal interface for The Answer Protocol. Built with
Ratatui and Crossterm, it provides the complete MUD-style game experience while
the shared `api-client` crate handles TAP framing, requests, responses, events,
and connection lifecycle.

See the [client overview](../README.md) for the shared client architecture, API
layer, and presentation assets.

## Requirements

- a Rust toolchain with Cargo and Rust 2024 edition support;
- a terminal supported by Crossterm;
- a running Go TAP server, normally available at `127.0.0.1:38800`.

## Build and run

From the repository root:

```bash
make build-client-tui
make run-client-tui
```

| Flag | Default | Purpose |
| --- | --- | --- |
| `--ip` | `127.0.0.1` | Go TAP server IP address or hostname. |
| `--port` | `38800` | Public TAP listening port. |

Pass the flags through Make when another endpoint is required:

```bash
make run-client-tui CLIENT_ARGS="--ip 192.0.2.10 --port 38800"
```

See the client [build and run guide](../README.md#build-and-run) for dependency
installation, GUI targets, and client-wide linting.

## Runtime behavior

### Login and state loading

The login view collects a player name, opens the TAP connection, and sends
`CONNECT`. After authentication, the application loads `WHO`, `STATUS`,
`INVENTORY`, `QUESTS`, and `LOOK` through the asynchronous network manager.

The UI keeps network, game, and presentation state separate. API responses and
server events update centralized state before views redraw. A successful move
automatically triggers `LOOK` to refresh the room.

### Terminal lifecycle

At startup, the client enables raw mode, enters the alternate screen, captures
mouse input, and starts the application event broker. The broker combines
terminal events with a 33 ms interface tick on a bounded asynchronous channel.

Normal shutdown restores the cursor, mouse capture, alternate screen, and raw
mode. The panic hook also restores the terminal before reporting a failure.

## Keyboard and mouse controls

| Key | Action |
| --- | --- |
| `Ctrl+C` | Quit the application. |
| `Ctrl+H` | Toggle the help overlay. |
| `Ctrl+E` | Toggle the event and trace overlay. |
| `F1` | Toggle the chat overlay. |
| `Tab` / `Shift+Tab` | Cycle focus through input, NPCs, room items, quests, action history, inventory, and the right panel. |
| Arrow keys on the right panel | Move north, south, west, or east. |
| Arrow keys in lists or overlays | Change selection or scroll. |
| `Enter` | Submit input, open an action or detail view, or advance dialogue depending on focus. |
| `Esc` | Close the active popup or modal. |
| Left mouse button | Focus and select supported panels or dismiss notifications. |
| `Ctrl+S` in the fight editor | Submit the encoded C solution. |

## Text commands

The input parser uses short aliases defined by `ApiRequest::parse`:

| Input | TAP command |
| --- | --- |
| `connect <name>` | `CONNECT <name>` |
| `quit` | `QUIT` |
| `look` | `LOOK` |
| `move <direction>` | `MOVE <direction>` |
| `who` | `WHO` |
| `say <message>` | `CHAT GLOBAL <message>` |
| `msg <name> <message>` | `CHAT PRIVATE <name> <message>` |
| `take <item>` | `TAKE <item>` |
| `drop <item>` | `DROP <item>` |
| `inv` | `INVENTORY` |
| `status` | `STATUS` |
| `talk <npc>` | `TALK <npc>` |
| `attack <npc>` | `ATTACK <npc>` |
| `quest <npc>` | `QUEST <npc>` |
| `quests` | `QUESTS` |
| `gc` | `GROUP CREATE` |
| `gi <name>` | `GROUP INVITE <name>` |
| `gj <name>` | `GROUP JOIN <name>` |
| `gl` | `GROUP LEAVE` |
| `fc <npc>` | `FIGHT CREATE <npc>` |
| `fa <encoded-code>` | `FIGHT ATTACK <encoded-code>` |

These aliases describe TUI input only. See the root
[TAP command reference](../../README.md#tap-commands) for the complete wire
syntax, responses, errors, and server behavior.

## Package layout

| Module | Responsibility |
| --- | --- |
| `app` | Main loop and centralized terminal, network, API, and event handlers. |
| `network` | `api-client` task ownership and request/response envelopes. |
| `events` | Terminal ticks, application events, and the event broker. |
| `states` | Network, player, room, group, fight, overlay, and UI state. |
| `ui` | Views, panels, widgets, popups, focus, themes, and image rendering. |
| `data` | Cosmetic manifest loading and asset lookup. |
| `collections` | Reusable selectable lists and bounded interface histories. |
| `errors` | Unified terminal, network, event-channel, and application failures. |

The manifest and image catalog used by `data` are documented in the client
[shared assets section](../README.md#shared-assets).

## Logging

When launched through Make, the TUI appends structured diagnostic output to
`client/app.log`. The default tracing filter is `debug`; set `RUST_LOG` to
select another level:

```bash
RUST_LOG=info make run-client-tui
```

See the root [Server Logging](../../README.md#server-logging) section for the
server log formats, output destinations, full-stack monitoring, and abuse
detection guidance.
