# TAP EVENTS

## Format

### Event Format

```abnf
; event format
message = event-line
event-line = "EVT" SP event-type SP event-data LF
event-type = 1*ALPHA
event-data = 1*(VCHAR / SP)
```

## Arguments

```abnf
; event arguments format
username = ALPHA *(ALPHA / DIGIT / "_" / "-")
leader-name = username
chat-message = VCHAR *(SP / VCHAR)
player-server-count = 1*DIGIT
```

## Room Events

```abnf
; event-line
room-event = room-presence-enter-event / room-presence-leave-event / room-chat-event

room-presence-enter-event = "EVT" SP "ROOM" SP "PRESENCE" SP "ENTER" SP username LF
room-presence-leave-event = "EVT" SP "ROOM" SP "PRESENCE" SP "LEAVE" SP username LF
room-chat-event = "EVT" SP "ROOM" SP "CHAT" SP username SP chat-message LF
```

### Room Event Meaning

| Event | Meaning |
|---|---|
| `EVT ROOM PRESENCE ENTER <username>` | Player entered the current room |
| `EVT ROOM PRESENCE LEAVE <username>` | Player left the current room |
| `EVT ROOM CHAT <username> <message>` | Room-scoped chat message |

## Global Events

```abnf
; event-line
global-event = global-chat-event

global-chat-event = "EVT" SP "GLOBAL" SP "CHAT" SP username SP chat-message LF
```

### Global Event Meaning

| Event | Meaning |
|---|---|
| `EVT GLOBAL CHAT <username> <message>` | Server-wide chat message |

## Group Events

```abnf
; event-line
group-event = group-invite-event / group-join-event / group-leave-event / group-chat-event

group-invite-event = "EVT" SP "GROUP" SP "INVITE" SP leader-name LF
group-join-event = "EVT" SP "GROUP" SP "JOIN" SP username LF
group-leave-event = "EVT" SP "GROUP" SP "LEAVE" SP username LF
group-chat-event = "EVT" SP "GROUP" SP "CHAT" SP username SP chat-message LF
```

### Group Event Meaning

| Event | Meaning |
|---|---|
| `EVT GROUP INVITE <leader>` | Group invitation received |
| `EVT GROUP JOIN <username>` | Player joined the group |
| `EVT GROUP LEAVE <username>` | Player left the group |
| `EVT GROUP CHAT <username> <message>` | Group-scoped chat message |

## Stats Events

```abnf
; event-line
stats-event = "EVT" SP "STATS" SP "players=" player-server-count LF
```

### Stats Event Meaning

| Event | Meaning |
|---|---|
| `EVT STATS players=<count>` | Updated player count |
