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

| Flag | Default | Purpose |
| --- | --- | --- |
| `--rust-server-port` | `38801` | Internal TCP listening port for the Go server. |

Pass the flag through Make when another port is required:

```bash
make run-rust-server RUST_SERVER_ARGS="--rust-server-port 38802"
```

The server listens on `0.0.0.0:38801` by default. The Go server must be
configured with the same Rust-server port. If the reader side of that
connection closes, the game loop saves state and exits.

Set `RUST_LOG` to change tracing verbosity:

```bash
RUST_LOG=debug make run-rust-server
```

## Startup and game loop

1. Parse and validate all four world JSON files.
2. Restore server and player saves where available.
3. Ensure the special lost item exists.
4. Bind the configured port (`38801` by default) and accept the Go server.
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

See the root [World Design](../../README.md#world-design) section for the room
graph, NPC roles, item distribution, and gameplay-facing world description.

## Accepted internal commands

The server [internal command matrix](../README.md#internal-command-matrix) is
the source of truth for accepted commands, envelope forms, response payloads,
group handling, and internal questions. The root
[TAP command reference](../../README.md#tap-commands) separately documents the
public syntax seen by clients.

## Events

Rust accumulates game changes in a per-player tick diff and sends one targeted
JSON event batch for each affected player at the end of the tick. The server
[targeted event batch](../README.md#targeted-event-batch) section documents that
private JSON envelope. Go translates it into the public frames cataloged in the
root [TAP event reference](../../README.md#tap-events).

## C challenge sandbox

See the root [Combat System](../../README.md#combat-system) section for fight
creation, participant actions, damage, deadlines, death, and respawn rules.

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
