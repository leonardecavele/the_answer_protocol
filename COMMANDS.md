## TAP PROTOCOL

Text in uppercase with single quotes is a word string.
Text in uppercase with quotes are commands.
Text in uppercase without quotes are ASCII characters.
Text in lowercase are variables.

Establish a connection
```bash
./client 127.0.0.1 4242
```

Server response
```txt
'OK' SP 'hello' SP 'proto=' protocol_version CRLF
```

### CONNECT command

Client request
```txt
"CONNECT" SP username CRLF
```


Server response
```txt
'Password:' SP
```


Client request
```txt
password CRLF
```


Server response
```txt
'Successfully' SP 'connected' SP 'as' username CRLF
```
or
```txt
'Invalid' SP 'user' SP 'or' SP 'password' CRLF
```
