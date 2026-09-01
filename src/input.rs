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
    mut editor: ResMut<crate::editor::EditorState>,
) {
    let mut action = None;

    // Directional movement: ↑ / W (North), ↓ / S (South), ← / A (West), → / D (East)
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        action = Some(PlayerAction::MoveNorth);
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        action = Some(PlayerAction::MoveSouth);
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        action = Some(PlayerAction::MoveWest);
    } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        action = Some(PlayerAction::MoveEast);
    }
    // Dedicated turn-in-place keys: Q (turn left) and E (turn right)
    else if keys.just_pressed(KeyCode::KeyQ) {
        action = Some(PlayerAction::TurnLeft);
    } else if keys.just_pressed(KeyCode::KeyE) {
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
        if game.engine.is_won() && !editor.playtest_win_recorded {
            if !game.engine.action_history.is_empty() {
                let actions = game.engine.action_history.clone();
                let name = format!("Player Play #{} ({} steps)", editor.solutions.len() + 1, actions.len());
                if !editor.solutions.iter().any(|s| s.actions == actions) {
                    editor.solutions.push(crate::level::LevelSolution {
                        name,
                        actions,
                    });
                    editor.toast(format!("Level Solved! Solution recorded ({} steps).", game.engine.action_history.len()));
                }
            }
            editor.playtest_win_recorded = true;
        } else if !game.engine.is_won() {
            editor.playtest_win_recorded = false;
        }
    }
}
