//! Solution playback state and execution engine.
//!
//! Handles step-by-step and automated replay of recorded puzzle solutions
//! at adjustable playback speeds.

use std::time::Duration;
use bevy::prelude::*;
use crate::turn::{PlayerAction, TurnEngine};
use crate::GameState;

/// Resource controlling solution replay/playback mode.
#[derive(Resource, Clone, Debug)]
pub struct PlaybackState {
    pub is_playback: bool,
    pub actions: Vec<PlayerAction>,
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

impl PlaybackState {
    pub fn new(actions: Vec<PlayerAction>) -> Self {
        Self {
            is_playback: true,
            actions,
            current_index: 0,
            auto_playing: true,
            speed: 1.0,
            step_timer: Timer::new(Duration::from_millis(400), TimerMode::Repeating),
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        let clamped = speed.clamp(0.2, 10.0);
        self.speed = clamped;
        self.step_timer.set_duration(Duration::from_secs_f32(0.40 / clamped));
    }

    pub fn step_forward(&mut self, engine: &mut TurnEngine) -> bool {
        if self.current_index < self.actions.len() {
            let action = self.actions[self.current_index];
            engine.apply(action);
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn step_backward(&mut self, engine: &mut TurnEngine) -> bool {
        if self.current_index > 0 {
            engine.apply(PlayerAction::Undo);
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn restart(&mut self, engine: &mut TurnEngine) {
        engine.apply(PlayerAction::Reset);
        self.current_index = 0;
        self.auto_playing = true;
    }
}

/// System for executing automated and manual step-by-step playback of a loaded solution.
pub fn playback_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<PlaybackState>,
    mut game: ResMut<GameState>,
) {
    if !playback.is_playback {
        return;
    }

    // Toggle Play / Pause
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyP) {
        playback.auto_playing = !playback.auto_playing;
    }

    // Speed Controls: - / _ or [ decreases speed, + / = or ] increases speed
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) || keys.just_pressed(KeyCode::BracketLeft) {
        let new_speed = (playback.speed * 0.75).clamp(0.2, 10.0);
        playback.set_speed(new_speed);
    } else if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) || keys.just_pressed(KeyCode::BracketRight) {
        let new_speed = (playback.speed * 1.5).clamp(0.2, 10.0);
        playback.set_speed(new_speed);
    }

    // Manual Step Forward
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Period) {
        playback.step_forward(&mut game.engine);
    }

    // Manual Step Backward (Undo)
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::Comma) {
        playback.step_backward(&mut game.engine);
    }

    // Restart Playback
    if keys.just_pressed(KeyCode::KeyR) {
        playback.restart(&mut game.engine);
    }

    // Automatic Step Progression
    if playback.auto_playing && !game.engine.outcome.is_game_over() {
        playback.step_timer.tick(time.delta());
        if playback.step_timer.just_finished() {
            playback.step_forward(&mut game.engine);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::World;

    #[test]
    fn playback_step_and_speed_controls() {
        let mut state = PlaybackState::new(vec![PlayerAction::MoveNorth, PlayerAction::MoveEast]);
        assert_eq!(state.speed, 1.0);
        assert_eq!(state.current_index, 0);

        state.set_speed(2.0);
        assert_eq!(state.speed, 2.0);

        let mut engine = TurnEngine::new(World::new());
        assert!(state.step_forward(&mut engine));
        assert_eq!(state.current_index, 1);

        assert!(state.step_forward(&mut engine));
        assert_eq!(state.current_index, 2);

        // No more steps
        assert!(!state.step_forward(&mut engine));
        assert_eq!(state.current_index, 2);

        // Step back
        assert!(state.step_backward(&mut engine));
        assert_eq!(state.current_index, 1);

        // Restart
        state.restart(&mut engine);
        assert_eq!(state.current_index, 0);
        assert!(state.auto_playing);
    }
}
