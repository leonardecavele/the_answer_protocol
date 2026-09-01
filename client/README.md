# Clients

The client Cargo workspace contains two active components:

- `api-client`, a reusable asynchronous Rust library for The Answer Protocol;
- `tui`, the Ratatui terminal application built on top of that library.

There is no active GUI crate in this workspace. The GUI targets that remain in
the root Makefile are legacy targets and fail deliberately when `client/gui` is
absent.

## Workspace requirements

- a Rust toolchain with Cargo and Rust 2024 edition support;
- a terminal supported by Crossterm;
- a running Go TAP server, normally at `127.0.0.1:38800`.

Build and test both crates from this directory:

```bash
cd client
cargo build --workspace
cargo test --workspace
```

From the repository root, `make build-client-tui` builds the TUI and its local
`api-client` dependency.

## api-client

`api-client` owns TCP connection setup, the TAP v1 handshake, command encoding,
response parsing, error conversion, event decoding, and connection lifecycle.
Applications do not need to parse raw TAP frames.

### Connection model

`Client::connect` performs these steps:

1. connect to the requested TCP address;
2. wait for `OK hello proto=<version>`;
3. require protocol version 1;
4. start one background bridge task;
5. return the client and a broadcast receiver for server events.

The bridge serializes command requests: only one command is pending on the wire
at a time. `EVT` frames are published without consuming the pending `OK` or
`ERR` response.

### Minimal example

Add the local crate to another workspace package:

```toml
[dependencies]
api-client = { path = "../api-client" }
tokio = { version = "1", features = ["full"] }
```

```rust
use api_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, mut events) = Client::connect("127.0.0.1:38800").await?;

    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            println!("event: {event:?}");
        }
    });

    client.login("ALICE".to_string()).await??;
    let room = client.look().await??;
    println!("room: {}", room.room.name);

    client.quit().await;
    Ok(())
}
```

Command methods return a nested result:

- the outer `Result<_, TapError>` reports network, protocol, or internal bridge
  failures;
- the inner `Result<_, CommandError>` reports a server `ERR` response.

### Public commands

Convenience methods on `Client` cover login, look, move, global/private chat,
who, groups, resources, status, NPC interaction, quests, and quit.

The generic `execute_request(ApiRequest)` interface additionally covers the
code-fight commands and is the interface used by the TUI. The request enum
contains:

```text
Connect, Quit, Look, Move, Who
FightCreate, FightAttack
GlobalChat, PrivateChat
Take, Drop, Inventory, Status, Talk, Attack, Quest, Quests
GroupCreate, GroupJoin, GroupLeave, GroupInvite
```

The library does not expose typed room-chat or group-chat commands even though
the Go server accepts those raw TAP scopes.

### Configuration

`Client::connect_with` accepts a `ClientConfig`.

| Setting | Default | Purpose |
| --- | --- | --- |
| `connect_timeout` | 5 seconds | TCP connection deadline. |
| `handshake_timeout` | 2 seconds | TAP greeting deadline. |
| `request_timeout` | 10 seconds | Per-command response deadline. |
| `close_timeout` | 2 seconds | Grace period for bridge shutdown. |
| `max_frame_length` | 65,536 bytes | Maximum frame accepted by the client codec. The Go server itself uses 4,096 bytes. |
| `command_channel_capacity` | 2,048 | Queued command requests. |
| `event_channel_capacity` | 2,048 | Broadcast event backlog. |

### Errors

| Type | Meaning |
| --- | --- |
| `CommandError` | Parsed TAP `ERR`, with an optional numeric code and display message. |
| `NetworkError` | Connection, codec, timeout, or unexpected disconnection failure. |
| `ProtocolError` | Invalid opcode, malformed arguments, unsupported protocol version, or parse failure. |
| `InternalError` | The background bridge is no longer able to receive or complete requests. |
| `TapError` | Top-level wrapper for network, protocol, and internal errors. |

### Events

`Client::subscribe` creates additional receivers for the same event stream.
Recognized `ServerEvent` values cover sessions, game-server availability,
presence, chats, groups, world resources, death/kill, statistics, and fight
lifecycle. Syntactically valid but unsupported event forms become
`ServerEvent::Unknown`.

## TUI

The `tui` crate provides the interactive game client. It connects through
`api-client`; it contains no second TAP parser.

### Run

For the current asset path, launch from `client/tui`:

```bash
cd client/tui
cargo run -- --ip 127.0.0.1 --port 38800
```

CLI options:

| Option | Default | Purpose |
| --- | --- | --- |
| `--ip` | `127.0.0.1` | Go TAP server IP or hostname. |
| `--port` | `38800` | Go TAP server port. |

The application writes `app.log` in its working directory. It restores terminal
raw mode and the alternate screen during normal exit and through its panic
hook.

The manifest path is currently `../assets/manifest.json`, relative to the
process working directory. Starting from `client/tui` resolves it to the tracked
`client/assets` directory. The root Make target starts the binary from `client`,
so that path currently misses the manifest and the TUI falls back to empty
cosmetic data.

### Login and state loading

The login view collects a player name, opens the TAP connection, and sends
`CONNECT`. After authentication, the application loads `WHO`, `STATUS`,
`INVENTORY`, `QUESTS`, and `LOOK` through the asynchronous network manager.

The UI keeps separate network, game, and presentation state. API responses and
server events update centralized state before views redraw. A successful move
automatically triggers `LOOK` to refresh the room.

### Keyboard and mouse controls

| Key | Action |
| --- | --- |
| `Ctrl+C` | Quit the application. |
| `Ctrl+H` | Toggle the help overlay. |
| `Ctrl+E` | Toggle the event/trace overlay. |
| `F1` | Toggle the chat overlay. |
| `Tab` / `Shift+Tab` | Cycle focus through input, NPCs, room items, quests, action history, inventory, and the right panel. |
| Arrow keys on the right panel | Move north, south, west, or east. |
| Arrow keys in lists or overlays | Change selection or scroll. |
| `Enter` | Submit input, open an action/detail view, or advance dialogue depending on focus. |
| `Esc` | Close the active popup or modal. |
| Left mouse button | Focus and select supported panels or dismiss notifications. |
| `Ctrl+S` in the fight editor | Submit the encoded C solution. |

### Text commands

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

The in-application help overlay still displays long names such as
`group_create` and `group_join`; those strings do not match the current parser.
The aliases in the table above are authoritative for the present code.

### UI architecture

| Area | Responsibility |
| --- | --- |
| `app` | Main loop and centralized terminal/network/API/event handlers. |
| `network` | `api-client` task ownership and request/response envelopes. |
| `events` | Terminal ticks, application events, and event broker. |
| `states` | Network, player, room, group, fight, overlay, and UI state. |
| `ui` | Views, panels, widgets, popups, focus, and image rendering. |
| `data` | Cosmetic manifest loading and asset lookup. |

`client/assets/manifest.json` maps protocol identifiers to local names, images,
NPC kinds, and contextual actions. The server remains authoritative for game
state and descriptions.

## Known compatibility gaps

- Rust quest responses use `name`, while `api-client::QuestData` requires
  `quest_id`; non-empty `QUEST` and `QUESTS` responses fail typed
  deserialization.
- `CHAT ROOM` and `CHAT GROUP` are raw TAP capabilities without typed client
  variants.
- `USE` is neither typed nor functional end to end.
- The TUI help's long group command names are stale relative to the short input
  aliases.
- The current relative asset path requires launching the TUI from `client/tui`
  for images and manifest metadata to load.
- During a Rust-server outage, the Go server does not persist `GROUP INVITE`
  and its `GROUP JOIN` path can terminate the Go process.
