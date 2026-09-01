//! Laser Potato — pure logic simulation, puzzle engine, and solver library.

use std::time::Duration;

use bevy::prelude::*;

pub mod block_types;
pub mod camera;
pub mod editor;
pub mod generator;
pub mod input;
pub mod laser;
pub mod level;
pub mod render;
pub mod sim;
pub mod solver;
pub mod turn;

/// Bevy resource wrapping the pure-logic [`TurnEngine`](turn::TurnEngine).
#[derive(Resource)]
pub struct GameState {
    pub engine: turn::TurnEngine,
}

/// Resource controlling solution replay/playback mode.
#[derive(Resource, Clone, Debug)]
pub struct PlaybackState {
    pub is_playback: bool,
    pub actions: Vec<turn::PlayerAction>,
    pub current_index: usize,
    pub auto_playing: bool,
    pub speed: f32,
    pub step_timer: Timer,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playback: false,
            actions: Vec::new(),
            current_index: 0,
            auto_playing: true,
            speed: 1.0,
            step_timer: Timer::new(Duration::from_millis(400), TimerMode::Repeating),
        }
    }
}
