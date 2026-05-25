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
