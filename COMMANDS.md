## TAP PROTOCOL

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
```

### CONNECT command

Client
```abnf
connect-request = "CONNECT" SP username LF
```


Server
```abnf
connect-response = password-prompt / name-in-use-error

password-prompt = "OK" SP "Password:" LF
name-in-use-error = "ERR" SP "201" SP "NAME_IN_USE" LF
```


Client
```abnf
password-request = "AUTH" SP password LF
```


Server
```abnf
password-response = connection-success / invalid-user-or-password

connection-success = "OK" SP "Successfully" SP "connected" SP "as" SP username LF
invalid-user-or-password = "ERR" SP "202" SP "INVALID_USER_OR_PASSWORD" LF
```

### QUIT command

Client
```abnf
quit-request = "QUIT" LF
```
or due to server/client connection issue or program aborption


Server
```abnf
quit-response = "OK" SP "Successfully" SP "disconnected" LF
```
