//! Play UI, HUD, Victory/GameOver overlays, and Shipped Level Browser screen.

pub mod banner;
pub mod catalog;
pub mod hud;
pub mod interactions;
pub mod level_select;
pub mod overlay;

pub use banner::*;
pub use catalog::*;
pub use hud::*;
pub use interactions::*;
pub use level_select::*;
pub use overlay::*;

use bevy::prelude::*;

/// Standalone Play UI Plugin managing the level browser and gameplay HUD.
pub struct PlayUiPlugin;

impl Plugin for PlayUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayMode>()
            .init_resource::<LevelCatalog>()
            .add_systems(
                Startup,
                setup_play_ui,
            )
            .add_systems(
                Update,
                (
                    browser_button_system.run_if(in_state(PlayMode::LevelSelect)),
                    hud_button_system,
                    update_hud_system,
                    mouse_wheel_scroll_system.run_if(in_state(PlayMode::LevelSelect)),
                    update_browser_dynamic_list_system.run_if(in_state(PlayMode::LevelSelect)),
                    update_browser_detail_card_system.run_if(in_state(PlayMode::LevelSelect)),
                    level_select_shortcuts_system.run_if(in_state(PlayMode::LevelSelect)),
                ),
            )
            .add_systems(
                Update,
                gameplay_shortcuts_system.run_if(in_state(PlayMode::Playing)),
            );
    }
}

/// Sets up the complete UI hierarchy for standalone Play mode.
pub fn setup_play_ui(mut commands: Commands, catalog: Res<LevelCatalog>) {
    spawn_level_select_screen(&mut commands, &catalog);
    spawn_play_hud(&mut commands);
    spawn_victory_overlay(&mut commands);
    spawn_game_over_overlay(&mut commands);
}
