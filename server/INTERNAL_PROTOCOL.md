## Rust server <-> Go server communication protocol

## abnf format:
```abnf
ALPHA-LOWER = %x61-7A
DIGIT = %x30-39
snake_case = ALPHA-LOWER *( ALPHA-LOWER / "_" ALPHA-LOWER )

player_name = snake_case
command_name = snake_case
command_id = 1*DIGIT
argument_name = snake_case
argument_value = snake_case / 1*DIGIT
error_code = 1*DIGIT
response_value = snake_case / 1*DIGIT
```

## Go -> Rust

#### Json format: 

```json
{
    "player": <player_name>,
    "command": <command_name>,
    "command_id": <command_id>,
    "arguments": {
        <argument_name>: <argument_value>
    }
}
```



## arguments format for all commands

### ``arguments`` for `CONNECT` command
```json
{}
```

### ``arguments`` for `LOOK` command
```json
{}
```

### ``arguments`` for `MOVE` command
```json
{
    "direction": NORTH / SOUTH / EAST / WEST
}
```

### ``arguments`` for ``QUIT`` command

```json
{}
```

### ``arguments`` for ``CHAT`` command

```json
{
    "scope": GLOBAL / ROOM / GROUP,
    "message": <message>
}
```

### ``arguments`` for ``WHO`` command
```json
{}
```

### ``arguments`` for ``GROUP CREATE`` command
```json
{}
```

### ``arguments`` for ``GROUP INVITE`` command
```json
{
    "username": <username>
}
```

### ``arguments`` for ``GROUP JOIN`` command
```json
{
    "leader": <username>
}
```

### ``arguments`` for ``GROUP LEAVE`` command
```json
{}
```

### ``arguments`` for ``TAKE`` command
```json
{
    "item": <item_id>
}
```

### ``arguments`` for ``DROP`` command
```json
{
    "item": <item_id>
}
```

### ``arguments`` for ``INVENTORY`` command
```json
{}
```

### ``arguments`` for ``TALK`` command
```json
{
    "npc": <npc_name>
}
```

### ``arguments`` for ``ATTACK`` command
```json
{
    "npc": <npc_name>
}
```

### ``arguments`` for ``STATUS`` command
```json
{}
```

### ``arguments`` for ``QUEST`` command
```json
{
    "npc": <npc_name>
}
```

### ``arguments`` for ``QUESTS`` command
```json
{}
```

## Rust -> Go

#### Json format: 

```json
{
    "player": <player_name>,
    "command_id": <command_id>,
    "error_code": <error_code>,
    "value": <response_value>
}
```

## ``value format for all commands``

### ``value`` for ``CONNECT`` command
```json
{
    "value": ""
}

```

### ``value`` for ``LOOK`` command
(See [Json Room Format in README.md](README.md#json-room-format) for details on the current-room-state-json structure)


### ``value`` for ``MOVE`` command
```json
    "value": <room_id>
```

### ``value`` for ``QUIT`` command
```json
    "value": ""
```

### ``value`` for ``CHAT`` command
```json
    "value": ""
```

### ``value`` for ``WHO`` command
```json
    "value": <player_server_count>
```

### ``value`` for ``GROUP CREATE`` command
```json
    "value": <group_id>
```

### ``value`` for ``GROUP INVITE`` command
```json
    "value": ""
```

### ``value`` for ``GROUP JOIN`` command
```json
    "value": <group_id>
```

### ``value`` for ``GROUP LEAVE`` command
```json
    "value": ""
```

### ``value`` for ``TAKE`` command
```json
    "value": <item_id>
```

### ``value`` for ``DROP`` command
```json
    "value": <item_id>
```

### ``value`` for ``INVENTORY`` command
(See [Json Inventory Format in README.md](README.md#json-inventory-format) for details on the inventory-json structure)


### ``value`` for ``TALK`` command
```json
    "value": <dialogue>
```

### ``value`` for ``ATTACK`` command
(See [Json Combat Result Format in README.md](README.md#json-combat-result-format) for details on the combat-result-json structure)

### ``value`` for ``STATUS`` command
(See [Json Status Format in README.md](README.md#json-status-format) for details on the player-status-json structure)

### ``value`` for ``QUEST`` command
(See [Json Quest Data Format in README.md](README.md#json-quest-data-format) for details on the quest-data-json structure)

### ``value`` for ``QUESTS`` command
(See [Json Quest List Format in README.md](README.md#json-quest-list-format) for details on the quest-list-json structure) 