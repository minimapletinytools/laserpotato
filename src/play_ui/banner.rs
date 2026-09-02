//! Victory, objective, and solution playback HUD banners.

use bevy::prelude::*;
use crate::editor::AppMode;
use crate::playback::PlaybackState;
use crate::turn::GameOutcome;
use crate::GameState;

/// Marker component for the playtest/playback banner.
#[derive(Component)]
pub struct VictoryBanner;

/// Marker for the text inside the playtest/playback victory banner.
#[derive(Component)]
pub struct VictoryBannerText;

/// Spawns the top status banner for Playtest and Playback modes.
pub fn spawn_victory_banner(commands: &mut Commands) {
    commands
        .spawn((
            VictoryBanner,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Percent(15.0),
                right: Val::Percent(15.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.88)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                VictoryBannerText,
                Text::new("Objective: Direct the laser to strike the Goal Pyramid\n[W/S / Up/Down] Move  |  [A/D / Left/Right] Turn  |  [Esc] Return  |  [Z] Undo  |  [R] Reset"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
            ));
        });
}

/// Update victory / objective / playback HUD banner during Playtest and Playback modes.
pub fn update_victory_ui(
    app_mode: Res<State<AppMode>>,
    playback: Res<PlaybackState>,
    game: Res<GameState>,
    mut banner_query: Query<&mut Visibility, With<VictoryBanner>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<VictoryBannerText>>,
) {
    let mode = *app_mode.get();
    let is_active_mode = mode == AppMode::Playtest || mode == AppMode::Playback;

    for mut vis in &mut banner_query {
        *vis = if is_active_mode {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !is_active_mode {
        return;
    }

    for (mut text, mut color) in &mut text_query {
        if mode == AppMode::Playback {
            if game.engine.is_won() {
                text.0 = format!(
                    "*** PLAYBACK COMPLETE: Goal Struck in {} Steps! ***\n[Esc] Return  |  [R] Replay  |  [< / >] Step  |  [- / +] Speed ({:.1}x)",
                    playback.current_index,
                    playback.speed
                );
                color.0 = Color::srgb(0.3, 1.0, 0.7);
            } else if game.engine.is_lost() {
                text.0 = format!(
                    "!!! PLAYBACK: Laser Vaporized Player at Step {} !!!\n[Esc] Return  |  [R] Restart  |  [<] Step Back  |  [- / +] Speed ({:.1}x)",
                    playback.current_index,
                    playback.speed
                );
                color.0 = Color::srgb(1.0, 0.3, 0.3);
            } else {
                let status_label = if playback.auto_playing { "[Playing]" } else { "[Paused]" };
                let next_action_str = if playback.current_index < playback.actions.len() {
                    format!("Next: {:?}", playback.actions[playback.current_index])
                } else {
                    "End of sequence".into()
                };

                text.0 = format!(
                    "TESTING WITH SOLUTION ({:.1}x speed) - {} Step {} / {} ({})\n[Space] Play/Pause  |  [< / >] Step  |  [- / +] Speed  |  [Esc] Return  |  [R] Restart",
                    playback.speed,
                    status_label,
                    playback.current_index,
                    playback.actions.len(),
                    next_action_str
                );
                color.0 = Color::srgb(0.3, 1.0, 0.6);
            }
        } else if mode == AppMode::Playtest {
            if let Some(err) = &game.engine.validation_error {
                text.0 = format!("! INVALID LEVEL: {}\n[Esc] Return", err);
                color.0 = Color::srgb(1.0, 0.35, 0.35);
            } else {
                match game.engine.outcome {
                    GameOutcome::Won => {
                        text.0 = "*** LEVEL COMPLETE! Laser Struck Goal Pyramid! ***\n[Esc] Return  |  [Z] Undo  |  [R] Reset".into();
                        color.0 = Color::srgb(0.3, 1.0, 0.7);
                    }
                    GameOutcome::Lost => {
                        text.0 = "!!! GAME OVER! Laser Vaporized Player! !!!\n[Esc] Return  |  [Z] Undo  |  [R] Reset".into();
                        color.0 = Color::srgb(1.0, 0.3, 0.3);
                    }
                    GameOutcome::InProgress => {
                        text.0 = "PLAYTEST MODE: Direct laser to Goal Pyramid\n[W/S / Up/Down] Move  |  [A/D / Left/Right] Turn  |  [Esc] Return  |  [Z] Undo  |  [R] Reset".into();
                        color.0 = Color::srgb(0.9, 0.95, 1.0);
                    }
                }
            }
        }
    }
}
