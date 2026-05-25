# TAP PROTOCOL

ABNF Syntax
```txt
message = command-line / response-line / event-line
command-line = command-name [SP arguments] LF
response-line = ("OK" / error-response) [SP response-data] LF
event-line = "EVT" SP event-type SP event-data LF
command-name = 1*ALPHA
arguments = 1*(VCHAR / SP)
response-data = 1*(VCHAR / SP)
error-response = "ERR" SP error-code SP error-message
error-code = 3DIGIT
error-message = 1*(ALPHA / DIGIT / "_")
event-type = 1*ALPHA
event-data = 1*(VCHAR / SP)
```

## Establish Connection

Client
```bash
./client 127.0.0.1 4242
```


Server
```abnf
server-greeting = "OK" SP "hello" SP "proto=" protocol-version LF

protocol-version = 1*DIGIT
```

## Core Commands

### CONNECT command

Client
```abnf
connect-request = "CONNECT" SP username LF

username = ALPHA *(ALPHA / DIGIT / "_" / "-")
```

Server
```abnf
connect-response = connect-success / err-already-connected

connect-success = "OK" SP "connected" LF
```

### LOOK command

Client
```abnf
look-request = "LOOK" LF
```


Server
```abnf
look-response = "OK" SP current-room-state-json LF

current-room-state-json = <valid JSON text encoded on one line, without LF>
```

### MOVE command

Client
```abnf
move-request = "MOVE" LF
```


Server
```abnf
move-response = succesful_move / failed_move

succesful_move = "OK" SP "room=" room_id LF
failed_move = "ERR" SP "301" NO_EXIT LF
```

### QUIT command

Client
```abnf
quit-request = "QUIT" LF
```
or due to server/client connection issue or program aborption


Server
```abnf
quit-response = "OK" SP "bye" LF
```

## Communication Commands

### CHAT command

Client
```abnf
chat-request = "CHAT" SP chat-scope SP chat_message LF

chat-scope = "GLOBAL" / "ROOM" / "GROUP"
chat-message = VCHAR *(SP / VCHAR)
```


Server
```abnf
chat-response = chat-success / err-no-such-group / err-invalid-scope

chat-success = "OK" LF
```

### WHO command

Client
```abnf
who-request = "WHO" LF
```


Server
```abnf
who-response = "OK" SP "players=" player-server-count LF

player-server-count = 1*DIGIT
```

## Group Management Commands

### GROUP CREATE command

Client
```abnf
group-create-request = "GROUP" SP "CREATE" LF
```


Server
```abnf
group-create-response = group-create-success / err-already-in-group 

group-create-success = "OK" SP "group-id=" group-id LF
group-id = 1*DIGIT
```

### GROUP INVITE command

Client
```abnf
group-invite-request = "GROUP" SP "INVITE" SP username LF

username = ALPHA *(ALPHA / DIGIT / "_" / "-")
```


Server
```abnf
group-invite-response = group-invite-success / err-no-such-user / err-already-in-group / err-group-not-found

groupe-invite-success = "OK" LF
```

### GROUP JOIN command

Client
```abnf
group-join-request = "GOUP" SP "JOIN" SP leader_name LF

leader_name = ALPHA *(ALPHA / DIGIT / "_" / "-")
```


Server
```abnf
group-join-response = group-join-success / err-no-such-user / err-group-not-found

group-join-success = "OK" SP "group=" group-id LF
group-id = 1*DIGIT
```

### GROUP LEAVE command

Client
```abnf
group-leave-request = "GROUP" SP "LEAVE" LF
```


Server
```abnf
group-leave-reponse = group-leave-success / err-group-not-found

group-leave-success = "OK" LF
```

###

Client
```abnf
```


Server
```abnf
```

###

Client
```abnf
```


Server
```abnf
```

###
