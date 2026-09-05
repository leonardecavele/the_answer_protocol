# Rust TAP API client

`api-client` is the reusable asynchronous Rust client for The Answer Protocol.
It owns TCP connection setup, the version-1 handshake, TAP encoding and
decoding, typed requests and responses, asynchronous events, raw-frame
inspection, connection state, timeouts, and shutdown.

The wire grammar consumed and produced by this crate is defined in the root
[TAP protocol reference](../../PROTOCOL.md).

## Add the crate

From another package in the client workspace:

```toml
[dependencies]
api-client = { path = "../api-client" }
tokio = { version = "1", features = ["full"] }
```

## Minimal example

```rust
use api_client::{ApiRequest, Client, Connection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Connection {
        mut client,
        mut events,
        ..
    } = Client::connect("127.0.0.1:38800").await?;

    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            println!("event: {event:?}");
        }
    });

    client.login("ALICE".to_string()).await??;

    let look = ApiRequest::parse("look").expect("valid LOOK request");
    let response = client.execute_request(look).await?;
    println!("response: {response:?}");

    client.close().await;
    Ok(())
}
```

`Client::connect` returns a `Connection` containing:

- `client`, the command and lifecycle handle;
- `events`, the first broadcast receiver for decoded server events;
- `frames`, the first broadcast receiver for raw sent and received frames.

## Connection model

`Client::connect`:

1. opens the TCP connection within the configured deadline;
2. reads `OK hello proto=<version>`;
3. validates protocol version 1;
4. creates bounded request, event, and frame channels;
5. starts one background bridge task;
6. returns the client and initial subscriptions.

The bridge serializes commands so only one request awaits an `OK` or `ERR` on
the wire at a time. Interleaved `EVT` frames are decoded and broadcast without
consuming that pending response. Every sent and received line is also
published as a directional `Frame`.

Connection state is available through a Tokio watch receiver:

```rust
let mut state = client.state();
state.changed().await?;
println!("state: {:?}", *state.borrow());
```

The values are `Connected`, `Closed`, and `Lost(String)`.

## Requests and responses

`ApiRequest::parse` accepts full commands and TUI-friendly aliases without
case sensitivity:

| Area | Accepted input |
| --- | --- |
| Core | `connect`, `quit`, `look`, `move`, `who` |
| Fight | `fight create` / `fc`, `fight attack` / `fa` |
| Chat | `chat global` / `say`, `chat room` / `cr`, `chat group` / `cg`, `chat private` / `msg` |
| Resources | `take`, `drop`, `inventory` / `inv`, `use`, `status`, `talk`, `attack` |
| Quests | `quest`, `quests` |
| Groups | `group create` / `gc`, `group join` / `gj`, `group leave` / `gl`, `group invite` / `gi` |

`Client::execute_request` returns the matching `ApiResponse` variant. Typed
command structures and response data are re-exported from
`api_client::commands` for callers that do not parse textual input.

Command execution uses two error layers:

- the outer `Result<ApiResponse, TapError>` reports local transport, protocol,
  or bridge failure;
- each `ApiResponse` variant contains a `Result<T, CommandError>` representing
  the server's `OK` or `ERR` outcome.

`ApiResponse::get_error` provides uniform access to the inner command error.

## Events and frames

The `events` module exposes typed session, server, room, chat, group, spawn,
death, kill, and fight events. Valid but unrecognized events are preserved as
`ServerEvent::Unknown`.

Create additional receivers at any time:

```rust
let mut events = client.subscribe();
let mut frames = client.subscribe_frames();
```

Broadcast receivers are independent. A slow receiver may lag without blocking
the bridge or other subscribers.

## Configuration

Use `Client::connect_with(address, ClientConfig)` to override defaults.

| Setting | Default | Purpose |
| --- | --- | --- |
| `connect_timeout` | 5 seconds | TCP connection deadline. |
| `handshake_timeout` | 2 seconds | TAP greeting deadline. |
| `request_timeout` | 10 seconds | Per-command response deadline. |
| `close_timeout` | 2 seconds | Bridge shutdown grace period. |
| `max_frame_length` | 65,536 bytes | Largest frame decoded by the client. |
| `command_channel_capacity` | 512 | Queued command requests. |
| `event_channel_capacity` | 512 | Decoded event backlog. |
| `frame_channel_capacity` | 512 | Raw frame backlog. |

The public [gateway](../../server/go_server/README.md) accepts client frames up to 4,096 bytes, so outgoing
commands remain within that server-side limit.

## Errors

| Type | Meaning |
| --- | --- |
| `CommandError` | Parsed server `ERR` with code and friendly message. |
| `NetworkError` | Connection, codec, timeout, or disconnection failure. |
| `ProtocolError` | Invalid frame, arguments, version, or response payload. |
| `InternalError` | The background bridge cannot receive or complete work. |
| `TapError` | Wrapper for network, protocol, and internal failures. |

## Shutdown

`client.close().await` cancels the bridge and waits up to `close_timeout` for a
clean exit. Dropping the client also aborts its background task, making resource
cleanup deterministic when an owning application exits early.

## Validation

From the repository root:

```bash
cargo test --manifest-path client/api-client/Cargo.toml
cargo clippy --manifest-path client/api-client/Cargo.toml --all-targets -- -D warnings
```
