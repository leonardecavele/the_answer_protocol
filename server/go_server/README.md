The Go server is the public endpoint for The Answer Protocol. It accepts TCP
clients, performs the TAP handshake, authenticates usernames, owns chat and
group state, applies connection limits, and translates game commands to the
[Rust engine's](../rust_server/README.md) internal JSON protocol.

Public command syntax, errors, and events are defined in the root
[TAP protocol reference](../../PROTOCOL.md).

## Requirements

- Go 1.18 or newer
- A reachable Rust game server for world operations

## Build and run

Run these commands from the repository root:

```bash
make build-go-server
make run-go-server
```

| Flag | Default | Purpose |
| --- | --- | --- |
| `--go-server-port` | `38800` | Public TAP listening port. |
| `--rust-server-ip` | `localhost` | Rust game-server hostname or IP. |
| `--rust-server-port` | `38801` | Rust game-server port. |

Example:

```bash
make run-go-server \
  GO_SERVER_ARGS="--go-server-port 38800 --rust-server-ip 127.0.0.1 --rust-server-port 38801"
```

Ports must be between 1 and 65535. Stop the process with `SIGINT`, `SIGTERM`,
or by entering `quit`, `exit`, or `q` on standard input.

## Client lifecycle

For each accepted connection, the gateway:

1. sends `OK hello proto=1`;
2. allows 30 seconds for `CONNECT`;
3. canonicalizes and reserves the username;
4. processes TAP commands while independently delivering events;
5. releases session and group state when the client leaves;
6. notifies the game engine and the remaining authenticated clients.

Each connection has its own reader goroutine and event writer. Mutex-protected
registries coordinate usernames, rooms, groups, invitations, rate windows, and
outbound event queues.

Command names and their subcommands are handled case-insensitively. Dispatch is
organized by domain-specific handler maps for core, chat, group, resource,
quest, and fight operations.

## Limits and timeouts

| Limit | Value |
| --- | --- |
| Concurrent client connections | 20 |
| Maximum TAP frame | 4,096 bytes |
| Authentication timeout | 30 seconds |
| Client read timeout | 30 minutes |
| Socket write timeout | 5 seconds |
| Client input frames per IP | 25 per second |
| Connection attempts per IP | 20 per second |
| Flood violations before IP ban | 5 |
| Flood-point decay | 1 point every 30 minutes and IP |
| Rust command response timeout | 3 seconds |
| Rust question response timeout | 5 seconds |
| Group size | 5 players |
| Group invitation lifetime | 5 minutes |

Every input frame is counted before parsing, including an invalid `CONNECT`, and
the rate window is shared by all connections from the same IP. Exceeding the
input or connection-attempt rate adds one flood point, capped at five. Five
points ban that IP until the 30-minute decay brings it below five points.

Before logging or dispatching a frame, the gateway rejects invalid UTF-8 and
control characters other than the terminating `LF` and its optional preceding
`CR` with `ERR 400 INVALID_ARGUMENTS`.

## Command ownership

The gateway handles these concerns directly:

- `CONNECT`, `QUIT`, and `WHO` session behavior;
- global, group, and private chat;
- group creation, invitations, joining, and leaving;
- public TAP parsing, validation, responses, errors, and event formatting.

It forwards commands that need authoritative game state:

| Public operation | Internal command | Envelope |
| --- | --- | --- |
| `LOOK` | `LOOK` | Single player |
| `MOVE` | `MOVE` | Single player or group |
| `TAKE` | `TAKE` | Single player |
| `DROP` | `DROP` | Single player |
| `INVENTORY` | `INVENTORY` | Single player |
| `USE` | `USE` | Single player |
| `TALK` | `TALK` | Single player |
| `ATTACK` | `ATTACK` | Single player |
| `STATUS` | `STATUS` | Single player |
| `QUEST` | `QUEST` | Single player or group |
| `QUESTS` | `QUESTS` | Single player |
| `FIGHT CREATE` | `FIGHT_CREATE` | Single player or group |
| `FIGHT ATTACK` | `FIGHT_ATTACK` | Single player |

Room chat and same-room group checks use the correlated `ROOM_PLAYERS`
question.

Group creation, invitations, and joining require an available game server.
Inviting and joining also check that the players share a room. Leaving remains
available when the game server is disconnected.

Invitation cleanup emits `GROUP INVITE <leader> REMOVED` so clients can remove
stale invitations. Expired invitations are cleaned up when a new invitation is
processed; disbanding a group removes its outstanding invitations. When the
leader leaves, remaining members receive `GROUP LEAVE <leader>`, allowing both
frontends to clear group state and notify the player.

## Internal JSON messages

Messages are compact UTF-8 JSON objects followed by `LF`.

### Single-player command

```json
{
  "player": "ALICE",
  "command": "MOVE",
  "data": "NORTH"
}
```

### Grouped command

```json
{
  "leader": "ALICE",
  "grouped_players": ["BOB", "CHARLIE"],
  "command": "QUEST",
  "data": "4.gagulhon"
}
```

The leader is not repeated in `grouped_players`.

### Correlated question

```json
{
  "question": "ROOM_PLAYERS",
  "data": "ALICE",
  "id": "c94d8d2b-..."
}
```

The response repeats `question` and `id`; its `data` field contains a JSON
array encoded as a string.

### Game response

```json
{
  "player": "ALICE",
  "command": "MOVE",
  "error_code": 0,
  "data": "6.entree"
}
```

A zero `error_code` is successful. Other values are mapped to the
command-specific public TAP error.

### Event batch

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

The gateway formats that event as:

```text
EVT ROOM PRESENCE ENTER BOB
```

It accepts a single batch or an array of batches and routes each one to the
selected authenticated client.

## Rust connection management

A dedicated manager owns the single outbound connection to the game engine.
It serializes writes, routes command responses to their waiting handlers,
correlates question answers, dispatches targeted event batches, and reconnects
when the backend becomes available again. Client sessions receive game-server
availability events, and authenticated players are registered again after a
reconnection.

## Package layout

| Package | Responsibility |
| --- | --- |
| `client_conn` | TAP connection loop, parsing, and dispatch. |
| `client_conn/tap_commands` | Domain-specific command handlers. |
| `config` | Ports, protocol limits, timeouts, and CLI parsing. |
| `game_conn` | Rust connection, JSON envelopes, routing, and questions. |
| `protocol` | TAP response strings, event formatting, and error mapping. |
| `session` | Clients, usernames, groups, invitations, and rate windows. |
| `logger` | Structured process logging. |

## Logging

The logger emits informational records to stdout, error records to stderr, and
all records to `app.log`. Each record contains a microsecond timestamp, severity,
and message:

```text
15:04:05.123456 INFO client connected remote=127.0.0.1:52144
```

The same ANSI-colored records are written to the console and log file.
Consumers that need plain text must strip the escape sequences. The file is
reset at process start.

## Validation

```bash
make lint-go-server
(cd server/go_server && go test ./...)
```

The lint target checks formatting with `gofmt` and runs `go vet`.
