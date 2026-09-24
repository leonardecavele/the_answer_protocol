The Rust server owns the authoritative world for The Answer Protocol (TAP). It loads
rooms, items, NPCs, and quests; tracks players and fights; applies timed world
updates; persists state; emits targeted events; and evaluates C submissions in
an isolated Linux sandbox.

It listens for one [Go gateway](../go_server/README.md) connection at a time and
does not expose TAP directly to clients. The Go gateway manages client multiplexing,
chat broadcast scopes, and public RFC 42TAP text framing, while delegating authoritative
game logic, world mutations, and code evaluation to this Rust engine. The server supports
seamless disconnections and reconnections without losing server or player state.

The public protocol follows RFC 42TAP, supplied with the subject, and is
documented in the repository [protocol reference](../../PROTOCOL.md).

## Requirements

- Rust 1.91 or newer, with Cargo
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
Set `RUST_LOG` to select the tracing level (defaults to `info`):

```bash
RUST_LOG=debug make run-rust-server
```

Logs are output to standard output (formatted with timestamps) and mirrored to
`app.log`. An interactive admin console is also available via standard input.

## Startup, connection lifecycle, and game loop

At startup, the server:

1. parses and validates the world JSON configuration files (`npcs.json`, `items.json`, `rooms.json`, `quests.json`);
2. binds the internal TCP port;
3. accepts a connection from the Go gateway;
4. restores the server state (`saves/server_save/server_state.toml`) and player states (`saves/player_saves/<player>.toml`);
5. ensures essential world entities exist (such as the lost item `0.objet_perdu`);
6. starts the authoritative 20 Hz game loop.

When a Go gateway disconnects (either abruptly or cleanly), the server:
1. breaks out of the tick loop gracefully without panicking or aborting;
2. persists the current world and active player states to `saves/`;
3. resets transient connection state and waits on `listener.accept()` for a new connection;
4. restores saved progression when reconnecting.

One connection-reader thread decodes newline-delimited JSON into an MPSC
channel. The game loop drains that channel and remains the single owner of
mutable world state. It also processes admin console input, drains completed C-test
results from worker threads, advances timers, emits batched events, and saves the world.

| Timed operation | Interval |
| --- | --- |
| Game tick | 50 ms / 20 Hz |
| Automatic save | 2 minutes |
| NPC respawn | 30 seconds |
| Dropped-item despawn | 1 minute |
| Fight deadline | 222 seconds (3 min 42 s) |
| Periodic item spawns | Configured per item model cooldown |

Saves are written to TOML files below `saves/` (`saves/server_save/server_state.toml`
and `saves/player_saves/<player_name>.toml`) every 2 minutes and upon disconnect or
shutdown.

## World configuration and assets

| File / Directory | Content |
| --- | --- |
| `rooms.json` | Room definitions, descriptions, exits, and initial items. |
| `items.json` | Item models, descriptions, usable flags, and spawn timers. |
| `npcs.json` | Spawn rooms, dialogue, health, hostility flags, and quest links. |
| `quests.json` | Quest definitions, ordered steps, and rewards. |
| `assets/code/` | C challenge source files presented to players. |
| `assets/tests/` | Trusted public and hidden test harnesses. |

Startup validation in `parser.rs` rejects inconsistent references, duplicate definitions,
invalid exits, and unknown room, item, NPC, or quest identifiers. Runtime
entities use protocol identifiers such as `0.devant_l'école`, `0.objet_perdu`, and `0.bde`.

Rooms form a directed graph satisfying RFC requirements (interconnected loops and branches).
Each room tracks its players, NPCs, fixed objects, and collectable item instances.
Dropped resources expire after one minute (non-lost items have their IDs recycled,
while `0.objet_perdu` respawns in `pature`). Items with configured spawn info spawn
periodically in their designated rooms.

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

Error codes (RFC 42TAP codes where the RFC defines one, project extensions
otherwise; see the [protocol reference](../../PROTOCOL.md#errors)):
- `0`: `NoError`
- `301`: `NoExit`
- `404`: `ItemNotFound`, `ItemNotInInventory`, `NpcNotFound`
- `405`: `PlayerNotFound`, `NpcNotHostile`
- `406`: `NoQuestAvailable`
- `408`: `NpcInCombat`
- `409`: `ActionAlreadyTaken`
- `410`: `PlayerAlreadyInCombat`
- `411`: `PlayerNotInCombat`
- `412`: `FileNotFound`
- `413`: `RoomNotFound`
- `414`: `MissingItem`
- `415`: `NotUsable`
- `416`: `TooBigData`
- `997`: `InvalidGroupCommand`
- `998`: `InvalidQuestion`
- `999`: `InvalidCommand`

### Questions

```json
{
  "question": "ROOM_PLAYERS",
  "data": "ALICE",
  "id": "c94d8d2b-..."
}
```

The answer repeats `question` and `id`. Its `data` contains a serialized JSON
array of player names in the room.

### Targeted events

World changes accumulate in a per-player tick diff. At the end of each tick,
the server sends one event batch for each affected player:

```json
{
  "player": "ALICE",
  "events": [
    {
      "emitted_by": "BOB",
      "event_name": "ROOM",
      "data": "PRESENCE ENTER BOB"
    },
    {
      "event_name": "SPAWN",
      "data": "type=ITEM id=0.objet_perdu"
    }
  ]
}
```

The Go gateway converts each internal entry to the public TAP event form (`EVT <category> <type> <data>`).

## Supported operations

| Internal command | Result |
| --- | --- |
| `CONNECT` | Register or restore a player save and emit `ROOM PRESENCE ENTER`. |
| `QUIT` | Save and disconnect the active player; a held lost item returns to `pature`. No `ROOM PRESENCE LEAVE` is emitted. |
| `LOOK` | Return room, exits, players, items, and NPCs as JSON. |
| `MOVE` | Move one player or an entire group across rooms. |
| `TAKE` | Transfer a room item to an inventory and emit `TAKE`. |
| `DROP` | Transfer an inventory item to the room and emit `DROP`. |
| `INVENTORY` | Return inventory item representations as JSON. |
| `USE` | Apply the selected inventory item's behavior (e.g. healing). |
| `TALK` | Advance per-player NPC dialogue. |
| `ATTACK` | Direct 1 HP attack on an NPC (10% chance of NPC counter-attack). |
| `STATUS` | Return health and status as JSON. |
| `QUEST` | Assign an individual or grouped quest. |
| `QUESTS` | Return active quests and completed runs, with progress, as JSON. |
| `FIGHT_CREATE` | Start an individual or grouped C challenge combat. |
| `FIGHT_ATTACK` | Queue a C submission for sandboxed compilation & execution. |
| `GROUP LEAVE` | Record that a player left the group during a fight; the player stays in the fight until it ends. |

### Interactive admin console

When running in an interactive terminal, the server provides a `game_server>` prompt:

| Admin command | Purpose |
| --- | --- |
| `help [command]` | Display available admin commands or details on a specific command. |
| `showitems` | List all item representations currently loaded on the server. |
| `giveitem <player> <item_name>` | Instantiate and give an item directly to a player's inventory. |
| `completequest <player> <quest_name>` | Instantly advance a player's quest to completion. |

## Items and `USE`

Item behavior is resolved by the game engine so inventory checks and effects
are applied atomically. `USE` accepts a protocol identifier or exact item name,
validates ownership, applies the item's effect (e.g. healing up to max HP),
and consumes the instance.

## Quests

Quest definitions contain descriptions, ordered objectives, and probabilistic
rewards (`merci`, `t_shirt_bde`, `wrap_du_foyer`). An NPC can assign a quest to
one player or, through a grouped `QUEST` request, to every eligible member of the
leader's group. Each `QUEST` picks one of the NPC's quests at random among those
the player does not already have active, and starts it immediately (`in progress`).
`QUESTS` returns the active quests followed by one entry per completed run.

Quest texts, step counts, and rewards come from `quests.json`, but the completion
conditions are implemented in `game_manager.rs` and matched by quest name.
Renaming a quest in `quests.json` therefore requires the matching code change.

Gameplay checks advance active quest instances, including campus tours and
selected coding challenges completed within their quest-specific deadlines.
Intermediate progress emits `QUEST STEP`. The game loop detects finished
quests, rolls configured loot chances, adds awarded item instances to the
player's inventory, records completion in player save data, and removes the finished active quest.
`QUEST COMPLETE` carries the quest name and awarded item identifiers to the
client.

> **Note on quest repeatability:** Completing the same quest multiple times (including twice
> or more) is **completely intentional**. Once a quest is finished, it is cleared from active instances
> and logged in the player's completion history. The player can talk to the NPC to accept the quest
> again whenever it is not currently active, allowing players to replay challenges and earn renewable rewards.

## C challenge combat

`FIGHT_CREATE` selects a challenge from `assets/code`; its filename selects the
trusted harness in `assets/tests`. A group leader creates one shared fight for
all eligible members.

For each `FIGHT_ATTACK`, the server:

1. rejects an encoded submission longer than 1,800 bytes (`MAX_CODE_SIZE`, sent to clients as `max_code_size`) with error `416`;
2. restores spaces and newlines from the negotiated TAP separators (`<SP>` and `<NL>`), then re-checks the decoded code against a 64 KiB bound, rejects null bytes, and validates the challenge filename;
3. acquires an execution permit from the sandbox queue (max 2 concurrent sandboxes);
4. compiles the submission together with the trusted test harness via Bubblewrap and Clang (timeout: 8 seconds);
5. runs the compiled binary inside Bubblewrap with resource limits and a strict BPF seccomp filter (timeout: 2 seconds);
6. returns the result via MPSC channel to the game loop, which applies damage and emits events.

The sandbox has no network access, capabilities, writable host mounts,
standard input/output, or normal C runtime. Allowed syscalls are strictly whitelisted
via seccomp (`SYS_write`, `SYS_close`, `SYS_wait4`, `SYS_rt_sigreturn`, `SYS_execve`,
`SYS_exit`, `SYS_exit_group`).

### Damage and death mechanics

- **Successful submission:** Damage is based on target NPC HP at fight creation divided by the
  participant count (clamped to minimum 5 damage). If `dmg * 2 > npc_hp`, it deals `npc_hp` as a finishing blow.
- **Failed submission:** Deals 25 to 50 base damage to the player (`NPC_MIN_DMG` to `NPC_MAX_DMG`).
  Each `t_shirt_bde` equipped reduces NPC damage by 10% (up to 30% reduction).
- **Player death (`HP == 0`):** Death is strictly punitive — **the player loses everything**:
  - The player's state is reset; the reset state replaces the save in `saves/player_saves/` at the next save.
  - All inventory items are lost (non-unique items are recycled; the unique `0.objet_perdu` drops on the floor in the current room).
  - Completed quest history (`completed_quests`) and NPC dialogue progress are wiped.
  - The player respawns at the starting room (`devant_l'école`) with starting HP (100).
  - A `DEATH` event (`respawn_room_id=devant_l'école`) is emitted to the room and spawn room.

Fight creation emits `FIGHT START`; each evaluation emits `FIGHT RESULT`; a
completed or expired instance emits `FIGHT END` with a per-player summary.
When at least one participant died, the surviving participants who did not leave
the group are moved back to `devant_l'école` and receive `TELEPORT`.

During a fight, the engine only accepts `LOOK`, `STATUS`, `FIGHT_ATTACK`,
`GROUP LEAVE`, and `QUIT` from a participant; any other engine command returns
`410 PLAYER_ALREADY_IN_COMBAT`. Chat, `WHO`, and group commands are handled by
the gateway and remain available. A participant who has not submitted when the
222-second deadline expires takes NPC damage, and `QUIT` during a fight without
a submission applies the same damage before disconnecting.

## Design choices

The TAP project specification (RFC 42TAP §6.1 and subject Chapter V.1) intentionally leaves
certain combat, quest, and architectural mechanics underspecified, requiring implementation
teams to document and justify their design decisions. Below is the rationale for each major
design choice implemented in the Rust server:

### 1. Combat system implementation and justification (RFC §6.1.1 & Subject V.1)

- **Dual combat model:**
  The server supports two complementary forms of combat:
  1. *Basic physical attacks (`ATTACK`):* A lightweight, direct command compliant with RFC §5.4.5. It deals 1 damage to an enemy NPC with a 10% chance of triggering an NPC counter-attack (dealing 1 damage to the player). This provides a simple, immediate combat loop.
  2. *Collaborative C challenge combat (`FIGHT_CREATE` & `FIGHT_ATTACK`):* An advanced, retro-themed programming combat mode where defeating powerful NPCs requires writing C functions that pass strict automated unit tests.
- **Turn management and timing:**
  Combat instances enforce a 222-second deadline (`MAX_TIME_FOR_COMBAT`, 3 min 42 s). Within this window, combatants take turns submitting code. Evaluations run asynchronously in background sandbox threads, allowing other players and group members to continue interacting with the world without latency or blocking.
- **Damage formulas & finishing blow threshold:**
  - *Player damage:* `(npc_hp_at_start / participant_count).clamp(5, MAX_DMG_DEALT)`. Distributing damage across group size rewards team coordination while ensuring individual contributors deal meaningful impact.
  - *Finishing blow rule:* If `damage * 2 > current_npc_hp`, the hit instantly deals `current_npc_hp`, defeating the boss. This avoids frustrating end-of-combat scenarios where a boss survives with a sliver of health due to rounding.
  - *NPC retaliation damage:* Failed code submissions trigger NPC retaliations dealing 25 to 50 randomized damage (`NPC_MIN_DMG` to `NPC_MAX_DMG`).
- **Defensive gear (T-Shirts):**
  Defensive scaling is tied to the world item `t_shirt_bde`. Each equipped shirt mitigates NPC retaliation damage by 10% (up to 30% for 3 shirts), offering a tangible gameplay incentive to explore and complete quests.
- **Combat states and additional commands:**
  - States transition cleanly: *Idle* $\to$ *In-Combat* (`FIGHT START`) $\to$ *Evaluating* $\to$ *Resolved* (`FIGHT END`).
  - There is no dedicated `DEFEND` or `FLEE` command. `GROUP LEAVE` during a fight only records the departure: the player stays in the fight until it ends and is not teleported afterwards.
  - `USE` consumes healing items (`wrap_du_foyer`) outside fights to recover health up to the 100 HP ceiling; it is refused during a fight.
- **Punitive permadeath-lite (loss of everything on death):**
  When a player's HP reaches 0, the server executes a full character wipe: the inventory is emptied, completed quests and dialogue progress are cleared, and the player respawns at `devant_l'école` with the starting 100 HP. This design creates genuine stakes for combat encounters and coding submissions, discouraging reckless brute-force attempts.

### 2. Quest system implementation and justification (RFC §6.1.2 & Subject V.1)

- **Multi-faceted progression validation:**
  Quests support both spatial exploration (campus tour checkpoints triggered automatically upon entering rooms via `check_quest_map_tour`) and technical coding challenges (completing assigned C challenges within quest deadlines via `check_complete_code_quest`).
- **Automated tick-based completion:**
  Quest progression and completion are validated automatically on each 20 Hz tick loop (`check_finished_quests`). Once a quest instance reaches or exceeds its target step count, rewards are granted immediately and the quest state is updated, eliminating the need for players to manually turn in finished quests.
- **Probabilistic loot distribution:**
  Completed quests roll for loot against configured probability tables (`merci`, `t_shirt_bde`, `wrap_du_foyer`). Awarded items are dynamically instantiated into unique world instances and placed into the player's inventory, emitting `QUEST COMPLETE`.
- **Intentional quest repeatability:**
  Completing the same quest multiple times (twice or more) is **explicitly intentional**. Once finished, a quest instance is archived into the player's completion history (`completed_quests`) and removed from active instances. Players can revisit NPCs and re-accept quests to re-test their code, obtain consumable healing items (`wrap_du_foyer`), or collect defensive shirts (`t_shirt_bde`).

### 3. Dynamic resource management and persistence (RFC §8 & Subject V.1)

- **Strict instance uniqueness:**
  The server strictly prevents item duplication by distinguishing between static model templates (`items.json`) and runtime item instances. Picking up an item transfers the exact instance from the room to the player; dropping it places that same instance into the room.
- **Lifecycle and identifier recycling:**
  Dropped resources expire after 1 minute (`ITEM_DESPAWN_TIME`). To maintain continuous server stability over long uptimes, expired IDs are reclaimed and recycled via a min-heap (`BinaryHeap<Reverse<ItemId>>`), avoiding numerical identifier bloat.
- **World sustainability and protected items:**
  - *Foyer wraps:* Spawn periodically based on model cooldowns, ensuring players always have a renewable food and healing source during long sessions.
  - *Lost item (`0.objet_perdu`):* Hardcoded unique item that never vanishes: upon despawn or player death, it automatically respawns in `pature`, ensuring vital quest objectives are never permanently lost.
- **Periodic and reactive persistence:**
  While the subject states that persistence is optional, this engine persists state every 2 minutes (`AUTO_SAVE_INTERVAL`), on graceful shutdown, and reactively upon gateway disconnects into TOML files (`saves/`).

### 4. Zero-trust C sandbox architecture (RFC §9 & Subject V.1)

- **Defense-in-depth isolation:**
  Player-submitted C code is treated as completely untrusted. Each evaluation creates an ephemeral directory and executes inside Bubblewrap (`bwrap`) with:
  - Private filesystem namespaces (read-only binds of required compilers and test files, isolated `/work` directory);
  - Dropped Linux capabilities (`--cap-drop ALL`);
  - Cleared environment variables (`--clearenv`);
  - No network connectivity (`--unshare-all`, `--unshare-user`);
  - Resource limits (rlimits on CPU time, memory address space, stack size, open file descriptors, and process creation).
- **Strict BPF seccomp whitelist:**
  Execution binaries run under a compiled Berkeley Packet Filter (BPF) that terminates the process on any syscall not in the explicit 7-call whitelist: `SYS_write`, `SYS_close`, `SYS_wait4`, `SYS_rt_sigreturn`, `SYS_execve`, `SYS_exit`, `SYS_exit_group`.
- **Bounded worker concurrency:**
  A semaphore permit system (`SandboxPermit`) caps active sandboxes to a maximum of 2 concurrent processes. Excess evaluations wait on a condition variable, safeguarding server responsiveness against CPU saturation.

### 5. Architectural concurrency model (Subject Chapter V.1 & VI)

- **Two-tier architecture:**
  Responsibilities are cleanly partitioned:
  - *Go gateway:* Front-facing network server handling client TCP sockets, authentication uniqueness, RFC 42TAP parsing, and event broadcasts.
  - *Rust engine:* Back-end authoritative simulator managing game state, room graphs, combat timers, quests, and sandboxed execution.
- **Single-owner authoritative simulation:**
  To guarantee thread safety without lock contention or deadlocks, the world state (`GameManager`) is strictly owned by the main thread running at 20 Hz (50 ms ticks).
- **Asynchronous I/O via MPSC channels:**
  Network reception threads, admin CLI input, and worker evaluation threads communicate with the engine exclusively through non-blocking MPSC channels.
- **Fault-tolerant gateway reconnects:**
  TCP communication errors with the gateway are caught as `TickResult::Exit`. The engine saves world state, resets transient session state, and returns to `listener.accept()` ready for immediate gateway reconnection.

### 6. Comprehensive server logging (Subject Chapter V.1)

- Leveled text logging using `tracing-subscriber` with microsecond-resolution UTC timestamps.
- Simultaneous dual output: formatted ANSI console output and persistent file logging (`app.log`).
- Comprehensive event coverage: connection/disconnection lifecycles, incoming player commands, server replies, error codes, room transitions, combat results, quest progression, and administrative interventions.

### 7. Interactive administrator CLI

- An embedded `rustyline` command prompt (`game_server>`) runs alongside the server.
- Provides real-time gamemaster tools (`help`, `showitems`, `giveitem <player> <item>`, `completequest <player> <quest>`) to assist debugging, monitor active state, and validate quest chains without stopping the 20 Hz game loop.

## Module layout

| Module | Responsibility |
| --- | --- |
| `main` | CLI arguments, tracing/logging setup, signal handling, TCP listener, and reconnect loop. |
| `game_manager` | World state container, entity lookup, item instantiation/recycling, save/restore. |
| `simulation` | Authoritative 20 Hz tick loop, incoming message dispatch, NPC revival, and item spawning. |
| `commands` | Single-player, grouped, question JSON handlers, event builders, and combat calculations. |
| `admin_commands` | Interactive console administration commands (`game_server>`). |
| `player_response` | Batching and transmission of per-player tick diff events to the gateway. |
| `parser` | Deserialization and cross-reference validation for `npcs.json`, `items.json`, `rooms.json`, and `quests.json`. |
| `room`, `player`, `npc`, `items`, `inventory`, `quests`, `combat_instances` | Core domain models and entities. |
| `save` | Structs (`Save`, `SavedItem`, `ServerSave`) for TOML persistence with `confy`. |
| `tester` | Clang/Bubblewrap sandbox runners, resource limits, permit queue, and BPF seccomp filters. |
| `constants` | Timings, damage constants, error codes, and global configuration values. |
| `logs` | Console and file logging channels and custom writer formatting. |

## Validation

```bash
make lint-rust-server
```

The lint target runs `cargo clippy --all-targets` and `cargo fmt --all --check`.
