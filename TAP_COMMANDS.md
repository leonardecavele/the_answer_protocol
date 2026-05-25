# TAP COMMANDS

## Format

### Command Format

```abnf
; command format
message = command-line
command-line = command-name [SP arguments] LF
command-name = 1*ALPHA
arguments = 1*(VCHAR / SP)
```

### Success Response Format

```abnf
; success response format
response-line = "OK" [SP response-data] LF
response-data = 1*(VCHAR / SP)
```

## Arguments

```abnf
; common values
line-text = VCHAR *(SP / VCHAR)
json-text = <valid JSON text encoded on one line, without LF>
json-array = <valid JSON array encoded on one line, without LF>
number = 1*DIGIT
```

```abnf
; protocol values
protocol-version = number

; player values
username = ALPHA *(ALPHA / DIGIT / "_" / "-")
leader-name = username
player-server-count = number

; world values
room-id = 1*(ALPHA / DIGIT / "_" / "-" / ".")
direction = 1*ALPHA

; group values
group-id = 1*(ALPHA / DIGIT / "_" / "-" / ".")

; resource values
item-identifier = line-text
npc-name = line-text
dialogue = line-text

; chat values
chat-scope = "GLOBAL" / "ROOM" / "GROUP"
chat-message = line-text

; json payloads
current-room-state-json = json-text
combat-result-json = json-text
player-status-json = json-text
quest-data-json = json-text
quest-list-json = json-array
inventory-json = json-array
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
quit-request = quit-request-command / <server/client connection issue or program aborption>

quit-request-command = "QUIT" LF
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
