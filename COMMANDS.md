## TAP PROTOCOL

/* move to another md file */
ROOM TEMPLATE
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

Variables
```abnf
protocol-version = 1*DIGIT
username = 1*VCHAR
password = 1*VCHAR
```

### establish connection

Client
```bash
./client 127.0.0.1 4242
```


Server
```abnf
server-greeting = "OK" SP "hello" SP "proto=" protocol-version LF

protocol-version = 1*DIGIT
```

### CONNECT command

Client
```abnf
connect-request = "CONNECT" SP username LF

username = ALPHA *(ALPHA / DIGIT / "_" / "-")
```


Server
```abnf
connect-response = "OK" SP "connected" LF
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
move-request = "MOVE" lf
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

### CHAT command

Client
```abnf
chat-request = "CHAT" SP chat-scope SP chat_message LF

chat-scope = "GLOBAL" / "ROOM" / "GROUP"
chat-message = VCHAR *(SP / VCHAR)
```


Server
```abnf
chat-response = "OK" LF
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

### GROUP CREATE command

Client
```abnf
group-create-request = "GROUP" SP "CREATE" LF
```


Server
```abnf
group-create-response = "OK" SP "group=" group-id LF

group-id = 1*DIGIT
```

### GROUP INVITE command
