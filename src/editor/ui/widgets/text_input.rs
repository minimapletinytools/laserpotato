use bevy::prelude::*;
use crate::editor::ui::theme;

// ---------------------------------------------------------------------------
// Keyboard Character Mapping
// ---------------------------------------------------------------------------

/// Convert a Bevy KeyCode into an alphanumeric or punctuation character.
pub fn key_code_to_char(key: KeyCode, shift: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift { '+' } else { '=' }),
        KeyCode::Period => Some(if shift { '>' } else { '.' }),
        KeyCode::Comma => Some(if shift { '<' } else { ',' }),
        KeyCode::Slash => Some(if shift { '?' } else { '/' }),
        KeyCode::Semicolon => Some(if shift { ':' } else { ';' }),
        KeyCode::Quote => Some(if shift { '"' } else { '\'' }),
        _ => None,
    }
}

/// Handle a key press on a string buffer (supports typing, backspace, and max length).
/// Returns true if the buffer was modified.
pub fn handle_text_input_key(
    buffer: &mut String,
    key: KeyCode,
    shift: bool,
    max_len: Option<usize>,
) -> bool {
    if key == KeyCode::Backspace {
        if !buffer.is_empty() {
            buffer.pop();
            return true;
        }
        return false;
    }

    if let Some(c) = key_code_to_char(key, shift) {
        if let Some(limit) = max_len {
            if buffer.len() >= limit {
                return false;
            }
        }
        buffer.push(c);
        return true;
    }

    false
}

/// Format a buffer with an active cursor indicator if focused.
pub fn format_input_with_cursor(buffer: &str, focused: bool) -> String {
    if focused {
        format!("{}_", buffer)
    } else {
        buffer.to_string()
    }
}

// ---------------------------------------------------------------------------
// Text Input UI Builder
// ---------------------------------------------------------------------------

/// Spawn a text input box container with inner Text display node.
pub fn spawn_text_input_box<TextMarker: Component>(
    parent: &mut ChildSpawnerCommands,
    text_marker: TextMarker,
    initial_text: &str,
    min_height_px: f32,
    font_size: f32,
) {
    parent
        .spawn((
            theme::input_box_node(min_height_px),
            BorderColor::all(theme::BORDER_CARD),
            BackgroundColor(theme::BG_INPUT),
        ))
        .with_children(|box_node| {
            box_node.spawn((
                text_marker,
                Text::new(initial_text),
                TextFont::from_font_size(font_size),
                TextColor(theme::TEXT_PRIMARY),
            ));
        });
}
