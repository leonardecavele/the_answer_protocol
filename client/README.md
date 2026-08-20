# CLIENT MODEL

## User State (Client Side)

### As Graph

```mermaid
stateDiagram-v2
    [*] --> DISCONNECTED

    DISCONNECTED --> CONNECTED: TCP connection established
    DISCONNECTED --> TERMINATED: connection attempt failed / fatal error

    CONNECTED --> AUTHENTICATED: CONNECT <username> accepted
    CONNECTED --> TERMINATED: authentication failed / socket closed / error

    AUTHENTICATED --> TERMINATED: QUIT / socket closed / error
```

```mermaid
flowchart TD
    TICK["EventBroker<br/>tick toutes les 33 ms"] --> CH
    TERM["EventBroker<br/>crossterm EventStream"] --> CH
    NET["NetworkManager<br/>réponses API + événements serveur"] --> CH

    CH[["mpsc channel · ApplicationEvent · cap 100"]] --> NEXT["App::run<br/>next_event await"]

    NEXT --> UPDATE["App::update"]
    UPDATE --> DRAIN{"try_next_event<br/>file vide ?"}
    DRAIN -- "non : on vide la file" --> UPDATE
    DRAIN -- "oui" --> RENDER["App::render<br/>terminal.draw"]
    RENDER --> NEXT
```

### As C Code

```c
typedef enum e_client_state
{
    DISCONNECTED,
    CONNECTED,
    AUTHENTICATED,
    TERMINATED
}   t_client_state;
```
