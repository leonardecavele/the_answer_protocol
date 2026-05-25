<style>
h6 { display: none; }
</style>

# TAP EVENTS

## Format

###### message
###### event-line
###### event-type
###### event-data

<pre><code class="language-abnf">; event format
<a href="TAP_EVENTS.md#message">message</a> = <a href="TAP_EVENTS.md#event-line">event-line</a>
<a href="TAP_EVENTS.md#event-line">event-line</a> = "EVT" SP <a href="TAP_EVENTS.md#event-type">event-type</a> SP <a href="TAP_EVENTS.md#event-data">event-data</a> LF
<a href="TAP_EVENTS.md#event-type">event-type</a> = 1*ALPHA
<a href="TAP_EVENTS.md#event-data">event-data</a> = 1*(VCHAR / SP)
</code></pre>

## Arguments

###### username
###### leader-name
###### chat-message
###### player-server-count

<pre><code class="language-abnf">; event arguments format
<a href="TAP_EVENTS.md#username">username</a> = ALPHA *(ALPHA / DIGIT / "_" / "-")
<a href="TAP_EVENTS.md#leader-name">leader-name</a> = <a href="TAP_EVENTS.md#username">username</a>
<a href="TAP_EVENTS.md#chat-message">chat-message</a> = VCHAR *(SP / VCHAR)
<a href="TAP_EVENTS.md#player-server-count">player-server-count</a> = 1*DIGIT
</code></pre>

## Room Events

###### room-event
###### room-presence-enter-event
###### room-presence-leave-event
###### room-chat-event

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a href="TAP_EVENTS.md#room-event">room-event</a> = <a href="TAP_EVENTS.md#room-presence-enter-event">room-presence-enter-event</a> / <a href="TAP_EVENTS.md#room-presence-leave-event">room-presence-leave-event</a> / <a href="TAP_EVENTS.md#room-chat-event">room-chat-event</a>

<a href="TAP_EVENTS.md#room-presence-enter-event">room-presence-enter-event</a> = "EVT" SP "ROOM" SP "PRESENCE" SP "ENTER" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a href="TAP_EVENTS.md#room-presence-leave-event">room-presence-leave-event</a> = "EVT" SP "ROOM" SP "PRESENCE" SP "LEAVE" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a href="TAP_EVENTS.md#room-chat-event">room-chat-event</a> = "EVT" SP "ROOM" SP "CHAT" SP <a href="TAP_EVENTS.md#username">username</a> SP <a href="TAP_EVENTS.md#chat-message">chat-message</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT ROOM PRESENCE ENTER <username>` | Player entered the current room |
| `EVT ROOM PRESENCE LEAVE <username>` | Player left the current room |
| `EVT ROOM CHAT <username> <message>` | Room-scoped chat message |

## Global Events

###### global-event
###### global-chat-event

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a href="TAP_EVENTS.md#global-event">global-event</a> = <a href="TAP_EVENTS.md#global-chat-event">global-chat-event</a>

<a href="TAP_EVENTS.md#global-chat-event">global-chat-event</a> = "EVT" SP "GLOBAL" SP "CHAT" SP <a href="TAP_EVENTS.md#username">username</a> SP <a href="TAP_EVENTS.md#chat-message">chat-message</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT GLOBAL CHAT <username> <message>` | Server-wide chat message |

## Group Events

###### group-event
###### group-invite-event
###### group-join-event
###### group-leave-event
###### group-chat-event

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a href="TAP_EVENTS.md#group-event">group-event</a> = <a href="TAP_EVENTS.md#group-invite-event">group-invite-event</a> / <a href="TAP_EVENTS.md#group-join-event">group-join-event</a> / <a href="TAP_EVENTS.md#group-leave-event">group-leave-event</a> / <a href="TAP_EVENTS.md#group-chat-event">group-chat-event</a>

<a href="TAP_EVENTS.md#group-invite-event">group-invite-event</a> = "EVT" SP "GROUP" SP "INVITE" SP <a href="TAP_EVENTS.md#leader-name">leader-name</a> LF
<a href="TAP_EVENTS.md#group-join-event">group-join-event</a> = "EVT" SP "GROUP" SP "JOIN" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a href="TAP_EVENTS.md#group-leave-event">group-leave-event</a> = "EVT" SP "GROUP" SP "LEAVE" SP <a href="TAP_EVENTS.md#username">username</a> LF
<a href="TAP_EVENTS.md#group-chat-event">group-chat-event</a> = "EVT" SP "GROUP" SP "CHAT" SP <a href="TAP_EVENTS.md#username">username</a> SP <a href="TAP_EVENTS.md#chat-message">chat-message</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT GROUP INVITE <leader>` | Group invitation received |
| `EVT GROUP JOIN <username>` | Player joined the group |
| `EVT GROUP LEAVE <username>` | Player left the group |
| `EVT GROUP CHAT <username> <message>` | Group-scoped chat message |

## Stats Events

###### stats-event

<pre><code class="language-abnf">; <a href="TAP_EVENTS.md#event-line">event-line</a>
<a href="TAP_EVENTS.md#stats-event">stats-event</a> = "EVT" SP "STATS" SP "players=" <a href="TAP_EVENTS.md#player-server-count">player-server-count</a> LF
</code></pre>

| Event | Meaning |
|---|---|
| `EVT STATS players=<count>` | Updated player count |
