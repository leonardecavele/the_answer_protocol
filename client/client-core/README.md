The `client-core` crate owns the application logic shared by the
[TUI](../tui/README.md) and [GUI](../gui/README.md): state, event handling,
network orchestration, screens, widgets, focus, and presentation assets.
It is a library; either frontend supplies device events and a Ratatui backend.

The [Client API](../client-api/README.md) owns TCP transport, TAP framing,
typed commands, and response decoding. The core turns those protocol results
into application state and user interactions. The server remains authoritative
for gameplay. Public frames are documented in [PROTOCOL.md](../../PROTOCOL.md).

## Application ownership

`App` is the common entry point. Its four fields separate the responsibilities
needed by both frontends:

| Field | Responsibility |
| --- | --- |
| `state: AppState` | Network status, game state, UI state, and shutdown flag. |
| `event_broker: EventBroker` | Bounded application-event channel and periodic ticks. |
| `network_manager: Option<NetworkManager>` | Active connection task and request-chain queue. |
| `view_manager: ViewManager` | Active screen, trace overlay, and notifications. |

The frontend feeds events to `App::update` and calls `App::draw` with its
Ratatui frame. Network tasks send events back to the application instead of
mutating its state directly. This keeps state transitions in the application's
event handler while requests wait asynchronously.

## Event flow

`EventBroker` holds up to 200 application events and generates a tick every
33 ms. Widgets and background tasks use cloned senders to feed the same bus.
Dropping the broker aborts its tick task.

| Event | Handling |
| --- | --- |
| `DeviceEvent` | Global shortcuts, then input routing through the active view. |
| `Tick` | Remove expired notifications and tick the active view. |
| `Connection` | Start a connection, enter the game, or report connection failure/loss. |
| `Send` | Parse raw input or submit a typed request. |
| `Api` | Apply command responses, server events, transport failures, or raw-frame traces. |
| `Custom` | Update the lag flag or leave a fight editor after its grace deadline. |

A contextual button and the command input both produce `SendEvent` values.
Raw input is parsed by `ApiRequest::parse`; invalid input produces a warning.
Valid requests pass through `App::send`, which builds a request chain before
submitting it to the network manager.

## State model

`AppState` separates three kinds of state:

| State | Content |
| --- | --- |
| `NetworkState` | Gateway address, connection status, and lag indicator. |
| `GameState` | Player, group, room, server counters, fight, chat, action history, focus, overlays, and assets. |
| `UiState` | Notifications and protocol/event traces. |

Game session models live under `states/game/session`; focus and popup state
live under `states/game/interaction`; item, NPC, direction, and sprite models
live under `states/game/world`. Selectable collections retain the current
selection for panels. Chat and action histories are bounded to 200 and 50
entries respectively.

Changing focus clears relevant selections. Overlay state records which popup
is open, while the renderer owns the corresponding widgets and screen areas.
Closing a dialogue starts a 300 ms cooldown before another NPC interaction can
reopen it.

## Connection lifecycle

`App::new` loads the manifest, creates the initial state and event broker, and
opens `LoginView`. A manifest error falls back to the default manifest and
appears as a notification.

Submitting the login form starts `NetworkManager`, which connects through
`client-api` and authenticates the uppercase player name. Once authenticated,
the application records the endpoint and player, queues its initial state
load, and switches to `GameView`.

The manager runs command execution separately from forwarding raw frames,
server events, and connection-state changes. Request failures become
notifications; connection loss returns the application to login. A successful
`QUIT` response also disconnects. Resetting the app preserves the endpoint,
manifest, and assets while rebuilding session and UI state. Dropping the
network manager aborts its owned task.

## Request chains

Both frontends submit `RequestChain` values to a queue with capacity for 128
chains. Each chain occupies one entry and is processed before the next chain.
Requests still execute sequentially: each command waits for its response
before the following command is sent. Responses are forwarded individually
to the application, and asynchronous server events remain active.

`App::send` uses `RequestChain::build`, which expands `MOVE` into
`[MOVE, LOOK]` and wraps other commands in a single-request chain.
`App::send_chain` accepts an explicitly constructed sequence.

| Trigger | Chain |
| --- | --- |
| Movement | `[MOVE, LOOK]` |
| Initial state load | `[WHO, STATUS, INVENTORY, QUESTS, LOOK]` |
| Player death refresh | `[LOOK, STATUS]` |

For two successful movements, the wire order is
`MOVE -> response -> LOOK -> response -> MOVE -> response -> LOOK -> response`.
Queuing the refresh with its movement prevents another queued movement from
being inserted between `MOVE` and `LOOK`. The chain is an application queue
unit; TAP still carries individual command and response frames.

A server command error or local request failure stops the current chain and
discards its unsent requests. Completed commands are not rolled back, and other
queued chains are not discarded by this mechanism. A full or closed queue
rejects the new chain and produces a warning; without a network manager, the
dropped chain is recorded in the trace log.

## Latency indicator

The shared command input displays a red `LAG` block when the rolling average
of the last three completed request timings reaches at least 1,000 ms. Until
three measurements are available, the average uses the available samples.
The indicator disappears when that average falls below the threshold.

Measurements cover request execution and response-event delivery, including
failed requests, but exclude time waiting in the application command queue.
The indicator updates after each request completes; it is shared by the TUI
and GUI.

## Group and quest updates

Both frontends remove invitations on `GROUP INVITE <leader> REMOVED`. A
`GROUP LEAVE` event from the leader clears local group state and displays a
disbanding notification. Quest-step and completion events update the quest
panel and display notifications; completion data includes the awarded items.

## Response and event handling

`app/handlers/api` dispatches successful responses using both their typed
variant and the original request. This retains context such as a movement
direction, chat message, or target name. Server command errors become warnings;
transport failures become error notifications and traces. An unrelated
request/response pairing is recorded and reported.

| Domain | Application updates |
| --- | --- |
| Room | Replace room data after `LOOK`; apply presence, item, spawn, and despawn events. |
| Player | Update health, inventory, and quests from responses and events. |
| Chat | Record sent and received messages in global, room, group, or private channels. |
| Group | Track membership and invitations, handle leader departure, and refresh after group movement. |
| Dialogue | Display NPC replies and manage interaction state. |
| Combat | Open the editor, apply results to displayed health, and return to the game. |
| Server | Update player counters and game-server availability. |

## Views and components

`ViewManager` stores one active `Box<dyn Component>`: login, game, or fight
editor. It draws that view, then the optional trace overlay, then notifications.
The shared layout requires at least 80 columns by 24 rows and centers the
interface within a maximum of 400 columns by 100 rows. Smaller areas display
a size hint.

Input goes to the trace overlay while it is visible; otherwise notifications
get a chance to consume it before the active view.

The game view assembles the header, command footer, room lists, action history,
inventory, contextual right panel, and popup overlays. Popup input takes
priority over game shortcuts, focus navigation, and ordinary panel input.

| Abstraction | Responsibility |
| --- | --- |
| `Component` | Draw a widget into a Ratatui frame and rectangle. |
| `Lifecycle` | Handle device events and ticks. |
| `EventFlow` | Indicate whether input was consumed or may continue. |
| `Interactive<T>` | Remember the drawn rectangle and provide mouse hit testing. |
| `Scrollable<T>` | Reuse scrolling behavior around content such as chat or traces. |

Widgets submit application events through a sender. A new shared action can
therefore be added once to the core and used by both frontends. Backend-specific
input conversion and terminal/window setup stay in their frontend crates.

## Keyboard and mouse controls

| Input | Action |
| --- | --- |
| `Ctrl+C` | Quit the application. |
| `Ctrl+H` | Toggle help. |
| `Ctrl+E` | Toggle the event and trace overlay. |
| `F1` | Toggle the chat overlay. |
| `Tab` / `Shift+Tab` | Cycle focus across interactive panels. |
| Arrow keys on navigation | Move north, south, west, or east. |
| Arrow keys in lists | Change selection or scroll. |
| `Enter` | Submit input, activate an action, or advance dialogue. |
| `Esc` | Close the active popup or modal. |
| Left mouse button | Focus, select, or dismiss supported elements. |
| `Ctrl+S` in the fight editor | Submit the encoded C solution. |

Focus covers command input, players, NPCs, room items, quests, invitations,
action history, inventory, and the contextual right panel. Empty invitation
lists are skipped when cycling focus.

## Command input

The input and contextual actions use the same typed requests. The
[API command and alias table](../client-api/README.md#requests-and-responses)
is the reference for accepted text. Room, inventory, NPC, player, invitation,
quest, chat, and navigation panels expose actions without requiring the full
command to be typed.

## Fight editor

`FIGHT START` opens `EditorView` using the challenge and separators supplied
by the server. It restores the C source's spaces and newlines and displays the
remaining time. The opponent sprite and health bar are shown when space allows.

`Ctrl+S` encodes the edited source and submits `FIGHT ATTACK`, then switches
the local fight state from editing to awaiting a result. `FIGHT RESULT`
updates the displayed player or NPC health and records the outcome.
`FIGHT END` returns to the game view. If the server does not end the fight
within its supplied duration plus a ten-second grace period, a tick triggers
a warning and leaves the editor locally.

## Shared assets

[`client/assets/manifest.json`](../assets/manifest.json) associates server identifiers with presentation
metadata:

- NPC display names, roles, contextual actions, and sprites;
- item display names, descriptions, and sprites;
- room illustrations and navigation orientation;

Static images use `image_path`. Animated NPCs use an ordered `image_paths`
array and `frame_ms`. The referenced files live below
`client/assets/pictures`.

Assets only affect presentation. The server remains authoritative for rooms,
inventories, NPC state, quests, groups, fights, and every gameplay mutation.
They are embedded in both client binaries by default. Passing
`--assets <directory>` loads `manifest.json` and the referenced images from an
external directory instead.

## Notifications and logging

Notifications have a severity, optional topic, and finite or infinite lifetime.
A new notification with a topic replaces the previous one with that topic.
Ticks remove expired entries; notification timing supports pause and resume.

The trace overlay records sent/received frames, connection changes, invalid
input, dropped requests, and failures with millisecond timestamps.
`ApiEvent::Lagged` reports lost broadcast entries in this trace; it is separate
from the response-time `LAG` badge.

`logging::setup(path)` configures file-based `tracing`, appending records
without ANSI coloring. `RUST_LOG` selects the filter, defaulting to `debug`.
The frontend chooses the file path; its README documents that destination.

## Source layout

| Path | Responsibility |
| --- | --- |
| `src/app.rs` | App ownership, update/draw entry points, sending, state loading, and reset. |
| `src/app/handlers/` | Device, connection, tick, and send handlers. |
| `src/app/handlers/api/` | Responses and server events split by domain. |
| `src/events.rs` | Application event types, bounded bus, and tick task. |
| `src/network/` | Connection orchestration, request chains, and latency measurements. |
| `src/states/` | Network, UI, session, interaction, and world presentation state. |
| `src/renderer/view.rs` | Active view and global overlay routing. |
| `src/renderer/views/` | Login, game panels/popups, and C editor. |
| `src/renderer/components/` | Shared component traits, wrappers, and widgets. |
| `src/assets.rs`, `src/manifest.rs` | Embedded/directory assets and presentation metadata. |
| `src/collections.rs` | Selectable lists and bounded histories. |
| `src/notification.rs`, `src/logging.rs` | Notifications and tracing setup. |
| `src/cli.rs` | Shared command-line options consumed by the frontends. |

## Validation

From the repository root:

```bash
cargo test --manifest-path client/client-core/Cargo.toml
cargo clippy --manifest-path client/client-core/Cargo.toml --all-targets -- -D warnings
```

The first command runs the crate's tests; the second checks all its targets
with Clippy and treats warnings as errors. To validate integration, run either
frontend and exercise login, movement chains, a failed chain, the lag badge,
group events, and a fight through the shared application.
