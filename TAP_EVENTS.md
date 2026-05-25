# TAP EVENTS

## Format

<pre><code class="language-abnf">; event format
<a id="message" href="TAP_EVENTS.md#message">message</a> = <a href="TAP_EVENTS.md#event-line">event-line</a>
<a id="event-line" href="TAP_EVENTS.md#event-line">event-line</a> = "EVT" SP <a href="TAP_EVENTS.md#event-type">event-type</a> SP <a href="TAP_EVENTS.md#event-data">event-data</a> LF
<a id="event-type" href="TAP_EVENTS.md#event-type">event-type</a> = 1*ALPHA
<a id="event-data" href="TAP_EVENTS.md#event-data">event-data</a> = 1*(VCHAR / SP)
</code></pre>

## Arguments

<pre><code class="language-abnf">; event arguments format
<a id="username" href="TAP_EVENTS.md#username">username</a> = ALPHA *(ALPHA / DIGIT / "_" / "-")
<a id="leader-name" href="TAP_EVENTS.md#leader-name">leader-name</a> = <a href="TAP_EVENTS.md#username">username</a>
<a id="chat-message" href="TAP_EVENTS.md#chat-message">chat-message</a> = VCHAR *(SP / VCHAR)
<a id="player-server-count" href="TAP_EVENTS.md#player-server-count">player-server-count</a> = 1*DIGIT
</code></pre>

## Room Events

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a id="room-event" href="TAP_EVENTS.md#room-event">room-event</a> = <a href="TAP_EVENTS.md#room-presence-enter-event">room-presence-enter-event</a> / <a href="TAP_EVENTS.md#room-presence-leave-event">room-presence-leave-event</a> / <a href="TAP_EVENTS.md#room-chat-event">room-chat-event</a>

<a id="room-presence-enter-event" href="TAP_EVENTS.md#room-presence-enter-event">room-presence-enter-event</a> = "EVT" SP "ROOM" SP "PRESENCE" SP "ENTER" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a id="room-presence-leave-event" href="TAP_EVENTS.md#room-presence-leave-event">room-presence-leave-event</a> = "EVT" SP "ROOM" SP "PRESENCE" SP "LEAVE" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a id="room-chat-event" href="TAP_EVENTS.md#room-chat-event">room-chat-event</a> = "EVT" SP "ROOM" SP "CHAT" SP <a href="TAP_EVENTS.md#username">username</a> SP <a href="TAP_EVENTS.md#chat-message">chat-message</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT ROOM PRESENCE ENTER <username>` | Player entered the current room |
| `EVT ROOM PRESENCE LEAVE <username>` | Player left the current room |
| `EVT ROOM CHAT <username> <message>` | Room-scoped chat message |

## Global Events

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a id="global-event" href="TAP_EVENTS.md#global-event">global-event</a> = <a href="TAP_EVENTS.md#global-chat-event">global-chat-event</a>

<a id="global-chat-event" href="TAP_EVENTS.md#global-chat-event">global-chat-event</a> = "EVT" SP "GLOBAL" SP "CHAT" SP <a href="TAP_EVENTS.md#username">username</a> SP <a href="TAP_EVENTS.md#chat-message">chat-message</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT GLOBAL CHAT <username> <message>` | Server-wide chat message |

## Group Events

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a id="group-event" href="TAP_EVENTS.md#group-event">group-event</a> = <a href="TAP_EVENTS.md#group-invite-event">group-invite-event</a> / <a href="TAP_EVENTS.md#group-join-event">group-join-event</a> / <a href="TAP_EVENTS.md#group-leave-event">group-leave-event</a> / <a href="TAP_EVENTS.md#group-chat-event">group-chat-event</a>

<a id="group-invite-event" href="TAP_EVENTS.md#group-invite-event">group-invite-event</a> = "EVT" SP "GROUP" SP "INVITE" SP <a href="TAP_EVENTS.md#leader-name">leader-name</a> LF
<a id="group-join-event" href="TAP_EVENTS.md#group-join-event">group-join-event</a> = "EVT" SP "GROUP" SP "JOIN" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a id="group-leave-event" href="TAP_EVENTS.md#group-leave-event">group-leave-event</a> = "EVT" SP "GROUP" SP "LEAVE" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a id="group-chat-event" href="TAP_EVENTS.md#group-chat-event">group-chat-event</a> = "EVT" SP "GROUP" SP "CHAT" SP <a href="TAP_EVENTS.md#username">username</a> SP <a href="TAP_EVENTS.md#chat-message">chat-message</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT GROUP INVITE <leader>` | Group invitation received |
| `EVT GROUP JOIN <username>` | Player joined the group |
| `EVT GROUP LEAVE <username>` | Player left the group |
| `EVT GROUP CHAT <username> <message>` | Group-scoped chat message |

## Stats Events

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a id="stats-event" href="TAP_EVENTS.md#stats-event">stats-event</a> = "EVT" SP "STATS" SP "players=" <a href="TAP_EVENTS.md#player-server-count">player-server-count</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT STATS players=<count>` | Updated player count |
