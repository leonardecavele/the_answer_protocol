<style>
h6 { display: none; }
</style>

# TAP COMMANDS

## Format

### Command Format

###### message
###### command-line
###### command-name
###### arguments

<pre><code class="language-abnf">; command format
<a href="TAP_COMMANDS.md#message">message</a> = <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#command-line">command-line</a> = <a href="TAP_COMMANDS.md#command-name">command-name</a> [SP <a href="TAP_COMMANDS.md#arguments">arguments</a>] LF
<a href="TAP_COMMANDS.md#command-name">command-name</a> = 1*ALPHA
<a href="TAP_COMMANDS.md#arguments">arguments</a> = 1*(VCHAR / SP)
</code></pre>

### Success Response Format

###### response-line
###### response-data

<pre><code class="language-abnf">; success response format
<a href="TAP_COMMANDS.md#response-line">response-line</a> = "OK" [SP <a href="TAP_COMMANDS.md#response-data">response-data</a>] LF
<a href="TAP_COMMANDS.md#response-data">response-data</a> = 1*(VCHAR / SP)
</code></pre>

## Arguments

### Common Values

###### line-text
###### json-text
###### json-array
###### number

<pre><code class="language-abnf">; common values
<a href="TAP_COMMANDS.md#line-text">line-text</a> = VCHAR *(SP / VCHAR)
<a href="TAP_COMMANDS.md#json-text">json-text</a> = &lt;valid JSON text encoded on one line, without LF&gt;
<a href="TAP_COMMANDS.md#json-array">json-array</a> = &lt;valid JSON array encoded on one line, without LF&gt;
<a href="TAP_COMMANDS.md#number">number</a> = 1*DIGIT
</code></pre>

### Arguments

###### protocol-version

<pre><code class="language-abnf">; protocol values
<a href="TAP_COMMANDS.md#protocol-version">protocol-version</a> = <a href="TAP_COMMANDS.md#number">number</a>
</code></pre>

###### username
###### leader-name
###### player-server-count

<pre><code class="language-abnf">; player values
<a href="TAP_COMMANDS.md#username">username</a> = ALPHA *(ALPHA / DIGIT / "_" / "-")
<a href="TAP_COMMANDS.md#leader-name">leader-name</a> = <a href="TAP_COMMANDS.md#username">username</a>
<a href="TAP_COMMANDS.md#player-server-count">player-server-count</a> = <a href="TAP_COMMANDS.md#number">number</a>
</code></pre>

###### room-id
###### direction

<pre><code class="language-abnf">; world values
<a href="TAP_COMMANDS.md#room-id">room-id</a> = 1*(ALPHA / DIGIT / "_" / "-" / ".")
<a href="TAP_COMMANDS.md#direction">direction</a> = 1*ALPHA
</code></pre>

###### group-id

<pre><code class="language-abnf">; group values
<a href="TAP_COMMANDS.md#group-id">group-id</a> = 1*(ALPHA / DIGIT / "_" / "-" / ".")
</code></pre>

###### item-identifier
###### npc-name
###### dialogue

<pre><code class="language-abnf">; resource values
<a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
<a href="TAP_COMMANDS.md#npc-name">npc-name</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
<a href="TAP_COMMANDS.md#dialogue">dialogue</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
</code></pre>

###### chat-scope
###### chat-message

<pre><code class="language-abnf">; chat values
<a href="TAP_COMMANDS.md#chat-scope">chat-scope</a> = "GLOBAL" / "ROOM" / "GROUP"
<a href="TAP_COMMANDS.md#chat-message">chat-message</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
</code></pre>

###### current-room-state-json
###### combat-result-json
###### player-status-json
###### quest-data-json
###### quest-list-json
###### inventory-json

<pre><code class="language-abnf">; json payloads
<a href="TAP_COMMANDS.md#current-room-state-json">current-room-state-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a href="TAP_COMMANDS.md#combat-result-json">combat-result-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a href="TAP_COMMANDS.md#player-status-json">player-status-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a href="TAP_COMMANDS.md#quest-data-json">quest-data-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a href="TAP_COMMANDS.md#quest-list-json">quest-list-json</a> = <a href="TAP_COMMANDS.md#json-array">json-array</a>
<a href="TAP_COMMANDS.md#inventory-json">inventory-json</a> = <a href="TAP_COMMANDS.md#json-array">json-array</a>
</code></pre>

## Establish Connection

Client
```bash
./client 127.0.0.1 4242
```


Server
###### server-greeting

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#server-greeting">server-greeting</a> = "OK" SP "hello" SP "proto=" <a href="TAP_COMMANDS.md#protocol-version">protocol-version</a> LF
</code></pre>

## Core Commands

### CONNECT command

Client
###### connect-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#connect-request">connect-request</a> = "CONNECT" SP <a href="TAP_COMMANDS.md#username">username</a> LF
</code></pre>

Server
###### connect-response
###### connect-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#connect-response">connect-response</a> = <a href="TAP_COMMANDS.md#connect-success">connect-success</a> / <a href="TAP_ERRORS.md#err-name-in-use">err-name-in-use</a> / <a href="TAP_ERRORS.md#err-already-connected">err-already-connected</a>

<a href="TAP_COMMANDS.md#connect-success">connect-success</a> = "OK" SP "connected" LF
</code></pre>

### LOOK command

Client
###### look-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#look-request">look-request</a> = "LOOK" LF
</code></pre>


Server
###### look-response

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#look-response">look-response</a> = "OK" SP <a href="TAP_COMMANDS.md#current-room-state-json">current-room-state-json</a> LF
</code></pre>

### MOVE command

Client
###### move-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#move-request">move-request</a> = "MOVE" SP <a href="TAP_COMMANDS.md#direction">direction</a> LF
</code></pre>


Server
###### move-response
###### successful-move

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#move-response">move-response</a> = <a href="TAP_COMMANDS.md#successful-move">successful-move</a> / <a href="TAP_ERRORS.md#err-no-exit">err-no-exit</a>

<a href="TAP_COMMANDS.md#successful-move">successful-move</a> = "OK" SP "room=" <a href="TAP_COMMANDS.md#room-id">room-id</a> LF
</code></pre>

### QUIT command

Client
###### quit-request
###### quit-request-command

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#quit-request">quit-request</a> = <a href="TAP_COMMANDS.md#quit-request-command">quit-request-command</a> / &lt;server/client connection issue or program aborption&gt;

<a href="TAP_COMMANDS.md#quit-request-command">quit-request-command</a> = "QUIT" LF
</code></pre>


Server
###### quit-response

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#quit-response">quit-response</a> = "OK" SP "bye" LF
</code></pre>

## Communication Commands

### CHAT command

Client
###### chat-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#chat-request">chat-request</a> = "CHAT" SP <a href="TAP_COMMANDS.md#chat-scope">chat-scope</a> SP <a href="TAP_COMMANDS.md#chat-message">chat-message</a> LF
</code></pre>


Server
###### chat-response
###### chat-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#chat-response">chat-response</a> = <a href="TAP_COMMANDS.md#chat-success">chat-success</a> / <a href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> / <a href="TAP_ERRORS.md#err-invalid-scope">err-invalid-scope</a> / <a href="TAP_ERRORS.md#err-no-such-group">err-no-such-group</a>

<a href="TAP_COMMANDS.md#chat-success">chat-success</a> = "OK" LF
</code></pre>

### WHO command

Client
###### who-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#who-request">who-request</a> = "WHO" LF
</code></pre>


Server
###### who-response

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#who-response">who-response</a> = "OK" SP "players=" <a href="TAP_COMMANDS.md#player-server-count">player-server-count</a> LF
</code></pre>

## Group Management Commands

### GROUP CREATE command

Client
###### group-create-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#group-create-request">group-create-request</a> = "GROUP" SP "CREATE" LF
</code></pre>


Server
###### group-create-response
###### group-create-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#group-create-response">group-create-response</a> = <a href="TAP_COMMANDS.md#group-create-success">group-create-success</a> / <a href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a>

<a href="TAP_COMMANDS.md#group-create-success">group-create-success</a> = "OK" SP "group=" <a href="TAP_COMMANDS.md#group-id">group-id</a> LF
</code></pre>

### GROUP INVITE command

Client
###### group-invite-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#group-invite-request">group-invite-request</a> = "GROUP" SP "INVITE" SP <a href="TAP_COMMANDS.md#username">username</a> LF
</code></pre>


Server
###### group-invite-response
###### group-invite-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#group-invite-response">group-invite-response</a> = <a href="TAP_COMMANDS.md#group-invite-success">group-invite-success</a> / <a href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> / <a href="TAP_ERRORS.md#err-no-such-user">err-no-such-user</a> / <a href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a> / <a href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a>

<a href="TAP_COMMANDS.md#group-invite-success">group-invite-success</a> = "OK" LF
</code></pre>

### GROUP JOIN command

Client
###### group-join-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#group-join-request">group-join-request</a> = "GROUP" SP "JOIN" SP <a href="TAP_COMMANDS.md#leader-name">leader-name</a> LF
</code></pre>


Server
###### group-join-response
###### group-join-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#group-join-response">group-join-response</a> = <a href="TAP_COMMANDS.md#group-join-success">group-join-success</a> / <a href="TAP_ERRORS.md#err-no-such-user">err-no-such-user</a> / <a href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a> / <a href="TAP_ERRORS.md#err-not-invited">err-not-invited</a> / <a href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a>

<a href="TAP_COMMANDS.md#group-join-success">group-join-success</a> = "OK" SP "group=" <a href="TAP_COMMANDS.md#group-id">group-id</a> LF
</code></pre>

### GROUP LEAVE command

Client
###### group-leave-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#group-leave-request">group-leave-request</a> = "GROUP" SP "LEAVE" LF
</code></pre>


Server
###### group-leave-response
###### group-leave-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#group-leave-response">group-leave-response</a> = <a href="TAP_COMMANDS.md#group-leave-success">group-leave-success</a> / <a href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> / <a href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a>

<a href="TAP_COMMANDS.md#group-leave-success">group-leave-success</a> = "OK" LF
</code></pre>

## Resource Interaction Commands

### TAKE command

Client
###### take-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#take-request">take-request</a> = "TAKE" SP <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>


Server
###### take-response
###### take-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#take-response">take-response</a> = <a href="TAP_COMMANDS.md#take-success">take-success</a> / <a href="TAP_ERRORS.md#err-item-not-found">err-item-not-found</a>

<a href="TAP_COMMANDS.md#take-success">take-success</a> = "OK" SP "taken=" <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>

### DROP command

Client
###### drop-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#drop-request">drop-request</a> = "DROP" SP <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>


Server
###### drop-response
###### drop-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#drop-response">drop-response</a> = <a href="TAP_COMMANDS.md#drop-success">drop-success</a> / <a href="TAP_ERRORS.md#err-item-not-in-inventory">err-item-not-in-inventory</a>

<a href="TAP_COMMANDS.md#drop-success">drop-success</a> = "OK" SP "dropped=" <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>

### INVENTORY command

Client
###### inventory-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#inventory-request">inventory-request</a> = "INVENTORY" LF
</code></pre>


Server
###### inventory-response

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#inventory-response">inventory-response</a> = "OK" SP <a href="TAP_COMMANDS.md#inventory-json">inventory-json</a> LF
</code></pre>

### TALK command

Client
###### talk-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#talk-request">talk-request</a> = "TALK" SP <a href="TAP_COMMANDS.md#npc-name">npc-name</a> LF
</code></pre>


Server
###### talk-response
###### talk-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#talk-response">talk-response</a> = <a href="TAP_COMMANDS.md#talk-success">talk-success</a> / <a href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a>

<a href="TAP_COMMANDS.md#talk-success">talk-success</a> = "OK" SP <a href="TAP_COMMANDS.md#dialogue">dialogue</a> LF
</code></pre>

### ATTACK command

Client
###### attack-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#attack-request">attack-request</a> = "ATTACK" SP <a href="TAP_COMMANDS.md#npc-name">npc-name</a> LF
</code></pre>


Server
###### attack-response
###### attack-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#attack-response">attack-response</a> = <a href="TAP_COMMANDS.md#attack-success">attack-success</a> / <a href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a> / <a href="TAP_ERRORS.md#err-npc-not-hostile">err-npc-not-hostile</a>

<a href="TAP_COMMANDS.md#attack-success">attack-success</a> = "OK" SP <a href="TAP_COMMANDS.md#combat-result-json">combat-result-json</a> LF
</code></pre>

### STATUS command

Client
###### status-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#status-request">status-request</a> = "STATUS" LF
</code></pre>


Server
###### status-response

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#status-response">status-response</a> = "OK" SP <a href="TAP_COMMANDS.md#player-status-json">player-status-json</a> LF
</code></pre>

### QUEST command

Client
###### quest-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#quest-request">quest-request</a> = "QUEST" SP <a href="TAP_COMMANDS.md#npc-name">npc-name</a> LF
</code></pre>


Server
###### quest-response
###### quest-success

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#quest-response">quest-response</a> = <a href="TAP_COMMANDS.md#quest-success">quest-success</a> / <a href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a> / <a href="TAP_ERRORS.md#err-no-quest-available">err-no-quest-available</a>

<a href="TAP_COMMANDS.md#quest-success">quest-success</a> = "OK" SP <a href="TAP_COMMANDS.md#quest-data-json">quest-data-json</a> LF
</code></pre>

### QUESTS command

Client
###### quests-request

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a href="TAP_COMMANDS.md#quests-request">quests-request</a> = "QUESTS" LF
</code></pre>


Server
###### quests-response

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a href="TAP_COMMANDS.md#quests-response">quests-response</a> = "OK" SP <a href="TAP_COMMANDS.md#quest-list-json">quest-list-json</a> LF
</code></pre>
