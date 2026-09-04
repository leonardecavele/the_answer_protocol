use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use eframe::egui::{Context, Event, Key, Modifiers};

pub fn terminal_events(ctx: &Context) -> Vec<CrosstermEvent> {
    let mut events = Vec::new();

    ctx.input(|input| {
        for event in &input.events {
            push_translated(event, &mut events);
        }
    });

    events
}

fn push_translated(event: &Event, events: &mut Vec<CrosstermEvent>) {
    match event {
        Event::Text(text) => {
            for character in text.chars() {
                events.push(key_event(KeyCode::Char(character), KeyModifiers::NONE));
            }
        }
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => {
            if let Some(code) = key_code(*key, *modifiers) {
                events.push(key_event(code, key_modifiers(*modifiers)));
            }
        }
        Event::Copy => events.push(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        _ => {}
    }
}

fn key_code(key: Key, modifiers: Modifiers) -> Option<KeyCode> {
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
        _ if modifiers.ctrl => KeyCode::Char(control_letter(key)?),
        _ => return None,
    };

    Some(code)
}

fn control_letter(key: Key) -> Option<char> {
    let mut characters = key.name().chars();

    match (characters.next(), characters.next()) {
        (Some(letter), None) if letter.is_ascii_alphabetic() => Some(letter.to_ascii_lowercase()),
        _ => None,
    }
}

fn key_modifiers(modifiers: Modifiers) -> KeyModifiers {
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

fn key_event(code: KeyCode, modifiers: KeyModifiers) -> CrosstermEvent {
    CrosstermEvent::Key(KeyEvent::new(code, modifiers))
}
