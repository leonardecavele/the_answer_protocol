# TAP PROTOCOL

## Format

### Command Format

```abnf
; command format
command-line = command-name [SP arguments] LF
command-name = 1*ALPHA
arguments = 1*(VCHAR / SP)
```

### Response Format

```abnf
; response format
response-line = ("OK" / error-response) [SP response-data] LF
response-data = 1*(VCHAR / SP)
```

```abnf
; error format
error-response = "ERR" SP error-code SP error-message
error-code = 3DIGIT
error-message = 1*(ALPHA / DIGIT / "_")
```

### Event Format

```abnf
; event format
event-line = "EVT" SP event-type SP event-data LF
event-type = 1*ALPHA
event-data = 1*(VCHAR / SP)
```

### Message Format

```abnf
; protocol format
message = command-line / response-line / event-line
```

## Arguments

```abnf
; arguments format
chat-scope = "GLOBAL" / "ROOM" / "GROUP"
chat-message = VCHAR *(SP / VCHAR)
protocol-version = 1*DIGIT
room-id = 1*(ALPHA / DIGIT / "_" / "-" / ".")
player-server-count = 1*DIGIT
current-room-state-json = <valid JSON text encoded on one line, without LF>
group-id = 1*(ALPHA / DIGIT / "_" / "-" / ".")
username = ALPHA *(ALPHA / DIGIT / "_" / "-")
leader-name = username
item-identifier = 1*(VCHAR / SP)
direction = 1*ALPHA
npc-name = 1*(VCHAR / SP)
dialogue = 1*(VCHAR / SP)
combat-result-json = <valid JSON text encoded on one line, without LF>
player-status-json = <valid JSON text encoded on one line, without LF>
quest-data-json = <valid JSON text encoded on one line, without LF>
quest-list-json = <valid JSON array encoded on one line, without LF>
inventory-json = <valid JSON array encoded on one line, without LF>
```

## Establish Connection

Client
```bash
./client 127.0.0.1 4242
```


Server
```abnf
; response-line
server-greeting = "OK" SP "hello" SP "proto=" protocol-version LF
```

# Error Handling

## RFC Errors

```abnf
; error response-lines
err-name-in-use = "ERR" SP "201" SP "NAME_IN_USE" LF
err-no-exit = "ERR" SP "301" SP "NO_EXIT" LF
err-not-in-group = "ERR" SP "401" SP "NOT_IN_GROUP" LF
err-already-in-group = "ERR" SP "402" SP "ALREADY_IN_GROUP" LF
err-item-not-found = "ERR" SP "404" SP "ITEM_NOT_FOUND" LF
err-item-not-in-inventory = "ERR" SP "404" SP "ITEM_NOT_IN_INVENTORY" LF
err-npc-not-found = "ERR" SP "404" SP "NPC_NOT_FOUND" LF
err-npc-not-hostile = "ERR" SP "405" SP "NPC_NOT_HOSTILE" LF
err-no-quest-available = "ERR" SP "406" SP "NO_QUEST_AVAILABLE" LF
err-connection-failed = "ERR" SP "900" SP "CONNECTION_FAILED" LF
err-send-failed = "ERR" SP "901" SP "SEND_FAILED" LF
```

## Implementation Errors

```abnf
; custom error response-lines
err-already-connected = "ERR" SP "400" SP "ALREADY_CONNECTED" LF
err-invalid-scope = "ERR" SP "400" SP "INVALID_SCOPE" LF
err-no-such-user = "ERR" SP "403" SP "NO_SUCH_USER" LF
err-not-invited = "ERR" SP "403" SP "NOT_INVITED" LF
err-group-not-found = "ERR" SP "404" SP "GROUP_NOT_FOUND" LF
err-no-such-group = "ERR" SP "404" SP "NO_SUCH_GROUP" LF
```

# Commands

## Core Commands

### CONNECT command

Client
```abnf
; command-line
connect-request = "CONNECT" SP username LF
```

Server
```abnf
; response-line
connect-response = connect-success / err-name-in-use / err-already-connected

connect-success = "OK" SP "connected" LF
```

### LOOK command

Client
```abnf
; command-line
look-request = "LOOK" LF
```


Server
```abnf
; response-line
look-response = "OK" SP current-room-state-json LF
```

### MOVE command

Client
```abnf
; command-line
move-request = "MOVE" SP direction LF
```


Server
```abnf
; response-line
move-response = successful-move / err-no-exit

successful-move = "OK" SP "room=" room-id LF
```

### QUIT command

Client
```abnf
; command-line
quit-request = "QUIT" LF
```


Server
```abnf
; response-line
quit-response = "OK" SP "bye" LF
```

## Communication Commands

### CHAT command

Client
```abnf
; command-line
chat-request = "CHAT" SP chat-scope SP chat-message LF
```


Server
```abnf
; response-line
chat-response = chat-success / err-not-in-group / err-invalid-scope / err-no-such-group

chat-success = "OK" LF
```

### WHO command

Client
```abnf
; command-line
who-request = "WHO" LF
```


Server
```abnf
; response-line
who-response = "OK" SP "players=" player-server-count LF
```

## Group Management Commands

### GROUP CREATE command

Client
```abnf
; command-line
group-create-request = "GROUP" SP "CREATE" LF
```


Server
```abnf
; response-line
group-create-response = group-create-success / err-already-in-group

group-create-success = "OK" SP "group=" group-id LF
```

### GROUP INVITE command

Client
```abnf
; command-line
group-invite-request = "GROUP" SP "INVITE" SP username LF
```


Server
```abnf
; response-line
group-invite-response = group-invite-success / err-not-in-group / err-no-such-user / err-already-in-group / err-group-not-found

group-invite-success = "OK" LF
```

### GROUP JOIN command

Client
```abnf
; command-line
group-join-request = "GROUP" SP "JOIN" SP leader-name LF
```


Server
```abnf
; response-line
group-join-response = group-join-success / err-no-such-user / err-already-in-group / err-not-invited / err-group-not-found

group-join-success = "OK" SP "group=" group-id LF
```

### GROUP LEAVE command

Client
```abnf
; command-line
group-leave-request = "GROUP" SP "LEAVE" LF
```


Server
```abnf
; response-line
group-leave-response = group-leave-success / err-not-in-group / err-group-not-found

group-leave-success = "OK" LF
```

## Resource Interaction Commands

### TAKE command

Client
```abnf
; command-line
take-request = "TAKE" SP item-identifier LF
```


Server
```abnf
; response-line
take-response = take-success / err-item-not-found

take-success = "OK" SP "taken=" item-identifier LF
```

### DROP command

Client
```abnf
; command-line
drop-request = "DROP" SP item-identifier LF
```


Server
```abnf
; response-line
drop-response = drop-success / err-item-not-in-inventory

drop-success = "OK" SP "dropped=" item-identifier LF
```

### INVENTORY command

Client
```abnf
; command-line
inventory-request = "INVENTORY" LF
```


Server
```abnf
; response-line
inventory-response = "OK" SP inventory-json LF
```

### TALK command

Client
```abnf
; command-line
talk-request = "TALK" SP npc-name LF
```


Server
```abnf
; response-line
talk-response = talk-success / err-npc-not-found

talk-success = "OK" SP dialogue LF
```

### ATTACK command

Client
```abnf
; command-line
attack-request = "ATTACK" SP npc-name LF
```


Server
```abnf
; response-line
attack-response = attack-success / err-npc-not-found / err-npc-not-hostile

attack-success = "OK" SP combat-result-json LF
```

### STATUS command

Client
```abnf
; command-line
status-request = "STATUS" LF
```


Server
```abnf
; response-line
status-response = "OK" SP player-status-json LF
```

### QUEST command

Client
```abnf
; command-line
quest-request = "QUEST" SP npc-name LF
```


Server
```abnf
; response-line
quest-response = quest-success / err-npc-not-found / err-no-quest-available

quest-success = "OK" SP quest-data-json LF
```

### QUESTS command

Client
```abnf
; command-line
quests-request = "QUESTS" LF
```


Server
```abnf
; response-line
quests-response = "OK" SP quest-list-json LF
```

# Events

## Room Events

```abnf
; event-line
room-event = room-presence-enter-event / room-presence-leave-event / room-chat-event

room-presence-enter-event = "EVT" SP "ROOM" SP "PRESENCE" SP "ENTER" SP username LF
room-presence-leave-event = "EVT" SP "ROOM" SP "PRESENCE" SP "LEAVE" SP username LF
room-chat-event = "EVT" SP "ROOM" SP "CHAT" SP username SP chat-message LF
```

## Global Events

```abnf
; event-line
global-event = global-chat-event

global-chat-event = "EVT" SP "GLOBAL" SP "CHAT" SP username SP chat-message LF
```

## Group Events

```abnf
; event-line
group-event = group-invite-event / group-join-event / group-leave-event / group-chat-event

group-invite-event = "EVT" SP "GROUP" SP "INVITE" SP leader-name LF
group-join-event = "EVT" SP "GROUP" SP "JOIN" SP username LF
group-leave-event = "EVT" SP "GROUP" SP "LEAVE" SP username LF
group-chat-event = "EVT" SP "GROUP" SP "CHAT" SP username SP chat-message LF
```

## Stats Events

```abnf
; event-line
stats-event = "EVT" SP "STATS" SP "players=" player-server-count LF
```
