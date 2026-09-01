# TAP Commands

This document describes the public, line-oriented protocol exposed by the Go
server. It reflects the current Go command handlers, the Rust game-server
responses, and the typed commands available in `api-client`.

## Transport and framing

TAP is a UTF-8 protocol over TCP. Each command and response occupies one line.
The Go server accepts `LF` and trims an optional preceding `CR`; it always writes
`LF`.

```abnf
command-line  = command-name [SP arguments] [CR] LF
command-name  = 1*ALPHA
arguments     = 1*(VCHAR / SP / utf8-nonascii)

success-line  = "OK" [SP response-data] LF
response-data = 1*(VCHAR / SP / utf8-nonascii)
utf8-nonascii = <a valid non-ASCII UTF-8 sequence>
```

Client frames, including the line ending, are limited to 4,096 bytes. A client
should keep at most one command awaiting an `OK` or `ERR` response on a
connection, while continuing to process any interleaved `EVT` frames.

Top-level command names are case-sensitive and must be uppercase. Group and
fight subcommands, and chat scopes, are normalized by the Go server. Movement
directions are interpreted by Rust and must be `NORTH`, `SOUTH`, `EAST`, or
`WEST`.

## Handshake and authentication

Immediately after accepting a TCP connection, the Go server sends:

```text
OK hello proto=1
```

Protocol version 1 is the only version accepted end to end by the current
`api-client`. The client then has 30 seconds to authenticate with `CONNECT`.

A username:

- contains between 3 and 20 ASCII characters;
- starts with an ASCII letter;
- contains only ASCII letters, digits, `_`, or `-` after the first character.

The Go server canonicalizes accepted usernames to uppercase.

## Support matrix

| TAP command | Implemented by | Typed `api-client` command | Status |
| --- | --- | --- | --- |
| `CONNECT`, `LOOK`, `MOVE`, `QUIT`, `WHO` | Go, with game state from Rust where required | Yes | Supported |
| `CHAT GLOBAL`, `CHAT PRIVATE` | Go | Yes | Supported |
| `CHAT ROOM`, `CHAT GROUP` | Go; room delivery asks Rust for `ROOM_PLAYERS` | No | Supported through raw TAP only |
| `GROUP CREATE`, `GROUP INVITE`, `GROUP JOIN`, `GROUP LEAVE` | Go; room checks ask Rust when available | Yes | Supported |
| `GROUP QUIT` | Go | No | Raw alias of `GROUP LEAVE` |
| `TAKE`, `DROP`, `INVENTORY`, `TALK`, `ATTACK`, `STATUS`, `QUESTS`, and solo `QUEST` | Go forwards to Rust | Yes | Supported, subject to the compatibility notes below |
| Grouped `QUEST` | Go sends a grouped envelope, but Rust has no grouped quest handler | Yes | Incomplete; currently times out |
| `FIGHT CREATE`, `FIGHT ATTACK` | Go forwards to Rust | Yes | Supported |
| `USE` | Go forwards it, but Rust has no handler | No | Incomplete; do not use |

## Common failures

All commands are subject to syntax, authentication, rate-limit, and connection
errors. In particular:

```text
ERR 400 EMPTY_COMMAND
ERR 400 COMMAND_NOT_FOUND
ERR 400 INVALID_ARGUMENTS
ERR 400 NOT_CONNECTED
ERR 429 TOO_MANY_REQUESTS
ERR 900 CONNECTION_FAILED
ERR 902 GAME_SERVER_TIMEOUT
ERR 999 UNKNOWN_ERROR
```

`TOO_MANY_REQUESTS` is sent after more than ten commands in one second and is
followed by connection closure. `CONNECTION_FAILED` and
`GAME_SERVER_TIMEOUT` apply only when a command or routing decision needs the
Rust server. The complete catalog is in `TAP_ERRORS.md`.

## Core commands

### CONNECT

```text
CONNECT <username>
```

```text
OK connected
```

`CONNECT` authenticates the Go session and registers the player with Rust when
the game server is available. A missing Rust server does not prevent Go-side
authentication; the client then receives `EVT GAME SERVER DISCONNECTED`.

Command-specific failures include `INVALID_USERNAME`, `NAME_IN_USE`,
`ALREADY_CONNECTED`, and `ROOM_FULL`.

### LOOK

```text
LOOK
```

```text
OK <room-state-json>
```

Example payload:

```json
{
  "room": {
    "id": "2.devant_l_ecole",
    "name": "Devant l'école",
    "description": "A large 42 sign marks the entrance.",
    "exits": {
      "WEST": "Entree",
      "SOUTH": "Pature"
    }
  },
  "players": ["ALICE", "BOB"],
  "items": ["0.objet_perdu"],
  "npcs": ["4.gagulhon", "11.ayteyssi"]
}
```

The `id` and resource values are protocol representations. Display names and
exit destinations are formatted by Rust. Command-specific failures include
`PLAYER_NOT_FOUND` and `ROOM_NOT_FOUND`.

### MOVE

```text
MOVE <NORTH|SOUTH|EAST|WEST>
```

```text
OK room=<room-identifier>
```

If the player belongs to a group, only the leader may send `MOVE`; Rust moves
the other group members with the leader and emits `GROUPMOVE` events to them.
Command-specific failures include `NO_EXIT`, `NOT_GROUP_LEADER`,
`PLAYER_NOT_FOUND`, and `PLAYER_ALREADY_IN_COMBAT`.

### WHO

```text
WHO
```

```text
OK players=<count>
```

The count is the number of players authenticated on this Go server. It does not
require the Rust server.

### QUIT

```text
QUIT
```

```text
OK bye
```

After sending the response, Go closes the connection, releases group state,
notifies other clients, and sends a best-effort internal `QUIT` to Rust.

## Chat commands

### Global chat

```text
CHAT GLOBAL <message>
```

The sender receives `OK`. Every other authenticated client receives:

```text
EVT GLOBAL CHAT <username> <message>
```

### Room chat

```text
CHAT ROOM <message>
```

The sender receives `OK`. Go asks Rust for the players in the sender's room and
routes the event to those players:

```text
EVT ROOM CHAT <username> <message>
```

This command requires a working Go-to-Rust connection.

### Group chat

```text
CHAT GROUP <message>
```

The sender receives `OK`. Other group members receive:

```text
EVT GROUP CHAT <username> <message>
```

The sender receives `NOT_IN_GROUP` when no group is active.

### Private chat

```text
CHAT PRIVATE <username> <message>
```

The sender receives `OK`. The named recipient receives:

```text
EVT PRIVATE CHAT <sender> <message>
```

An unknown recipient returns `NO_SUCH_USER`.

All chat messages must contain at least one non-whitespace character. An unknown
scope returns `INVALID_SCOPE`.

## Group commands

Groups are maintained by the Go server. A group contains at most three players,
and invitations expire after five minutes.

### GROUP CREATE

```text
GROUP CREATE
```

```text
OK group=<group-id>
```

The authenticated player becomes the group leader. An existing membership
returns `ALREADY_IN_GROUP`.

### GROUP INVITE

```text
GROUP INVITE <username>
```

```text
OK
```

When the Rust server is connected, the inviter and target must be in the same
game room. The target receives `EVT GROUP INVITE <leader>`. Possible failures
include `NOT_IN_GROUP`, `NO_SUCH_USER`, `ALREADY_IN_GROUP`, `GROUP_FULL`,
`GROUP_NOT_FOUND`, and `NOT_IN_SAME_ROOM`.

During a Rust outage, the current handler still returns `OK` and sends the
invite event, but does not record the invitation. The recipient cannot use that
notice to join a group.

### GROUP JOIN

```text
GROUP JOIN <leader-or-member-username>
```

```text
OK group=<group-id>
```

The argument may name any connected member of the invited group. Joining
requires a valid invitation and, when Rust is connected, the same game room.
Existing members receive `EVT GROUP JOIN <username>`. Possible failures include
`NO_SUCH_USER`, `ALREADY_IN_GROUP`, `NOT_INVITED`, `GROUP_FULL`,
`GROUP_NOT_FOUND`, and `NOT_IN_SAME_ROOM`.

`GROUP JOIN` must not be attempted while Rust is disconnected. The current Go
handler skips group assignment and then dereferences the missing membership,
which can terminate the Go process.

### GROUP LEAVE

```text
GROUP LEAVE
```

```text
OK
```

`GROUP QUIT` is accepted as an undocumented-client alias with the same behavior.
If the leader leaves, Go dissolves the group. Other members receive
`EVT GROUP LEAVE <username>`. A player without a group receives `NOT_IN_GROUP`.

## Resource and NPC commands

An item or NPC can normally be addressed by its `<numeric-id>.<name>` protocol
representation. Rust also accepts a unique exact name in the relevant room or
inventory.

### TAKE

```text
TAKE <item-identifier>
```

```text
OK taken=<item-identifier>
```

Other players in the room receive `EVT TAKE <username> <item-identifier>`.
Failures include `ITEM_NOT_FOUND` and `PLAYER_NOT_FOUND`.

### DROP

```text
DROP <item-identifier>
```

```text
OK dropped=<item-identifier>
```

Other players in the room receive `EVT DROP <username> <item-identifier>`.
Failures include `ITEM_NOT_IN_INVENTORY` and `PLAYER_NOT_FOUND`. Rust may use
its shared numeric `404` code while resolving the item; Go maps every `DROP`
error with that code to `ITEM_NOT_IN_INVENTORY`.

### INVENTORY

```text
INVENTORY
```

```text
OK ["0.objet_perdu","2.t_shirt"]
```

The response data is a JSON array of item protocol representations.

### TALK

```text
TALK <npc-identifier>
```

```text
OK <dialogue-text>
```

The response advances that player's dialogue with the NPC. Rust uses
`[end of dialogue]` to mark the end of a dialogue sequence. Failures include
`NPC_NOT_FOUND` and `PLAYER_NOT_FOUND`. Although Go defines an
`NPC_NOT_IN_ROOM` mapping for `TALK`, the current Rust handler folds a room
mismatch into `NPC_NOT_FOUND`.

### ATTACK

```text
ATTACK <npc-identifier>
```

```text
OK <combat-result-json>
```

```json
{
  "attacker_hp": 100,
  "target_hp": 149,
  "damage": 1,
  "status": "combat"
}
```

This is the legacy direct-damage command, separate from the code-challenge
`FIGHT` flow. Failures include `NPC_NOT_FOUND`, `NPC_NOT_IN_ROOM`,
`NPC_NOT_HOSTILE`, and `NPC_IN_COMBAT`. If the player is already in a
code-challenge fight, Rust returns code `410`; Go does not map that code for
`ATTACK`, so the TAP response is `UNKNOWN_ERROR`.

### STATUS

```text
STATUS
```

```text
OK <player-status-json>
```

```json
{
  "hp": 80,
  "max_hp": 100,
  "status": "healthy"
}
```

The status is `healthy` at 80% HP or above, `normal` at 30% or above but below
80%, and `critical` below 30%.

## Quest commands

### QUEST

```text
QUEST <npc-identifier>
```

```text
OK <quest-json>
```

```json
{
  "name": "Tunnel",
  "description": "Complete the requested objective.",
  "reward": [
    {
      "qty": 1,
      "chance": 100,
      "type": "MERCI"
    }
  ],
  "status": "in progress"
}
```

If the player belongs to a group, Go permits only the leader and sends a grouped
internal `QUEST`. Rust does not implement grouped quest envelopes, so this path
normally ends in `GAME_SERVER_TIMEOUT`. If the leader is already in a fight,
Rust's pre-dispatch combat guard instead returns code `410`; because that code
is not mapped for `QUEST`, Go exposes it as `UNKNOWN_ERROR`. Solo failures
include `NPC_NOT_FOUND`, `NPC_NOT_IN_ROOM`, `PLAYER_NOT_FOUND`, and
`NO_QUEST_AVAILABLE`; a non-leader receives `NOT_GROUP_LEADER` before anything
is forwarded.

The current Rust wire object uses `name`. The current `api-client::QuestData`
expects `quest_id`, so a non-empty quest response does not deserialize through
the typed client until that implementation mismatch is fixed.

### QUESTS

```text
QUESTS
```

```text
OK [<quest-json>,...]
```

The response is a JSON array using the same quest object shape as `QUEST`.
`[]` is valid when the player has no active quest. The same `name` versus
`quest_id` compatibility issue applies to non-empty arrays.

## Code-challenge fight commands

### FIGHT CREATE

```text
FIGHT CREATE <npc-identifier>
```

```text
OK FIGHT CREATED
```

The NPC must be a hostile NPC in the player's room and must not already be in a
combat instance. A group leader creates one combat instance for the group;
non-leaders receive `NOT_GROUP_LEADER`. Rust sends each participant an
`EVT FIGHT START` payload containing the source code, separators, time limit,
and NPC health.

Failures include `NPC_NOT_FOUND`, `NPC_NOT_IN_ROOM`, `NPC_NOT_HOSTILE`,
`NPC_IN_COMBAT`, `PLAYER_ALREADY_IN_COMBAT`, and `FILE_NOT_FOUND`.

### FIGHT ATTACK

```text
FIGHT ATTACK <encoded-code>
```

```text
OK Processing
```

`encoded-code` is a single-line source submission. Clients must replace spaces
and newlines with the `sp_sep` and `nl_sep` values received in `EVT FIGHT START`.
Evaluation is asynchronous; results arrive through `EVT FIGHT RESULT`, followed
eventually by `EVT FIGHT END`.

Failures include `PLAYER_NOT_FOUND` and `PLAYER_NOT_IN_COMBAT`. Rust also emits
code `409` for a duplicate action, but Go's current `FIGHT_ATTACK` map does not
associate that code with `ACTION_ALREADY_TAKEN`; the public result is therefore
`ERR 999 UNKNOWN_ERROR` for that case.

## Incomplete USE command

The Go listener currently recognizes:

```text
USE <arguments>
```

It forwards an internal `USE` command, but the Rust server has no matching
handler and `api-client` exposes no typed command. With Rust connected, the Go
handler normally returns `GAME_SERVER_TIMEOUT`. If the player is already in a
fight, Rust's global combat guard returns code `410` before command dispatch;
Go has no `USE` error map, so this becomes `UNKNOWN_ERROR`. Without Rust, the
result is `CONNECTION_FAILED`. `USE` is therefore not part of the supported TAP
contract.
