# TAP ERRORS

## Format

### Response Format

<pre><code class="language-abnf">; response format
<a id="message" href="TAP_ERRORS.md#message">message</a> = <a href="TAP_ERRORS.md#response-line">response-line</a>
<a id="response-line" href="TAP_ERRORS.md#response-line">response-line</a> = ("OK" / <a href="TAP_ERRORS.md#error-response">error-response</a>) [SP <a href="TAP_ERRORS.md#response-data">response-data</a>] LF
<a id="response-data" href="TAP_ERRORS.md#response-data">response-data</a> = 1*(VCHAR / SP)
</code></pre>

### Error Format

<pre><code class="language-abnf">; error format
<a id="error-response" href="TAP_ERRORS.md#error-response">error-response</a> = "ERR" SP <a href="TAP_ERRORS.md#error-code">error-code</a> SP <a href="TAP_ERRORS.md#error-message">error-message</a>
<a id="error-code" href="TAP_ERRORS.md#error-code">error-code</a> = 3DIGIT
<a id="error-message" href="TAP_ERRORS.md#error-message">error-message</a> = 1*(ALPHA / DIGIT / "_")
</code></pre>

## Errors

<pre><code class="language-abnf">; error response-lines
<a id="err-name-in-use" href="TAP_ERRORS.md#err-name-in-use">err-name-in-use</a> = "ERR" SP "201" SP "NAME_IN_USE" LF
<a id="err-no-exit" href="TAP_ERRORS.md#err-no-exit">err-no-exit</a> = "ERR" SP "301" SP "NO_EXIT" LF
<a id="err-not-in-group" href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> = "ERR" SP "401" SP "NOT_IN_GROUP" LF
<a id="err-already-in-group" href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a> = "ERR" SP "402" SP "ALREADY_IN_GROUP" LF
<a id="err-item-not-found" href="TAP_ERRORS.md#err-item-not-found">err-item-not-found</a> = "ERR" SP "404" SP "ITEM_NOT_FOUND" LF
<a id="err-item-not-in-inventory" href="TAP_ERRORS.md#err-item-not-in-inventory">err-item-not-in-inventory</a> = "ERR" SP "404" SP "ITEM_NOT_IN_INVENTORY" LF
<a id="err-npc-not-found" href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a> = "ERR" SP "404" SP "NPC_NOT_FOUND" LF
<a id="err-npc-not-hostile" href="TAP_ERRORS.md#err-npc-not-hostile">err-npc-not-hostile</a> = "ERR" SP "405" SP "NPC_NOT_HOSTILE" LF
<a id="err-no-quest-available" href="TAP_ERRORS.md#err-no-quest-available">err-no-quest-available</a> = "ERR" SP "406" SP "NO_QUEST_AVAILABLE" LF
<a id="err-connection-failed" href="TAP_ERRORS.md#err-connection-failed">err-connection-failed</a> = "ERR" SP "900" SP "CONNECTION_FAILED" LF
<a id="err-send-failed" href="TAP_ERRORS.md#err-send-failed">err-send-failed</a> = "ERR" SP "901" SP "SEND_FAILED" LF
</code></pre>

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

## Additional Errors

<pre><code class="language-abnf">; custom error response-lines
<a id="err-already-connected" href="TAP_ERRORS.md#err-already-connected">err-already-connected</a> = "ERR" SP "400" SP "ALREADY_CONNECTED" LF
<a id="err-not-connected" href="TAP_ERRORS.md#err-not-connected">err-not-connected</a> = "ERR" SP "400" SP "NOT_CONNECTED" LF
<a id="err-invalid-username" href="TAP_ERRORS.md#err-invalid-username">err-invalid-username</a> = "ERR" SP "400" SP "INVALID_USERNAME" LF
<a id="err-room-full" href="TAP_ERRORS.md#err-room-full">err-room-full</a> = "ERR" SP "400" SP "ROOM_FULL" LF
<a id="err-group-full" href="TAP_ERRORS.md#err-group-full">err-group-full</a> = "ERR" SP "400" SP "GROUP_FULL" LF
<a id="err-empty-command" href="TAP_ERRORS.md#err-empty-command">err-empty-command</a> = "ERR" SP "400" SP "EMPTY_COMMAND" LF
<a id="err-command-not-found" href="TAP_ERRORS.md#err-command-not-found">err-command-not-found</a> = "ERR" SP "400" SP "COMMAND_NOT_FOUND" LF
<a id="err-invalid-arguments" href="TAP_ERRORS.md#err-invalid-arguments">err-invalid-arguments</a> = "ERR" SP "400" SP "INVALID_ARGUMENTS" LF
<a id="err-invalid-scope" href="TAP_ERRORS.md#err-invalid-scope">err-invalid-scope</a> = "ERR" SP "400" SP "INVALID_SCOPE" LF
<a id="err-no-such-user" href="TAP_ERRORS.md#err-no-such-user">err-no-such-user</a> = "ERR" SP "403" SP "NO_SUCH_USER" LF
<a id="err-not-invited" href="TAP_ERRORS.md#err-not-invited">err-not-invited</a> = "ERR" SP "403" SP "NOT_INVITED" LF
<a id="err-group-not-found" href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a> = "ERR" SP "404" SP "GROUP_NOT_FOUND" LF
<a id="err-no-such-group" href="TAP_ERRORS.md#err-no-such-group">err-no-such-group</a> = "ERR" SP "404" SP "NO_SUCH_GROUP" LF
<a id="err-not-group-leader" href="TAP_ERRORS.md#err-not-group-leader">err-not-group-leader</a> = "ERR" SP "403" SP "NOT_GROUP_LEADER" LF
<a id="err-invalid-question" href="TAP_ERRORS.md#err-invalid-question">err-invalid-question</a> = "ERR" SP "998" SP "INVALID_QUESTION" LF
<a id="err-invalid-command" href="TAP_ERRORS.md#err-invalid-command">err-invalid-command</a> = "ERR" SP "999" SP "INVALID_COMMAND" LF
<a id="err-unknown-error" href="TAP_ERRORS.md#err-unknown-error">err-unknown-error</a> = "ERR" SP "999" SP "UNKNOWN_ERROR" LF
</code></pre>

| Error | Meaning |
|---|---|
| `ERR 400 ALREADY_CONNECTED` | Client tried to connect while already authenticated |
| `ERR 400 NOT_CONNECTED` | Command requires an authenticated client |
| `ERR 400 INVALID_USERNAME` | Username does not match the protocol format |
| `ERR 400 ROOM_FULL` | Target room cannot accept another player |
| `ERR 400 GROUP_FULL` | Target group cannot accept another player |
| `ERR 400 EMPTY_COMMAND` | Client sent an empty command line |
| `ERR 400 COMMAND_NOT_FOUND` | Command is not supported by this server |
| `ERR 400 INVALID_ARGUMENTS` | Command arguments do not match the expected format |
| `ERR 400 INVALID_SCOPE` | Chat scope is not `GLOBAL`, `ROOM`, or `GROUP` |
| `ERR 403 NO_SUCH_USER` | Requested user does not exist or is not connected |
| `ERR 403 NOT_INVITED` | Player tried to join a group without a valid invitation |
| `ERR 404 GROUP_NOT_FOUND` | Target group does not exist anymore |
| `ERR 404 NO_SUCH_GROUP` | Requested group does not exist |
| `ERR 403 NOT_GROUP_LEADER` | Player must be the group leader to perform this action |
| `ERR 998 INVALID_QUESTION` | Game server received an invalid question payload |
| `ERR 999 INVALID_COMMAND` | The player sent an invalid command format |
| `ERR 999 UNKNOWN_ERROR` | Server received an unknown or invalid game-server error code |
