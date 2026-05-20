## TAP PROTOCOL

This document uses an ABNF-like syntax.

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
server-greeting = "OK" SP "hello" SP "proto=" protocol-version CRLF
```

### CONNECT command

Client
```abnf
connect-request = "CONNECT" SP username CRLF
```


Server
```abnf
connect-response = password-prompt / name-in-use-error

password-prompt = "Password:" SP
name-in-use-error = "ERR" SP "201" SP "NAME_IN_USE" CRLF
```


Client
```abnf
password-request = password CRLF
```


Server
```abnf
password-response = connection-success / invalid-user-or-password

connection-success = "Successfully" SP "connected" SP "as" SP username CRLF
invalid-user-or-password = "Invalid" SP "user" SP "or" SP "password" CRLF
```

### QUIT command

Client
```abnf
quit-request = "QUIT" CRLF
```
or due to server/client connection issue or program aborption


Server
```abnf
quit-response = "Successfully" SP "disconnected" CRLF
```
