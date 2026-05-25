# ROOM MODEL

## Json Room Format

```json
{
  "room": {
    "id": "room.identifier",
    "name": "Room Display Name",
    "description": "Room description text",
    "exits": {
      "north": "room.north_id",
      "south": "room.south_id"
    }
  },
  "players": ["username1", "username2"],
  "items": ["item.id1", "item.id2"],
  "npcs": ["npc.id1", "npc.id2"]
}
```

## User State (Server Side)

### As Graph

```mermaid
stateDiagram-v2
    [*] --> EMPTY

    EMPTY --> CONNECTED: TCP connection accepted
    CONNECTED --> AUTHENTICATED: CONNECT <username>
    AUTHENTICATED --> EMPTY: QUIT / socket closed / error

    CONNECTED --> EMPTY: authentication failed / socket closed
```

### As C Code

```c
typedef enum e_state
{
    EMPTY,
    CONNECTED,
    AUTHENTICATED
}   t_state;
```

### Text Representation

```txt
[ROOM]
00 CONNECTED
01 AUTHENTIFICATED as BOB
02 EMPTY
03 EMPTY
04 EMPTY
05 EMPTY
06 EMPTY
07 EMPTY
08 EMPTY
09 EMPTY
10 EMPTY
11 EMPTY
12 EMPTY
13 EMPTY
14 EMPTY
15 EMPTY
16 EMPTY
17 EMPTY
18 EMPTY
19 EMPTY
```
