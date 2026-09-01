# Go TAP Server

The Go server is the public endpoint for The Answer Protocol. It accepts TAP
clients, performs the protocol handshake, authenticates usernames, maintains
chat and group state, and translates game commands between TAP and the Rust
server's internal JSON protocol.

See the [server architecture](../README.md) for the Go-to-Rust wire contract and
the root [TAP documentation](../../TAP_COMMANDS.md) for the public client
protocol.

## Requirements

- Go 1.18 or newer;
- a running Rust server for world, item, NPC, quest, status, and combat
  commands.

The Go server can start without Rust. `CONNECT`, `WHO`, global/private/group
chat for existing groups, and `GROUP CREATE` or `GROUP LEAVE` remain available,
while commands that need game state return `ERR 900 CONNECTION_FAILED`.
Invitation and join behavior during a Rust outage is unsafe and is described
under known implementation gaps.

## Build and run

From the repository root:

```bash
make build-go-server
make run-go-server
```

Directly:

```bash
cd server/go_server
go build
./go_server \
  --go-server-port 38800 \
  --rust-server-ip 127.0.0.1 \
  --rust-server-port 38801
```

| Flag | Default | Purpose |
| --- | --- | --- |
| `--go-server-port` | `38800` | Public TAP listening port. |
| `--rust-server-ip` | `localhost` | Rust game-server hostname or IP. |
| `--rust-server-port` | `38801` | Rust game-server port. |

Ports must be between 1 and 65535. Unexpected positional arguments and an empty
Rust host are rejected.

Stop the process with `SIGINT`, `SIGTERM`, or by entering `quit`, `exit`, or `q`
on standard input.

## Runtime behavior

### Client lifecycle

1. Accept a TCP connection and send `OK hello proto=1`.
2. Give the client 30 seconds to authenticate with `CONNECT`.
3. Process newline-delimited TAP commands and asynchronous events.
4. On `QUIT`, EOF, timeout, or error, release the username and group, notify
   other clients, and send a best-effort internal `QUIT` to Rust.

Authenticated usernames are stored in uppercase. The server supports at most 20
concurrent connections and groups of at most three players.

### Limits and timeouts

| Limit | Value |
| --- | --- |
| Maximum TAP frame | 4,096 bytes |
| Authentication timeout | 30 seconds |
| Client read timeout | 30 minutes |
| Socket write timeout | 5 seconds |
| Commands per client | 10 per second |
| Connection attempts per host | 5 per 10 seconds |
| Rust command response timeout | 3 seconds |
| Rust question response timeout | 5 seconds |
| Group invitation lifetime | 5 minutes |

Exceeding the command rate sends `ERR 429 TOO_MANY_REQUESTS` and closes the
connection. Pre-handler connection limits reject the socket without a TAP error
frame.

### Rust connection

The server retries the Rust endpoint every five seconds. A disconnect clears
the active game connection and broadcasts `EVT GAME SERVER DISCONNECTED`.
After reconnecting, it re-registers all authenticated usernames and broadcasts
`EVT GAME SERVER CONNECTED`.

## Command ownership

Handled primarily in Go:

- handshake, `CONNECT`, `QUIT`, and `WHO`;
- all chat scopes;
- group creation, invitations, joins, leaves, and group event routing.

Forwarded to Rust:

- `LOOK`, `MOVE`, `TAKE`, `DROP`, `INVENTORY`, `TALK`, `ATTACK`, and `STATUS`;
- `QUEST`, `QUESTS`, `FIGHT CREATE`, and `FIGHT ATTACK`.

`CHAT ROOM` and same-room group checks use the internal `ROOM_PLAYERS`
question. Grouped `MOVE` and `FIGHT CREATE` include all group members in one
internal command.

## Package layout

| Package | Responsibility |
| --- | --- |
| `client_conn` | TAP connection loop, parsing, and command dispatch. |
| `client_conn/tap_commands` | Core, chat, group, resource, and fight handlers. |
| `config` | Protocol, ports, limits, timeouts, and CLI parsing. |
| `game_conn` | Rust connection lifecycle, JSON envelopes, response routing, and questions. |
| `protocol` | Public response strings, event formatting, and error mapping. |
| `session` | Clients, usernames, groups, event delivery, and rate windows. |
| `logger` | Process logging. |

## Logging

At startup, the server creates or truncates `app.log` in its working directory
and logs to both standard output and that file. Run it from `server/go_server`
when you want the log to remain beside the server source.

## Known implementation gaps

- `USE` is registered as a TAP command but has no Rust handler and always fails
  or times out. During a fight, Rust's pre-dispatch code `410` is unmapped for
  `USE` and becomes `UNKNOWN_ERROR`.
- `GROUP QUIT` is a raw alias of `GROUP LEAVE`; the typed client only exposes
  leave.
- A grouped `QUEST` is forwarded as a grouped internal command, but Rust only
  accepts grouped `MOVE` and `FIGHT_CREATE`; grouped quest requests normally
  time out. During a fight, the pre-dispatch code `410` becomes
  `UNKNOWN_ERROR` because it is not mapped for `QUEST`.
- Group invitation and join state depends on Rust-backed same-room checks when
  the game server is connected. During an outage, `GROUP INVITE` sends a notice
  but fails to store it, and `GROUP JOIN` can panic after skipping assignment.
  Do not use either operation until Rust is connected.

## Verification

```bash
go test ./...
go vet ./...
```

The repository currently has no Go behavioral test files; `go test` verifies
that every package compiles.
