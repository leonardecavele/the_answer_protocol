The Go server is the public endpoint for The Answer Protocol. It accepts TAP
clients, performs the protocol handshake, authenticates usernames, maintains
chat and group state, and translates game commands between TAP and the Rust
server's internal JSON protocol.

See the [server architecture](../README.md) for the Go-to-Rust wire contract.

## Requirements

- Go 1.18 or newer;
- a running Rust server for world, item, NPC, quest, status, and combat
  commands.

The Go server can accept sessions while Rust is unavailable. Operations owned
entirely by the gateway remain available, while world operations report the
backend failure defined in the root
[TAP error reference](../../README.md#tap-errors).

## Build and run

From the repository root:

```bash
make build-go-server
make run-go-server
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

The gateway maintains one connection to the Rust game server and coordinates
pending commands, questions, disconnect events, and player registration after
a reconnect. See the server [routing and timing](../README.md#routing-and-timing)
section for the retry sequence and response deadlines.

## Command ownership

Go owns the public connection, authentication, session, chat, and group layers.
Commands requiring authoritative world state are translated and forwarded to
Rust.

See the server [internal command matrix](../README.md#internal-command-matrix)
for exact routing, grouped envelopes, response handling, and internal
questions. See the root [TAP command reference](../../README.md#tap-commands)
for the complete public wire syntax and behavior.

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

See the root [Server Logging](../../README.md#server-logging) section for the Go
log format, event coverage, output destinations, monitoring commands, and abuse
detection guidance.
