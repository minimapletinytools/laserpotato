//! Laser Potato — pure logic simulation, puzzle engine, and solver library.

use bevy::prelude::*;

pub mod block_types;
pub mod camera;
pub mod editor;
pub mod generator;
pub mod input;
pub mod laser;
pub mod level;
pub mod playback;
pub mod play_ui;
pub mod render;
pub mod sim;
pub mod solver;
pub mod turn;

/// Global reusable UI theme and widget primitives.
pub mod ui {
    pub use crate::editor::ui::theme;
    pub use crate::editor::ui::widgets;
}

pub use playback::PlaybackState;

/// Bevy resource wrapping the pure-logic [`TurnEngine`](turn::TurnEngine).
#[derive(Resource)]
pub struct GameState {
    pub engine: turn::TurnEngine,
}
