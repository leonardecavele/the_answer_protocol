use crate::screen::Grid;
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use eframe::egui::{Context, Event, Key, Modifiers, PointerButton, Pos2};

pub fn to_crossterm_events(ctx: &Context, grid: Option<&Grid>) -> Vec<CrosstermEvent> {
    let mut events = Vec::new();

    ctx.input(|input| {
        let pointer = input.pointer.latest_pos();

        for event in &input.events {
            push_crossterm_events(event, grid, pointer, &mut events);
        }
    });

    events
}

fn push_crossterm_events(
    event: &Event,
    grid: Option<&Grid>,
    pointer: Option<Pos2>,
    events: &mut Vec<CrosstermEvent>,
) {
    match event {
        Event::Text(text) => {
            for character in text.chars() {
                events.push(to_key_event(KeyCode::Char(character), KeyModifiers::NONE));
            }
        }
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => {
            if let Some(code) = to_key_code(*key, *modifiers) {
                events.push(to_key_event(code, to_key_modifiers(*modifiers)));
            }
        }
        Event::Copy => events.push(to_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers,
        } => {
            if let Some(cell) = grid.and_then(|grid| grid.cell_at(*pos)) {
                let kind = MouseEventKind::Down(MouseButton::Left);
                events.push(to_mouse_event(kind, cell, *modifiers));
            }
        }
        Event::PointerMoved(position) => {
            if let Some(cell) = grid.and_then(|grid| grid.cell_at(*position)) {
                events.push(to_mouse_event(
                    MouseEventKind::Moved,
                    cell,
                    Modifiers::default(),
                ));
            }
        }
        Event::MouseWheel {
            delta, modifiers, ..
        } if !modifiers.ctrl && delta.y != 0.0 => {
            let cell = pointer.and_then(|position| grid.and_then(|grid| grid.cell_at(position)));

            if let Some(cell) = cell {
                let kind = if delta.y > 0.0 {
                    MouseEventKind::ScrollUp
                } else {
                    MouseEventKind::ScrollDown
                };

                events.push(to_mouse_event(kind, cell, *modifiers));
            }
        }
        _ => {}
    }
}

fn to_key_code(key: Key, modifiers: Modifiers) -> Option<KeyCode> {
    let code = match key {
        Key::ArrowUp => KeyCode::Up,
        Key::ArrowDown => KeyCode::Down,
        Key::ArrowLeft => KeyCode::Left,
        Key::ArrowRight => KeyCode::Right,
        Key::Enter => KeyCode::Enter,
        Key::Escape => KeyCode::Esc,
        Key::Backspace => KeyCode::Backspace,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::F1 => KeyCode::F(1),
        Key::Tab if modifiers.shift => KeyCode::BackTab,
        Key::Tab => KeyCode::Tab,
        _ if modifiers.ctrl => KeyCode::Char(to_control_letter(key)?),
        _ => return None,
    };

    Some(code)
}

fn to_control_letter(key: Key) -> Option<char> {
    let mut characters = key.name().chars();

    match (characters.next(), characters.next()) {
        (Some(letter), None) if letter.is_ascii_alphabetic() => Some(letter.to_ascii_lowercase()),
        _ => None,
    }
}

fn to_key_modifiers(modifiers: Modifiers) -> KeyModifiers {
    let mut translated = KeyModifiers::NONE;

    if modifiers.ctrl {
        translated |= KeyModifiers::CONTROL;
    }

    if modifiers.alt {
        translated |= KeyModifiers::ALT;
    }

    if modifiers.shift {
        translated |= KeyModifiers::SHIFT;
    }

    translated
}

fn to_key_event(code: KeyCode, modifiers: KeyModifiers) -> CrosstermEvent {
    CrosstermEvent::Key(KeyEvent::new(code, modifiers))
}

fn to_mouse_event(kind: MouseEventKind, cell: (u16, u16), modifiers: Modifiers) -> CrosstermEvent {
    CrosstermEvent::Mouse(MouseEvent {
        kind,
        column: cell.0,
        row: cell.1,
        modifiers: to_key_modifiers(modifiers),
    })
}
