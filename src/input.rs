//! Keyboard → [`PlayerAction`](crate::turn::PlayerAction) translation.
//!
//! Instantaneous direct key translation to the turn engine in Playtest mode.

use bevy::prelude::*;

use crate::turn::PlayerAction;
use crate::GameState;

/// Reads keyboard input and applies the corresponding [`PlayerAction`] directly with zero latency.
///
/// Controls:
/// - **↑ / W** — move forward (in facing direction)
/// - **↓ / S** — move backward (opposite facing direction)
/// - **← / A / Q** — turn 90° left (counter-clockwise)
/// - **→ / D / E** — turn 90° right (clockwise)
/// - **Space** — wait / interact
/// - **Z / U** — undo
/// - **R** — reset puzzle
pub fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<GameState>,
) {
    let mut action = None;

    // Forward / backward
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        action = Some(PlayerAction::Forward);
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        action = Some(PlayerAction::Backward);
    }
    // Turn in place
    else if keys.just_pressed(KeyCode::ArrowLeft)
        || keys.just_pressed(KeyCode::KeyA)
        || keys.just_pressed(KeyCode::KeyQ)
    {
        action = Some(PlayerAction::TurnLeft);
    } else if keys.just_pressed(KeyCode::ArrowRight)
        || keys.just_pressed(KeyCode::KeyD)
        || keys.just_pressed(KeyCode::KeyE)
    {
        action = Some(PlayerAction::TurnRight);
    }
    // Actions
    else if keys.just_pressed(KeyCode::Space) {
        action = Some(PlayerAction::Wait);
    }
    // Meta
    else if keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::KeyU) {
        action = Some(PlayerAction::Undo);
    } else if keys.just_pressed(KeyCode::KeyR) {
        action = Some(PlayerAction::Reset);
    }

    if let Some(act) = action {
        game.engine.apply(act);
    }
}
