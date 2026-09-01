# TAP Errors

This document describes the error frames that the Go TAP server can send to a
client. The catalog is derived from the Go error response definitions and the
error codes returned by the Rust game server.

## Frame format

TAP uses UTF-8, line-oriented frames. Every error ends with `LF` on the wire.

```abnf
error-line    = "ERR" SP error-code SP error-name LF
error-code    = 3DIGIT
error-name    = 1*(ALPHA / DIGIT / "_")
```

Example:

```text
ERR 404 NPC_NOT_FOUND
```

Codes are not unique identifiers by themselves. Several error names share a
code, so clients should retain both the numeric code and the symbolic name.

## Protocol errors

| Code | Error name | Meaning |
| --- | --- | --- |
| `201` | `NAME_IN_USE` | The requested username is already authenticated on the Go server. |
| `204` | `NO_CONTENT` | The Rust server returned no usable command data. This code is defined and routed globally, but is not emitted by a current command handler. |
| `301` | `NO_EXIT` | The direction is invalid or the current room has no exit in that direction. |
| `400` | `ALREADY_CONNECTED` | The client sent `CONNECT` after it was already authenticated. |
| `400` | `NOT_CONNECTED` | The command requires an authenticated TAP session. |
| `400` | `INVALID_USERNAME` | The username violates the TAP username rules. |
| `400` | `ROOM_FULL` | The Go server already has its maximum number of authenticated clients. |
| `400` | `GROUP_FULL` | The target group already contains three players. |
| `400` | `EMPTY_COMMAND` | The client sent an empty command line. |
| `400` | `COMMAND_NOT_FOUND` | The top-level command or group/fight subcommand is unknown. |
| `400` | `INVALID_ARGUMENTS` | Arguments are missing, unexpected, or contain invalid surrounding whitespace. |
| `400` | `INVALID_SCOPE` | The `CHAT` scope is not `GLOBAL`, `ROOM`, `GROUP`, or `PRIVATE`. |
| `401` | `NOT_IN_GROUP` | The requested operation requires group membership. |
| `402` | `ALREADY_IN_GROUP` | The player or invitation target already belongs to a group. |
| `403` | `NO_SUCH_USER` | The requested user is not authenticated on the Go server. |
| `403` | `NOT_INVITED` | The player has no current invitation to the target group. Invitations expire after five minutes. |
| `403` | `NOT_GROUP_LEADER` | Only the group leader may perform the requested grouped action. |
| `404` | `ITEM_NOT_FOUND` | The item identifier cannot be resolved in the current room. |
| `404` | `ITEM_NOT_IN_INVENTORY` | The item is not in the player's inventory. |
| `404` | `NPC_NOT_FOUND` | The NPC identifier cannot be resolved. |
| `404` | `GROUP_NOT_FOUND` | The referenced group no longer exists. |
| `404` | `NO_SUCH_GROUP` | A requested group cannot be found. This name is defined for routed game-server errors. |
| `405` | `PLAYER_NOT_FOUND` | The Rust game server does not know the player. |
| `405` | `NPC_NOT_HOSTILE` | The selected NPC cannot be attacked. |
| `406` | `NO_QUEST_AVAILABLE` | The NPC has no quest currently available to the player. |
| `407` | `NPC_NOT_IN_ROOM` | The selected NPC is not in the player's room. |
| `407` | `NOT_IN_SAME_ROOM` | A group invitation or join target is not in the same room. |
| `408` | `NPC_IN_COMBAT` | The NPC already belongs to another combat instance. |
| `409` | `ACTION_ALREADY_TAKEN` | The player already submitted an action for the current combat round. This name is defined, but the current `FIGHT ATTACK` map exposes that Rust error as `UNKNOWN_ERROR`. |
| `410` | `PLAYER_ALREADY_IN_COMBAT` | The command is unavailable while the player is in a combat instance, or a new combat was requested for a player already fighting. Go maps it for `MOVE` and `FIGHT CREATE`; on other forwarded commands the same Rust code becomes `UNKNOWN_ERROR`. |
| `411` | `PLAYER_NOT_IN_COMBAT` | `FIGHT ATTACK` was sent without an active combat instance. |
| `412` | `FILE_NOT_FOUND` | The Rust server could not load the challenge source file for a fight. |
| `413` | `ROOM_NOT_FOUND` | The Rust server cannot resolve the player's current room. |
| `429` | `TOO_MANY_REQUESTS` | More than ten commands were received within one second. The Go server sends this error and then closes the client connection. |
| `900` | `CONNECTION_FAILED` | The Rust game server is unavailable or a Go handler failed while communicating with it. |
| `901` | `SEND_FAILED` | A command could not be transmitted between the servers. This mapping is defined, but no current Rust command handler emits it. |
| `902` | `GAME_SERVER_TIMEOUT` | The Rust server did not answer a forwarded command within three seconds. |
| `997` | `INVALID_GROUP_COMMAND` | Reserved for a rejected internal grouped-command envelope. The current Rust validation path does not emit it. |
| `998` | `INVALID_QUESTION` | Reserved for a rejected internal question envelope. The current Rust validation path does not emit it. |
| `999` | `INVALID_COMMAND` | The Rust server rejected an internal command envelope or invalid JSON. |
| `999` | `UNKNOWN_ERROR` | The Go server received an unmapped, invalid, or unexpected game-server error code. |

## Where errors originate

The Go server generates session and syntax errors directly, including
`INVALID_USERNAME`, `NOT_CONNECTED`, group errors, rate limiting, and failures
to reach the Rust server.

The Rust server returns numeric error codes inside its internal JSON response.
The Go server maps the code through the command-specific response table before
it sends a TAP error. If the code is not valid for the pending command, the
client receives `ERR 999 UNKNOWN_ERROR`.

The following failures close the TCP connection instead of guaranteeing a TAP
error frame:

- a frame larger than 4,096 bytes;
- a client read timeout;
- a socket read or write failure;
- failure to authenticate within 30 seconds;
- rejection before the TAP client handler starts because the connection or
  per-host connection-attempt limit was reached.

## Client-side errors

The Rust `api-client` also exposes local `NetworkError`, `ProtocolError`, and
`InternalError` values. These are client-library failures, not `ERR` frames, and
therefore are not assigned TAP error codes.
