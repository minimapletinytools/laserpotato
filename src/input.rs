//! Keyboard → [`PlayerAction`](crate::turn::PlayerAction) translation.
//!
//! This is the **only** Bevy system that writes into the simulation's input
//! channel. It produces at most one action per frame.

use bevy::prelude::*;

use crate::turn::PlayerAction;

/// Holds the pending player action for this frame, if any.
#[derive(Resource, Default)]
pub struct PendingAction(pub Option<PlayerAction>);

/// Reads keyboard input and writes the corresponding [`PlayerAction`].
///
/// Key mapping (from design doc):
/// - **↑ / W** — move forward (in facing direction)
/// - **↓ / S** — move backward (opposite facing direction)
/// - **← / Q** — turn 90° left (counter-clockwise)
/// - **→ / E** — turn 90° right (clockwise)
/// - **Space** — interact
/// - **A** — wait (counts as a turn)
/// - **Z** — undo
/// - **R** — reset puzzle
pub fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingAction>,
) {
    pending.0 = None;

    // Forward / backward
    if keys.just_pressed(KeyCode::ArrowUp) {
        pending.0 = Some(PlayerAction::Forward);
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        pending.0 = Some(PlayerAction::Backward);
    }
    // Turn in place
    else if keys.just_pressed(KeyCode::ArrowLeft) {
        pending.0 = Some(PlayerAction::TurnLeft);
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        pending.0 = Some(PlayerAction::TurnRight);
    }
    // Actions
    else if keys.just_pressed(KeyCode::Space) {
        pending.0 = Some(PlayerAction::Interact);
    } else if keys.just_pressed(KeyCode::KeyA) {
        pending.0 = Some(PlayerAction::Wait);
    }
    // Meta
    else if keys.just_pressed(KeyCode::KeyZ) {
        pending.0 = Some(PlayerAction::Undo);
    } else if keys.just_pressed(KeyCode::KeyR) {
        pending.0 = Some(PlayerAction::Reset);
    }
}
