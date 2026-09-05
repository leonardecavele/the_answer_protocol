This document is the wire-level reference for the public TAP endpoint exposed
by the Go server. It consolidates the protocol grammar, commands, responses,
errors, events, and the implementation choices permitted by RFC 42TAP.

## Transport and framing

TAP is a UTF-8, line-oriented protocol carried over TCP. Each client command
and each server frame occupies one line. The server accepts `LF`, optionally
preceded by `CR`, and emits `LF`.

```abnf
command-line  = command-name [SP arguments] [CR] LF
command-name  = 1*ALPHA
arguments     = 1*(VCHAR / SP / utf8-nonascii)

success-line  = "OK" [SP response-data] LF
error-line    = "ERR" SP error-code SP error-name LF
event-line    = "EVT" SP event-content LF

error-code    = 3DIGIT
error-name    = 1*(ALPHA / DIGIT / "_")
utf8-nonascii = <a valid non-ASCII UTF-8 sequence>
```

Client frames, including the line ending, are limited to 4,096 bytes. A client
keeps at most one command awaiting an `OK` or `ERR` response while continuing
to process any interleaved `EVT` frames.

Command names, subcommands, chat scopes, and movement directions are
case-insensitive. Examples use uppercase for readability.

## Connection lifecycle

Immediately after accepting a TCP connection, the server sends:

```text
OK hello proto=1
```

The client then has 30 seconds to authenticate with `CONNECT`. A username:

- contains between 3 and 20 ASCII characters;
- starts with an ASCII letter;
- contains only ASCII letters, digits, `_`, or `-` after the first character.

Accepted usernames are canonicalized to uppercase.

## Commands

### Session and navigation

#### CONNECT

```text
CONNECT <username>
OK connected
```

Authenticates the session and registers the player in the world. Relevant
errors include `INVALID_USERNAME`, `NAME_IN_USE`, `ALREADY_CONNECTED`, and
`ROOM_FULL`.

#### LOOK

```text
LOOK
OK <room-state-json>
```

Example response payload:

```json
{
  "room": {
    "id": "2.devant_l'école",
    "name": "Devant l'école",
    "description": "Un grand panneau 42 vous accueille à la meilleure école du numérique.",
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

#### MOVE

```text
MOVE <NORTH|SOUTH|EAST|WEST>
OK room=<room-identifier>
```

Only a group leader moves a group; the other members follow automatically.
Relevant errors include `NO_EXIT`, `NOT_GROUP_LEADER`, `PLAYER_NOT_FOUND`, and
`PLAYER_ALREADY_IN_COMBAT`.

#### WHO

```text
WHO
OK players=<count>
```

Returns the number of authenticated players.

#### QUIT

```text
QUIT
OK bye
```

Releases the player's world and group state, notifies the other clients, and
closes the connection.

### Chat

Messages are non-empty, single-line UTF-8 text.

```text
CHAT GLOBAL <message>
CHAT ROOM <message>
CHAT GROUP <message>
CHAT PRIVATE <username> <message>
```

Each valid command returns `OK`. Delivery occurs through the matching event:

| Scope | Event sent to recipients |
| --- | --- |
| Global | `EVT GLOBAL CHAT <username> <message>` |
| Room | `EVT ROOM CHAT <username> <message>` |
| Group | `EVT GROUP CHAT <username> <message>` |
| Private | `EVT PRIVATE CHAT <username> <message>` |

Relevant errors include `INVALID_SCOPE`, `NOT_IN_GROUP`, and `NO_SUCH_USER`.

### Groups

Groups contain at most three players. Invitations expire after five minutes.

#### GROUP CREATE

```text
GROUP CREATE
OK group=<group-id>
```

The creator becomes the leader.

#### GROUP INVITE

```text
GROUP INVITE <username>
OK
```

The target must be connected, available, and in the same room. The target
receives `EVT GROUP INVITE <leader>`.

#### GROUP JOIN

```text
GROUP JOIN <leader-or-member-username>
OK group=<group-id>
```

The argument can name any member of the invited group. Existing members
receive `EVT GROUP JOIN <username>`.

#### GROUP LEAVE

```text
GROUP LEAVE
OK
```

`GROUP QUIT` is accepted as an alias. If the leader leaves, the group is
dissolved. Members receive `EVT GROUP LEAVE <username>`.

Group failures include `NOT_IN_GROUP`, `ALREADY_IN_GROUP`, `NO_SUCH_USER`,
`NOT_INVITED`, `NOT_GROUP_LEADER`, `GROUP_FULL`, `GROUP_NOT_FOUND`, and
`NOT_IN_SAME_ROOM`.

### Resources and NPCs

An item or NPC is normally addressed by its `<numeric-id>.<name>` protocol
identifier. An unambiguous exact name is also accepted in the relevant room or
inventory.

#### TAKE

```text
TAKE <item-identifier>
OK taken=<item-name>
```

Other players in the room receive
`EVT TAKE <username> <item-identifier>`.

#### DROP

```text
DROP <item-identifier>
OK dropped=<item-reference>
```

Other players in the room receive
`EVT DROP <username> <item-identifier>`.

#### INVENTORY

```text
INVENTORY
OK ["0.objet_perdu","2.t_shirt_bde"]
```

The response data is a JSON array of item identifiers.

#### USE

```text
USE <item-identifier>
OK <item-use-result>
```

Activates the selected inventory item's behavior and consumes or updates it as
defined by that item.

#### TALK

```text
TALK <npc-identifier>
OK <dialogue-text>
```

The response advances the player's dialogue with that NPC. The marker
`[end of dialogue]` terminates a dialogue sequence.

#### ATTACK

```text
ATTACK <npc-identifier>
OK <combat-result-json>
```

Example payload:

```json
{
  "attacker_hp": 100,
  "target_hp": 149,
  "damage": 1,
  "status": "combat"
}
```

This is the direct-damage combat command. The code-challenge combat flow uses
`FIGHT` instead.

#### STATUS

```text
STATUS
OK <player-status-json>
```

Example payload:

```json
{
  "hp": 80,
  "max_hp": 100,
  "status": "healthy"
}
```

Status is `healthy` at 80% health or above, `normal` from 30% to below 80%,
and `critical` below 30%.

### Quests

#### QUEST

```text
QUEST <npc-identifier>
OK <quest-json>
```

Example payload:

```json
{
  "name": "Kaizen",
  "description": "Réussissez une fois l'épreuve is_sorted_ascending.c en moins de 55 secondes.",
  "reward": [
    {
      "qty": 3,
      "chance": 100,
      "type": "MERCI"
    }
  ],
  "status": "in progress"
}
```

For a group, only the leader starts the quest. The grouped request applies the
quest to the eligible members and returns one public TAP response.

#### QUESTS

```text
QUESTS
OK [<quest-json>,...]
```

Returns the player's active quests. An empty JSON array is valid.

### Code-challenge fights

#### FIGHT CREATE

```text
FIGHT CREATE <npc-identifier>
OK FIGHT CREATED
```

The target must be a hostile NPC in the player's room and available for a new
fight. A group leader creates a shared fight for the group. Participants then
receive `EVT FIGHT START`.

#### FIGHT ATTACK

```text
FIGHT ATTACK <encoded-code>
OK Processing
```

The submission is a single TAP line. The client replaces spaces and line
breaks using the `sp_sep` and `nl_sep` values received in the start event.
Evaluation is asynchronous; its outcome is delivered through
`EVT FIGHT RESULT` and the fight eventually closes with `EVT FIGHT END`.

## Errors

The server reports failures as `ERR <code> <name>`. Codes are not unique: a
client retains both the number and symbolic name.

| Code | Error name | Meaning |
| --- | --- | --- |
| `201` | `NAME_IN_USE` | The requested username is already authenticated. |
| `204` | `NO_CONTENT` | A command produced no usable data. |
| `301` | `NO_EXIT` | The room has no exit in that direction. |
| `400` | `ALREADY_CONNECTED` | The session is already authenticated. |
| `400` | `NOT_CONNECTED` | The command requires authentication. |
| `400` | `INVALID_USERNAME` | The username violates the accepted grammar. |
| `400` | `ROOM_FULL` | The authenticated-player limit has been reached. |
| `400` | `GROUP_FULL` | The group already contains three players. |
| `400` | `EMPTY_COMMAND` | The submitted command line is empty. |
| `400` | `COMMAND_NOT_FOUND` | The command or subcommand is unknown. |
| `400` | `INVALID_ARGUMENTS` | Arguments are missing, unexpected, or malformed. |
| `400` | `INVALID_SCOPE` | The requested chat scope is invalid. |
| `401` | `NOT_IN_GROUP` | The operation requires group membership. |
| `402` | `ALREADY_IN_GROUP` | The player already belongs to a group. |
| `403` | `NO_SUCH_USER` | The requested user is not connected. |
| `403` | `NOT_INVITED` | No valid invitation exists for the target group. |
| `403` | `NOT_GROUP_LEADER` | The operation is restricted to the group leader. |
| `404` | `ITEM_NOT_FOUND` | The item cannot be resolved in the room. |
| `404` | `ITEM_NOT_IN_INVENTORY` | The item is not in the player's inventory. |
| `404` | `NPC_NOT_FOUND` | The NPC cannot be resolved. |
| `404` | `GROUP_NOT_FOUND` | The referenced group no longer exists. |
| `404` | `NO_SUCH_GROUP` | The requested group cannot be found. |
| `405` | `PLAYER_NOT_FOUND` | The game engine cannot resolve the player. |
| `405` | `NPC_NOT_HOSTILE` | The selected NPC cannot be attacked. |
| `406` | `NO_QUEST_AVAILABLE` | The NPC has no available quest. |
| `407` | `NPC_NOT_IN_ROOM` | The selected NPC is not in the player's room. |
| `407` | `NOT_IN_SAME_ROOM` | Group participants are not in the same room. |
| `408` | `NPC_IN_COMBAT` | The NPC already belongs to another fight. |
| `409` | `ACTION_ALREADY_TAKEN` | The player already acted in this combat round. |
| `410` | `PLAYER_ALREADY_IN_COMBAT` | The player is already participating in a fight. |
| `411` | `PLAYER_NOT_IN_COMBAT` | No active fight exists for the player. |
| `412` | `FILE_NOT_FOUND` | The challenge source file cannot be loaded. |
| `413` | `ROOM_NOT_FOUND` | The player's current room cannot be resolved. |
| `429` | `TOO_MANY_REQUESTS` | More than ten commands arrived within one second. |
| `900` | `CONNECTION_FAILED` | The game server is unavailable. |
| `901` | `SEND_FAILED` | A command could not be sent between servers. |
| `902` | `GAME_SERVER_TIMEOUT` | The game server did not answer within three seconds. |
| `997` | `INVALID_GROUP_COMMAND` | A grouped internal envelope was rejected. |
| `998` | `INVALID_QUESTION` | An internal question envelope was rejected. |
| `999` | `INVALID_COMMAND` | An internal command or JSON envelope is invalid. |
| `999` | `UNKNOWN_ERROR` | An unexpected game-server error was returned. |

Malformed frames, authentication timeout, read timeout, rate-limit abuse, and
socket failure may close the connection. `NetworkError`, `ProtocolError`, and
`InternalError` exposed by the Rust API client are local library errors rather
than TAP `ERR` frames.

## Events

Events are unsolicited server-to-client frames and may arrive between a
command and its response. Clients route them separately while continuing to
wait for the pending `OK` or `ERR`.

### Session and server events

| Frame | Meaning |
| --- | --- |
| `EVT CONNECT <username>` | A player authenticated. |
| `EVT QUIT <username>` | A player disconnected or sent `QUIT`. |
| `EVT STATS players=<count>` | The authenticated-player count changed. |
| `EVT GAME SERVER CONNECTED` | The Go gateway connected to the game engine. |
| `EVT GAME SERVER DISCONNECTED` | The game-engine connection became unavailable. |

### Chat events

| Frame | Delivery |
| --- | --- |
| `EVT GLOBAL CHAT <username> <message>` | Other authenticated clients. |
| `EVT ROOM CHAT <username> <message>` | Other players in the sender's room. |
| `EVT GROUP CHAT <username> <message>` | Other members of the sender's group. |
| `EVT PRIVATE CHAT <username> <message>` | The named private recipient. |

### Groups and movement

| Frame | Meaning |
| --- | --- |
| `EVT GROUP INVITE <leader>` | The recipient was invited to a group. |
| `EVT GROUP JOIN <username>` | A player joined the recipient's group. |
| `EVT GROUP LEAVE <username>` | A player left or the group was dissolved. |
| `EVT GROUPMOVE <leader> <direction>` | A member moved with the group leader. |
| `EVT ROOM PRESENCE ENTER <username>` | A player entered the recipient's room. |
| `EVT ROOM PRESENCE LEAVE <username>` | A player left the recipient's room. |

The room-presence field order follows RFC 42TAP: scope, category, action, then
username.

### World and resources

| Frame | Meaning |
| --- | --- |
| `EVT TAKE <username> <item-identifier>` | A player took an item from the room. |
| `EVT DROP <username> <item-identifier>` | A player dropped an item in the room. |
| `EVT SPAWN type=ITEM id=<item-identifier>` | An item spawned in the room. |
| `EVT SPAWN type=NPC id=<npc-identifier>` | An NPC respawned in the room. |
| `EVT DESPAWN type=ITEM id=<item-identifier>` | A dropped item despawned. |
| `EVT KILL <username> <npc-identifier>` | A player killed an NPC. |
| `EVT DEATH <username> respawn_room_id=<room-name>` | A player died and respawned. |

### Fight events

#### Fight start

```text
EVT FIGHT START <fight-start-json>
```

```json
{
  "code": "int<SP>answer(void)<SP>{<NL>...<NL>}",
  "time": 222,
  "nl_sep": "<NL>",
  "sp_sep": "<SP>",
  "npc_id": "12.ldecavel",
  "npc_hp": 150,
  "npc_max_hp": 150
}
```

#### Fight result

```text
EVT FIGHT RESULT <fight-result-json>
```

```json
{
  "player_name": "ALICE",
  "success": true,
  "damage_dealt": 50
}
```

#### Fight end

```text
EVT FIGHT END
```

Structured event data is encoded as compact, single-line JSON. An unrecognized
event remains an event and is exposed by the Rust client as
`ServerEvent::Unknown`; it is never mistaken for a command response.

## Implementation choices and extensions

RFC 42TAP intentionally leaves several gameplay and representation decisions
to implementations. This project standardizes them as follows:

- The public endpoint uses protocol version 1 and canonical uppercase player
  names.
- Resource and room identifiers use `<numeric-id>.<name>` where an identifier
  is needed on the wire.
- `LOOK`, `STATUS`, quest responses, inventory contents, and fight payloads use
  compact JSON after the leading TAP frame.
- The server supports global, room, group, and private chat scopes.
- Groups contain up to three players; invitations last five minutes.
- `GROUP QUIT` aliases `GROUP LEAVE`.
- `USE` applies inventory-item behavior through the authoritative game engine.
- `QUEST` supports individual and grouped quest assignment.
- Hostile NPC fights use sandboxed C challenges and asynchronous fight events.
- Wraps spawn periodically in the foyer as renewable consumable resources.
