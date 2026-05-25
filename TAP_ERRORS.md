# TAP ERRORS

## Format

### Response Format

```abnf
; response format
message = response-line
response-line = ("OK" / error-response) [SP response-data] LF
response-data = 1*(VCHAR / SP)
```

### Error Format

```abnf
; error format
error-response = "ERR" SP error-code SP error-message
error-code = 3DIGIT
error-message = 1*(ALPHA / DIGIT / "_")
```

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

## Standard Error Meaning

| Error | Meaning |
|---|---|
| `ERR 201 NAME_IN_USE` | Requested username already taken |
| `ERR 301 NO_EXIT` | Invalid movement direction |
| `ERR 401 NOT_IN_GROUP` | Group operation requires group membership |
| `ERR 402 ALREADY_IN_GROUP` | Player already belongs to a group |
| `ERR 404 ITEM_NOT_FOUND` | Requested item not available in the room |
| `ERR 404 ITEM_NOT_IN_INVENTORY` | Requested item not in player inventory |
| `ERR 404 NPC_NOT_FOUND` | Requested NPC not present |
| `ERR 405 NPC_NOT_HOSTILE` | NPC cannot be attacked |
| `ERR 406 NO_QUEST_AVAILABLE` | NPC has no quest, or the quest is already completed |
| `ERR 900 CONNECTION_FAILED` | Connection establishment failed |
| `ERR 901 SEND_FAILED` | Message transmission failed |

## Additional Error Meaning

| Error | Meaning |
|---|---|
| `ERR 400 ALREADY_CONNECTED` | Client tried to connect while already authenticated |
| `ERR 400 INVALID_SCOPE` | Chat scope is not `GLOBAL`, `ROOM`, or `GROUP` |
| `ERR 403 NO_SUCH_USER` | Requested user does not exist or is not connected |
| `ERR 403 NOT_INVITED` | Player tried to join a group without a valid invitation |
| `ERR 404 GROUP_NOT_FOUND` | Target group does not exist anymore |
| `ERR 404 NO_SUCH_GROUP` | Requested group does not exist |
