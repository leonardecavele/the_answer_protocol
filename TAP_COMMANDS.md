# TAP COMMANDS

## Format

### Command Format

<pre><code class="language-abnf">; command format
<a id="message" href="TAP_COMMANDS.md#message">message</a> = <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="command-line" href="TAP_COMMANDS.md#command-line">command-line</a> = <a href="TAP_COMMANDS.md#command-name">command-name</a> [SP <a href="TAP_COMMANDS.md#arguments">arguments</a>] LF
<a id="command-name" href="TAP_COMMANDS.md#command-name">command-name</a> = 1*ALPHA
<a id="arguments" href="TAP_COMMANDS.md#arguments">arguments</a> = 1*(VCHAR / SP)
</code></pre>

### Success Response Format

<pre><code class="language-abnf">; success response format
<a id="response-line" href="TAP_COMMANDS.md#response-line">response-line</a> = "OK" [SP <a href="TAP_COMMANDS.md#response-data">response-data</a>] LF
<a id="response-data" href="TAP_COMMANDS.md#response-data">response-data</a> = 1*(VCHAR / SP)
</code></pre>

## Arguments

### Common Values

<pre><code class="language-abnf">; common values
<a id="line-text" href="TAP_COMMANDS.md#line-text">line-text</a> = VCHAR *(SP / VCHAR)
<a id="json-text" href="TAP_COMMANDS.md#json-text">json-text</a> = &lt;valid JSON text encoded on one line, without LF&gt;
<a id="json-array" href="TAP_COMMANDS.md#json-array">json-array</a> = &lt;valid JSON array encoded on one line, without LF&gt;
<a id="number" href="TAP_COMMANDS.md#number">number</a> = 1*DIGIT
</code></pre>

### Arguments

<pre><code class="language-abnf">; protocol values
<a id="protocol-version" href="TAP_COMMANDS.md#protocol-version">protocol-version</a> = <a href="TAP_COMMANDS.md#number">number</a>
</code></pre>

<pre><code class="language-abnf">; player values
<a id="username" href="TAP_COMMANDS.md#username">username</a> = ALPHA *(ALPHA / DIGIT / "_" / "-")
<a id="leader-name" href="TAP_COMMANDS.md#leader-name">leader-name</a> = <a href="TAP_COMMANDS.md#username">username</a>
<a id="player-server-count" href="TAP_COMMANDS.md#player-server-count">player-server-count</a> = <a href="TAP_COMMANDS.md#number">number</a>
</code></pre>

<pre><code class="language-abnf">; world values
<a id="room-id" href="TAP_COMMANDS.md#room-id">room-id</a> = 1*(ALPHA / DIGIT / "_" / "-" / ".")
<a id="direction" href="TAP_COMMANDS.md#direction">direction</a> = 1*ALPHA
</code></pre>

<pre><code class="language-abnf">; group values
<a id="group-id" href="TAP_COMMANDS.md#group-id">group-id</a> = 1*(ALPHA / DIGIT / "_" / "-" / ".")
</code></pre>

<pre><code class="language-abnf">; resource values
<a id="item-identifier" href="TAP_COMMANDS.md#item-identifier">item-identifier</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
<a id="npc-name" href="TAP_COMMANDS.md#npc-name">npc-name</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
<a id="dialogue" href="TAP_COMMANDS.md#dialogue">dialogue</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
</code></pre>

<pre><code class="language-abnf">; chat values
<a id="chat-scope" href="TAP_COMMANDS.md#chat-scope">chat-scope</a> = "GLOBAL" / "ROOM" / "GROUP"
<a id="chat-message" href="TAP_COMMANDS.md#chat-message">chat-message</a> = <a href="TAP_COMMANDS.md#line-text">line-text</a>
</code></pre>

<pre><code class="language-abnf">; json payloads
<a id="current-room-state-json" href="TAP_COMMANDS.md#current-room-state-json">current-room-state-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a id="combat-result-json" href="TAP_COMMANDS.md#combat-result-json">combat-result-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a id="player-status-json" href="TAP_COMMANDS.md#player-status-json">player-status-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a id="quest-data-json" href="TAP_COMMANDS.md#quest-data-json">quest-data-json</a> = <a href="TAP_COMMANDS.md#json-text">json-text</a>
<a id="quest-list-json" href="TAP_COMMANDS.md#quest-list-json">quest-list-json</a> = <a href="TAP_COMMANDS.md#json-array">json-array</a>
<a id="inventory-json" href="TAP_COMMANDS.md#inventory-json">inventory-json</a> = <a href="TAP_COMMANDS.md#json-array">json-array</a>
</code></pre>

## Establish Connection

Client
```bash
./client 127.0.0.1 4242
```

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="server-greeting" href="TAP_COMMANDS.md#server-greeting">server-greeting</a> = "OK" SP "hello" SP "proto=" <a href="TAP_COMMANDS.md#protocol-version">protocol-version</a> LF
</code></pre>

## Core Commands

### CONNECT command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="connect-request" href="TAP_COMMANDS.md#connect-request">connect-request</a> = "CONNECT" SP <a href="TAP_COMMANDS.md#username">username</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="connect-response" href="TAP_COMMANDS.md#connect-response">connect-response</a> = <a href="TAP_COMMANDS.md#connect-success">connect-success</a> / <a href="TAP_ERRORS.md#err-name-in-use">err-name-in-use</a> / <a href="TAP_ERRORS.md#err-already-connected">err-already-connected</a>

<a id="connect-success" href="TAP_COMMANDS.md#connect-success">connect-success</a> = "OK" SP "connected" LF
</code></pre>

### LOOK command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="look-request" href="TAP_COMMANDS.md#look-request">look-request</a> = "LOOK" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="look-response" href="TAP_COMMANDS.md#look-response">look-response</a> = "OK" SP <a href="TAP_COMMANDS.md#current-room-state-json">current-room-state-json</a> LF
</code></pre>

### MOVE command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="move-request" href="TAP_COMMANDS.md#move-request">move-request</a> = "MOVE" SP <a href="TAP_COMMANDS.md#direction">direction</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="move-response" href="TAP_COMMANDS.md#move-response">move-response</a> = <a href="TAP_COMMANDS.md#successful-move">successful-move</a> / <a href="TAP_ERRORS.md#err-no-exit">err-no-exit</a>

<a id="successful-move" href="TAP_COMMANDS.md#successful-move">successful-move</a> = "OK" SP "room=" <a href="TAP_COMMANDS.md#room-id">room-id</a> LF
</code></pre>

### QUIT command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="quit-request" href="TAP_COMMANDS.md#quit-request">quit-request</a> = <a href="TAP_COMMANDS.md#quit-request-command">quit-request-command</a> / &lt;server/client connection issue or program aborption&gt;

<a id="quit-request-command" href="TAP_COMMANDS.md#quit-request-command">quit-request-command</a> = "QUIT" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="quit-response" href="TAP_COMMANDS.md#quit-response">quit-response</a> = "OK" SP "bye" LF
</code></pre>

## Communication Commands

### CHAT command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="chat-request" href="TAP_COMMANDS.md#chat-request">chat-request</a> = "CHAT" SP <a href="TAP_COMMANDS.md#chat-scope">chat-scope</a> SP <a href="TAP_COMMANDS.md#chat-message">chat-message</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="chat-response" href="TAP_COMMANDS.md#chat-response">chat-response</a> = <a href="TAP_COMMANDS.md#chat-success">chat-success</a> / <a href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> / <a href="TAP_ERRORS.md#err-invalid-scope">err-invalid-scope</a> / <a href="TAP_ERRORS.md#err-no-such-group">err-no-such-group</a>

<a id="chat-success" href="TAP_COMMANDS.md#chat-success">chat-success</a> = "OK" LF
</code></pre>

### WHO command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="who-request" href="TAP_COMMANDS.md#who-request">who-request</a> = "WHO" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="who-response" href="TAP_COMMANDS.md#who-response">who-response</a> = "OK" SP "players=" <a href="TAP_COMMANDS.md#player-server-count">player-server-count</a> LF
</code></pre>

## Group Management Commands

### GROUP CREATE command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="group-create-request" href="TAP_COMMANDS.md#group-create-request">group-create-request</a> = "GROUP" SP "CREATE" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="group-create-response" href="TAP_COMMANDS.md#group-create-response">group-create-response</a> = <a href="TAP_COMMANDS.md#group-create-success">group-create-success</a> / <a href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a>

<a id="group-create-success" href="TAP_COMMANDS.md#group-create-success">group-create-success</a> = "OK" SP "group=" <a href="TAP_COMMANDS.md#group-id">group-id</a> LF
</code></pre>

### GROUP INVITE command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="group-invite-request" href="TAP_COMMANDS.md#group-invite-request">group-invite-request</a> = "GROUP" SP "INVITE" SP <a href="TAP_COMMANDS.md#username">username</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="group-invite-response" href="TAP_COMMANDS.md#group-invite-response">group-invite-response</a> = <a href="TAP_COMMANDS.md#group-invite-success">group-invite-success</a> / <a href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> / <a href="TAP_ERRORS.md#err-no-such-user">err-no-such-user</a> / <a href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a> / <a href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a>

<a id="group-invite-success" href="TAP_COMMANDS.md#group-invite-success">group-invite-success</a> = "OK" LF
</code></pre>

### GROUP JOIN command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="group-join-request" href="TAP_COMMANDS.md#group-join-request">group-join-request</a> = "GROUP" SP "JOIN" SP <a href="TAP_COMMANDS.md#leader-name">leader-name</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="group-join-response" href="TAP_COMMANDS.md#group-join-response">group-join-response</a> = <a href="TAP_COMMANDS.md#group-join-success">group-join-success</a> / <a href="TAP_ERRORS.md#err-no-such-user">err-no-such-user</a> / <a href="TAP_ERRORS.md#err-already-in-group">err-already-in-group</a> / <a href="TAP_ERRORS.md#err-not-invited">err-not-invited</a> / <a href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a>

<a id="group-join-success" href="TAP_COMMANDS.md#group-join-success">group-join-success</a> = "OK" SP "group=" <a href="TAP_COMMANDS.md#group-id">group-id</a> LF
</code></pre>

### GROUP LEAVE command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="group-leave-request" href="TAP_COMMANDS.md#group-leave-request">group-leave-request</a> = "GROUP" SP "LEAVE" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="group-leave-response" href="TAP_COMMANDS.md#group-leave-response">group-leave-response</a> = <a href="TAP_COMMANDS.md#group-leave-success">group-leave-success</a> / <a href="TAP_ERRORS.md#err-not-in-group">err-not-in-group</a> / <a href="TAP_ERRORS.md#err-group-not-found">err-group-not-found</a>

<a id="group-leave-success" href="TAP_COMMANDS.md#group-leave-success">group-leave-success</a> = "OK" LF
</code></pre>

## Resource Interaction Commands

### TAKE command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="take-request" href="TAP_COMMANDS.md#take-request">take-request</a> = "TAKE" SP <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="take-response" href="TAP_COMMANDS.md#take-response">take-response</a> = <a href="TAP_COMMANDS.md#take-success">take-success</a> / <a href="TAP_ERRORS.md#err-item-not-found">err-item-not-found</a>

<a id="take-success" href="TAP_COMMANDS.md#take-success">take-success</a> = "OK" SP "taken=" <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>

### DROP command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="drop-request" href="TAP_COMMANDS.md#drop-request">drop-request</a> = "DROP" SP <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="drop-response" href="TAP_COMMANDS.md#drop-response">drop-response</a> = <a href="TAP_COMMANDS.md#drop-success">drop-success</a> / <a href="TAP_ERRORS.md#err-item-not-in-inventory">err-item-not-in-inventory</a>

<a id="drop-success" href="TAP_COMMANDS.md#drop-success">drop-success</a> = "OK" SP "dropped=" <a href="TAP_COMMANDS.md#item-identifier">item-identifier</a> LF
</code></pre>

### INVENTORY command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="inventory-request" href="TAP_COMMANDS.md#inventory-request">inventory-request</a> = "INVENTORY" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="inventory-response" href="TAP_COMMANDS.md#inventory-response">inventory-response</a> = "OK" SP <a href="TAP_COMMANDS.md#inventory-json">inventory-json</a> LF
</code></pre>

### TALK command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="talk-request" href="TAP_COMMANDS.md#talk-request">talk-request</a> = "TALK" SP <a href="TAP_COMMANDS.md#npc-name">npc-name</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="talk-response" href="TAP_COMMANDS.md#talk-response">talk-response</a> = <a href="TAP_COMMANDS.md#talk-success">talk-success</a> / <a href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a>

<a id="talk-success" href="TAP_COMMANDS.md#talk-success">talk-success</a> = "OK" SP <a href="TAP_COMMANDS.md#dialogue">dialogue</a> LF
</code></pre>

### ATTACK command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="attack-request" href="TAP_COMMANDS.md#attack-request">attack-request</a> = "ATTACK" SP <a href="TAP_COMMANDS.md#npc-name">npc-name</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="attack-response" href="TAP_COMMANDS.md#attack-response">attack-response</a> = <a href="TAP_COMMANDS.md#attack-success">attack-success</a> / <a href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a> / <a href="TAP_ERRORS.md#err-npc-not-hostile">err-npc-not-hostile</a>

<a id="attack-success" href="TAP_COMMANDS.md#attack-success">attack-success</a> = "OK" SP <a href="TAP_COMMANDS.md#combat-result-json">combat-result-json</a> LF
</code></pre>

### STATUS command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="status-request" href="TAP_COMMANDS.md#status-request">status-request</a> = "STATUS" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="status-response" href="TAP_COMMANDS.md#status-response">status-response</a> = "OK" SP <a href="TAP_COMMANDS.md#player-status-json">player-status-json</a> LF
</code></pre>

### QUEST command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="quest-request" href="TAP_COMMANDS.md#quest-request">quest-request</a> = "QUEST" SP <a href="TAP_COMMANDS.md#npc-name">npc-name</a> LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="quest-response" href="TAP_COMMANDS.md#quest-response">quest-response</a> = <a href="TAP_COMMANDS.md#quest-success">quest-success</a> / <a href="TAP_ERRORS.md#err-npc-not-found">err-npc-not-found</a> / <a href="TAP_ERRORS.md#err-no-quest-available">err-no-quest-available</a>

<a id="quest-success" href="TAP_COMMANDS.md#quest-success">quest-success</a> = "OK" SP <a href="TAP_COMMANDS.md#quest-data-json">quest-data-json</a> LF
</code></pre>

### QUESTS command

Client

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#command-line">command-line</a>
<a id="quests-request" href="TAP_COMMANDS.md#quests-request">quests-request</a> = "QUESTS" LF
</code></pre>

Server

<pre><code class="language-abnf">; <a href="TAP_COMMANDS.md#response-line">response-line</a>
<a id="quests-response" href="TAP_COMMANDS.md#quests-response">quests-response</a> = "OK" SP <a href="TAP_COMMANDS.md#quest-list-json">quest-list-json</a> LF
</code></pre>
