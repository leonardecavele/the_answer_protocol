The Rust server owns the persistent game world for The Answer Protocol. It
loads rooms, items, NPCs, and quests; tracks players and combat instances;
produces per-player event batches; saves progress; and evaluates C submissions
inside a Linux sandbox.

It is not a public TAP endpoint. Only the [Go server](../go_server/README.md)
connects to it. The complete internal JSON contract is documented in the
[server architecture guide](../README.md).

## Requirements

- a Rust toolchain with Cargo and Rust 2024 edition support;
- Linux on `x86_64` or `aarch64` for challenge execution;
- `/usr/bin/clang` and `/usr/bin/bwrap` for the C sandbox.

World exploration can compile without invoking the sandbox, but fight
submissions fail evaluation when the absolute `clang` or `bwrap` paths are not
available.

## Build and run

From the repository root:

```bash
make build-rust-server
make run-rust-server
```

The server listens on `0.0.0.0:38801`. If the reader
side of that connection closes, the game loop saves state and exits.

Set `RUST_LOG` to change tracing verbosity:

```bash
RUST_LOG=debug cargo run
```

## Startup and game loop

1. Parse and validate all four world JSON files.
2. Restore server and player saves where available.
3. Ensure the special lost item exists.
4. Bind port `38801` and accept the Go server.
5. Run the game tick at 10 Hz.
6. Process incoming commands, tester results, respawns, item despawns, combat
   deadlines, event batches, and periodic saves.

The server saves automatically every two minutes and on a handled Ctrl-C or
game-loop exit. Saves are written below `saves/` as TOML files and are ignored
by Git.

## World data

| File | Content |
| --- | --- |
| `rooms.json` | Room names, descriptions, exits, and starting inventories. |
| `items.json` | Item model names and descriptions. Runtime instances receive numeric IDs. |
| `npcs.json` | NPC names, spawn rooms, dialogue, flags, health, and quest references. |
| `quests.json` | Quest descriptions and probabilistic loot definitions. |

Startup validation rejects duplicate quests, missing spawn rooms, invalid room
exits, unknown item IDs, invalid NPC spawn rooms, and unknown NPC quest IDs.

Protocol representations normally use `<numeric-id>.<name>`, for example
`2.devant_l_ecole`, `0.objet_perdu`, or `12.ldecavel`.

## Accepted internal commands

| Command | Purpose |
| --- | --- |
| `CONNECT` | Create or restore a player and emit room presence. |
| `LOOK` | Return the current room, players, items, NPCs, and exits. |
| `MOVE` | Move a single player or group and emit presence/group movement events. |
| `QUIT` | Save and disconnect a player. |
| `TAKE`, `DROP`, `INVENTORY` | Manage item ownership and room inventory. |
| `TALK` | Advance dialogue with an NPC in the same room. |
| `ATTACK` | Perform the legacy direct-damage attack. |
| `STATUS` | Return HP, maximum HP, and health status. |
| `QUEST`, `QUESTS` | Assign or list quests. |
| `FIGHT_CREATE` | Create a code-challenge combat instance. |
| `FIGHT_ATTACK` | Queue an encoded C submission for evaluation. |

Grouped envelopes are implemented only for `MOVE` and `FIGHT_CREATE`.
`ROOM_PLAYERS` is the only accepted internal question.

## Events

Rust accumulates game changes in a per-player tick diff. At the end of a tick,
it sends one JSON event batch for each affected player. Current event names
include:

- `ROOM`, with `PRESENCE ENTER` or `PRESENCE LEAVE` data;
- `GROUPMOVE`, `TAKE`, and `DROP`;
- `SPAWN`, `DESPAWN`, `KILL`, and `DEATH`;
- `FIGHT START`, `FIGHT RESULT`, and `FIGHT END`.

Go turns those objects into the public frames cataloged in the root
[TAP events](../../README.md#tap-events) section.

## C challenge sandbox

`FIGHT_CREATE` selects a source file from `assets/code`. The file name also
selects a trusted test harness from `assets/tests`.

On `FIGHT_ATTACK`, the server:

1. restores spaces and newlines from the protocol separators;
2. limits submissions to 64 KiB and validates the challenge filename;
3. compiles the submission and trusted test harness with Clang;
4. runs both compiler and program inside Bubblewrap;
5. applies process groups, wall-clock timeouts, rlimits, and a seccomp filter;
6. emits a `FIGHT RESULT` event and applies player or NPC damage.

The compiled program runs without networking, capabilities, writable host
mounts, standard input/output, or the normal C runtime. Compilation and
execution are additionally bounded by CPU, address-space, file, descriptor,
and stack limits.