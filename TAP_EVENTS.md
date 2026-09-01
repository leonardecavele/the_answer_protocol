# TAP Events

Events are unsolicited server-to-client frames. They may arrive between a
command request and its response. A client must route `EVT` frames separately
and keep waiting for the corresponding `OK` or `ERR` frame.

## Frame format

TAP uses UTF-8, line-oriented frames. Every event ends with `LF` on the wire.

```abnf
event-line = "EVT" SP event-content LF
event-content = 1*(VCHAR / SP / utf8-nonascii)
utf8-nonascii = <a valid non-ASCII UTF-8 sequence>
```

The Go formatter builds an event in this order:

```text
EVT <event_name> [<emitted_by>] [<data>]
```

`event_name` may contain spaces, and `emitted_by` is optional. The concrete
forms below are the authoritative way to split an event.

## Session and server events

| Frame | Meaning |
| --- | --- |
| `EVT CONNECT <username>` | Another player authenticated. The newly connected player does not receive its own event. |
| `EVT QUIT <username>` | An authenticated player disconnected or sent `QUIT`. |
| `EVT STATS players=<count>` | The number of authenticated players changed. |
| `EVT GAME SERVER CONNECTED` | The Go server connected or reconnected to the Rust game server. |
| `EVT GAME SERVER DISCONNECTED` | The Rust game-server connection is unavailable. |

## Chat events

| Frame | Delivery |
| --- | --- |
| `EVT GLOBAL CHAT <username> <message>` | Every other authenticated TAP client. |
| `EVT ROOM CHAT <username> <message>` | Other players reported by Rust as being in the sender's room. |
| `EVT GROUP CHAT <username> <message>` | Other members of the sender's Go-side group. |
| `EVT PRIVATE CHAT <username> <message>` | Only the named recipient of `CHAT PRIVATE`. |

Messages are non-empty, single-line UTF-8 text. Spaces are preserved after the
sender field.

## Group and movement events

| Frame | Meaning |
| --- | --- |
| `EVT GROUP INVITE <leader>` | The recipient was invited to the leader's group. |
| `EVT GROUP JOIN <username>` | A player joined the recipient's group. |
| `EVT GROUP LEAVE <username>` | A player left the group, disconnected, or caused the group to be dissolved. |
| `EVT GROUPMOVE <leader> <direction>` | A non-leader group member was moved with the leader. |
| `EVT ROOM <username> PRESENCE ENTER` | The player entered the recipient's room. |
| `EVT ROOM <username> PRESENCE LEAVE` | The player left the recipient's room. |

The room presence order is intentionally `ROOM`, username, then `PRESENCE` and
the action. This is the order emitted by the Go formatter and parsed by
`api-client`.

## World and resource events

| Frame | Meaning |
| --- | --- |
| `EVT TAKE <username> <item-identifier>` | Another player took an item from the room. |
| `EVT DROP <username> <item-identifier>` | Another player dropped an item in the room. |
| `EVT SPAWN type=ITEM id=<item-identifier>` | An item spawned in the current room. |
| `EVT SPAWN type=NPC id=<npc-identifier>` | An NPC respawned in the current room. |
| `EVT DESPAWN type=ITEM id=<item-identifier>` | A dropped item despawned from the current room. |
| `EVT KILL <username> <npc-identifier>` | The named player killed an NPC. |
| `EVT DEATH <username> respawn_room_id=<room-name>` | A player died and was reset to the spawn room. |

Item, NPC, and room identifiers are the representations produced by the Rust
server, normally `<numeric-id>.<name>` for resources and rooms.

## Fight events

### Fight start

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

The event is delivered to every player in the newly created combat instance.
`code` uses the separators supplied by `nl_sep` and `sp_sep` so the payload
remains on one TAP line.

### Fight result

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

The result is delivered to the players in the combat instance after a code
submission has been evaluated.

### Fight end

```text
EVT FIGHT END
```

This event indicates that the combat instance has finished and was removed.

## JSON event data

When event data is a structured Go value, the Go server serializes it as compact
JSON. When Rust supplies JSON as a string, the string is forwarded verbatim
after the event name. JSON event payloads therefore remain on a single line and
must not contain a literal `LF`.

An event not matching one of the concrete forms is exposed by `api-client` as
`ServerEvent::Unknown` rather than being treated as a command response.
