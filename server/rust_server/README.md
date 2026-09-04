# Rust game server

The Rust server owns the authoritative world for The Answer Protocol. It loads
rooms, items, NPCs, and quests; tracks players and fights; applies timed world
updates; persists state; emits targeted events; and evaluates C submissions in
a Linux sandbox.

It listens for one Go gateway connection and does not expose TAP directly. The
public gameplay commands and frames are defined in the root
[TAP protocol reference](../../protocole.md).

## Requirements

- Rust toolchain with Cargo and Rust 2024 edition support
- Linux on `x86_64` or `aarch64`
- `/usr/bin/clang`
- `/usr/bin/bwrap`

## Build and run

Run these commands from the repository root:

```bash
make build-rust-server
make run-rust-server
```

| Flag | Default | Purpose |
| --- | --- | --- |
| `--rust-server-port` | `38801` | Internal TCP port used by the Go gateway. |

Example:

```bash
make run-rust-server RUST_SERVER_ARGS="--rust-server-port 38802"
```

The server listens on `0.0.0.0`. Configure the Go gateway with the same port.
Set `RUST_LOG` to select the tracing level:

```bash
RUST_LOG=debug make run-rust-server
```

## Startup and game loop

At startup, the server:

1. parses and validates the world JSON assets;
2. restores server and player saves;
3. initializes renewable world resources;
4. binds the internal TCP port and accepts the Go gateway;
5. starts the authoritative 10 Hz game loop.

One connection-reader thread decodes newline-delimited JSON into an MPSC
channel. The game loop drains that channel and remains the single owner of
mutable world state. It also receives completed C-test results from worker
threads, advances timers, emits batched events, and saves the world.

| Timed operation | Interval |
| --- | --- |
| Game tick | 100 ms / 10 Hz |
| Automatic save | 2 minutes |
| NPC respawn | 30 seconds |
| Dropped-item despawn | 1 minute |
| Fight deadline | 222 seconds |
| Foyer wrap spawn | Periodic |

The server saves on its periodic deadline and during a handled shutdown. TOML
saves are written below `saves/`.

## World assets

| File | Content |
| --- | --- |
| `assets/rooms.json` | Room names, descriptions, exits, and initial items. |
| `assets/items.json` | Item models, descriptions, and behaviors. |
| `assets/npcs.json` | Spawn rooms, dialogue, flags, health, and quest links. |
| `assets/quests.json` | Quest definitions, ordered steps, and rewards. |
| `assets/code/` | C challenge source presented to players. |
| `assets/tests/` | Trusted public and hidden test harnesses. |

Startup validation rejects inconsistent references, duplicate definitions,
invalid exits, and unknown room, item, NPC, or quest identifiers. Runtime
entities use protocol identifiers such as `2.devant_l_ecole`,
`0.objet_perdu`, and `12.ldecavel`.

Rooms form a directed graph. Each room tracks its players, NPCs, fixed objects,
and collectable item instances. Dropped resources expire after one minute.
Wraps spawn periodically in the foyer, providing a renewable consumable during
long sessions.

## Internal protocol

Every internal message is UTF-8 JSON followed by `LF`. The server accepts
single-player commands, grouped commands, and correlated questions.

### Single-player command

```json
{
  "player": "ALICE",
  "command": "USE",
  "data": "0.wrap"
}
```

### Grouped command

```json
{
  "leader": "ALICE",
  "grouped_players": ["BOB", "CHARLIE"],
  "command": "FIGHT_CREATE",
  "data": "12.ldecavel"
}
```

The leader is excluded from `grouped_players`. Grouped `MOVE`, `QUEST`, and
`FIGHT_CREATE` are handled as one authoritative mutation.

### Command response

```json
{
  "player": "ALICE",
  "command": "USE",
  "error_code": 0,
  "data": "health=100"
}
```

### Room question

```json
{
  "question": "ROOM_PLAYERS",
  "data": "ALICE",
  "id": "c94d8d2b-..."
}
```

The answer repeats `question` and `id`. Its `data` contains a serialized JSON
array of player names.

### Targeted events

World changes accumulate in a per-player tick diff. At the end of each tick,
the server sends one event batch for each affected player:

```json
{
  "player": "ALICE",
  "events": [
    {
      "emitted_by": "BOB",
      "event_name": "ROOM PRESENCE ENTER"
    }
  ]
}
```

The Go gateway converts each internal entry to the public TAP event form.

## Supported operations

| Internal command | Result |
| --- | --- |
| `CONNECT` | Register or restore a player. |
| `QUIT` | Save and remove the active player. |
| `LOOK` | Return room, exits, players, items, and NPCs as JSON. |
| `MOVE` | Move one player or an entire group. |
| `TAKE` | Transfer a room item to an inventory. |
| `DROP` | Transfer an inventory item to the room. |
| `INVENTORY` | Return item identifiers as JSON. |
| `USE` | Apply the selected inventory item's behavior. |
| `TALK` | Advance per-player NPC dialogue. |
| `ATTACK` | Resolve a direct-damage attack. |
| `STATUS` | Return health and status as JSON. |
| `QUEST` | Assign an individual or grouped quest. |
| `QUESTS` | Return active quests as JSON. |
| `FIGHT_CREATE` | Start an individual or grouped C challenge. |
| `FIGHT_ATTACK` | Queue a submission for sandboxed evaluation. |

The `ROOM_PLAYERS` question returns the authenticated players that share a
specified player's current room.

## Items and `USE`

Item behavior is resolved by the game engine so inventory checks and effects
are applied atomically. `USE` accepts a protocol identifier or an unambiguous
exact item name, validates ownership, applies the item's effect, updates or
consumes the instance as appropriate, and returns the result to the gateway.

## Quests

Quest definitions contain descriptions, ordered objectives, and probabilistic
rewards. An NPC can assign a quest to one player or, through a grouped `QUEST`
request, to every eligible member of the leader's group. `QUESTS` serializes
the active quest state for the client.

TODO: connect gameplay events to automatic quest-step progression and reward
distribution.

## C challenge combat

`FIGHT_CREATE` selects a challenge from `assets/code`; its filename selects the
trusted harness in `assets/tests`. A group leader creates one shared fight for
all eligible members.

For each `FIGHT_ATTACK`, the server:

1. restores spaces and newlines from the negotiated TAP separators;
2. limits the submission to 64 KiB and validates the challenge filename;
3. compiles the submission together with the trusted test harness;
4. runs compilation and execution inside Bubblewrap;
5. applies process groups, timeouts, rlimits, and a seccomp filter;
6. returns the result to the game loop, which applies damage and emits events.

The sandbox has no network access, capabilities, writable host mounts,
standard input/output, or normal C runtime. CPU time, address space, file size,
open descriptors, and stack size are bounded.

Successful damage is based on target health at fight creation divided by the
participant count, with a minimum of five. A finishing blow deals exactly the
target's remaining health. A failed submission deals 25 to 50 damage to the
player. On death, the player returns to the starting room with reduced health.

Fight creation emits `FIGHT START`; each evaluation emits `FIGHT RESULT`; the
completed or expired instance emits `FIGHT END`.

## Module layout

| Module | Responsibility |
| --- | --- |
| `game` | Authoritative loop, timers, dispatch, events, and saves. |
| `models` | Rooms, resources, players, groups, quests, and combat state. |
| `commands` | Single-player and grouped command handlers. |
| `parser` | Internal JSON envelope decoding and validation. |
| `network` | Gateway connection and newline-delimited transport. |
| `tester` | Clang/Bubblewrap compilation and execution workers. |
| `save` | Server and per-player TOML persistence. |

## Validation

```bash
make lint-rust-server
cargo test --manifest-path server/rust_server/Cargo.toml
```

The lint target runs `rustfmt` and Clippy.
