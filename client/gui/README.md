The `gui` crate provides the native graphical interface for The Answer
Protocol. It reuses the complete Ratatui `App`, including its screens, state,
networking, focus model, commands, and widgets. Egui supplies the window and
input events; a software Ratatui backend supplies the character-cell display.

Public command behavior and server frames are defined in the root
[TAP protocol reference](../../PROTOCOL.md). The transport layer is documented
in the [Client API README](../client-api/README.md).

## Rendering pipeline

```mermaid
flowchart LR
    EE["Egui Event"] --> CE["Crossterm Event"]
    CE --> App["App"]
    App --> Ratatui["Ratatui"]
    Ratatui --> SB["Soft Ratatui Backend"]
    SB --> EW["Egui Window"]
```

The GUI does not start or embed a terminal process. Instead:

1. Egui keyboard, text, pointer, and wheel input is translated into Crossterm
   events.
2. Those events are passed directly to the shared `App`.
3. The app draws its existing widgets with Ratatui.
4. `soft_ratatui` renders the Ratatui cells into an in-memory grid.
5. `egui_ratatui` displays that grid inside the Eframe window.

This preserves identical client behavior across the GUI and [TUI](../tui/README.md) without a
pseudo-terminal or a second set of screens.

## Requirements

- Rust toolchain with Cargo and Rust 2024 edition support
- A native desktop environment supported by Eframe
- A [TAP gateway](../../server/go_server/README.md) on `127.0.0.1:38800`

## Build and run

From the repository root:

```bash
make build-client-gui
make run-client-gui
```

The native window is titled `The Answer Protocol` and connects to the default
gateway at `127.0.0.1:38800`.

The GUI crate uses these frontend dependencies:

| Crate | Responsibility |
| --- | --- |
| `eframe` / `egui` | Native application lifecycle, window, and input. |
| `tui` | Shared `App`, state, events, views, and widgets. |
| `ratatui` | Backend-independent terminal UI drawing. |
| `soft_ratatui` | In-memory character-cell renderer and font atlases. |
| `egui_ratatui` | Egui widget that paints the software terminal. |
| `crossterm` | Common event representation consumed by `App`. |
| `tokio` | Runtime used by the shared asynchronous application. |

## Runtime behavior

`GuiApp` owns four long-lived values:

- the shared `tui::app::App`;
- a software `ratatui::Terminal`;
- the active Tokio runtime handle;
- the current Egui-to-terminal cell grid.

On each Eframe logic pass, the GUI enters the Tokio runtime, converts pending
Egui input, drains application events, updates the shared app, draws a new
Ratatui frame, and schedules the next repaint after the standard 33 ms tick.
When `App` requests shutdown, the GUI closes the native viewport.

The software terminal starts at 120 columns by 40 rows and uses 9-by-18 regular
and bold monospace atlases. Cells with an unset background are normalized to
black before display. The drawable area is clamped to the current Egui window
and graphics texture limit.

## Input translation

The input adapter converts these Egui events:

| Egui input | Crossterm representation |
| --- | --- |
| Text input | One `KeyCode::Char` event per character. |
| Arrow keys | `Up`, `Down`, `Left`, and `Right`. |
| `Enter`, `Escape`, `Backspace` | Matching Crossterm key codes. |
| `PageUp`, `PageDown`, `F1` | Matching Crossterm key codes. |
| `Tab` / `Shift+Tab` | `Tab` / `BackTab`. |
| `Ctrl` plus a letter | Control-modified character event. |
| Copy request | `Ctrl+C`. |
| Primary pointer press | Left-button event at the corresponding cell. |
| Mouse wheel | `ScrollUp` or `ScrollDown` at the pointed cell. |

Ctrl, Alt, and Shift modifiers are preserved. Pointer coordinates are mapped
from Egui pixels to Ratatui column and row coordinates using the active cell
size and displayed grid rectangle. Wheel events modified with Ctrl remain
available to Egui for native zoom behavior.

## Controls

Because the GUI feeds the same event type into the same `App`, it retains the
TUI controls:

| Input | Action |
| --- | --- |
| `Ctrl+C` | Quit. |
| `Ctrl+H` | Toggle help. |
| `Ctrl+E` | Toggle events and traces. |
| `F1` | Toggle chat. |
| `Tab` / `Shift+Tab` | Cycle focus. |
| Arrow keys | Navigate, select, or move according to focus. |
| `Enter` | Submit or activate the focused action. |
| `Esc` | Close the active popup. |
| Left click | Focus or select a mapped terminal cell. |
| Mouse wheel | Scroll the panel beneath the pointer. |
| `Ctrl+S` in the fight editor | Submit the C solution. |

The command input accepts the same full commands and aliases as the terminal
client. Contextual room, inventory, NPC, quest, group, chat, and navigation
panels expose `LOOK`, `MOVE`, `TAKE`, `DROP`, `USE`, `TALK`, `ATTACK`,
`STATUS`, `QUEST`, `QUESTS`, `WHO`, `GROUP`, and `QUIT` actions. Room state and
server/player counters update from command responses and asynchronous events.

## Source layout

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Tokio runtime and native Eframe startup. |
| `src/app.rs` | Shared-app ownership and Eframe update/draw integration. |
| `src/input.rs` | Egui-to-Crossterm keyboard and pointer conversion. |
| `src/screen.rs` | Software terminal, cell grid, fonts, and Egui sizing. |

## Validation

```bash
cargo test --manifest-path client/gui/Cargo.toml
cargo clippy --manifest-path client/gui/Cargo.toml --all-targets -- -D warnings
```
