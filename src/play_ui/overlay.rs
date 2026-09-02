//! Modal overlay dialogs for Victory and Game Over screens.

use bevy::prelude::*;
use crate::editor::ui::theme;
use crate::editor::ui::widgets::button::spawn_action_button;

#[derive(Component)]
pub struct VictoryOverlayRoot;

#[derive(Component)]
pub struct VictoryOverlayText;

#[derive(Component)]
pub struct GameOverOverlayRoot;

#[derive(Component)]
pub struct GameOverOverlayText;

#[derive(Component)]
pub struct NextLevelButton;

#[derive(Component)]
pub struct ReplayButton;

#[derive(Component)]
pub struct ReturnToMenuButton;

#[derive(Component)]
pub struct UndoButton;

#[derive(Component)]
pub struct ResetButton;

/// Spawns the modal victory overlay card.
pub fn spawn_victory_overlay(commands: &mut Commands) {
    commands
        .spawn((
            VictoryOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(28.0),
                left: Val::Percent(25.0),
                right: Val::Percent(25.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.08, 0.96)),
            BorderColor::all(theme::TEXT_SUCCESS),
            Visibility::Hidden,
        ))
        .with_children(|modal| {
            modal.spawn((
                Text::new("*** LEVEL COMPLETE! ***"),
                TextFont::from_font_size(28.0),
                TextColor(theme::TEXT_SUCCESS),
            ));

            modal.spawn((
                VictoryOverlayText,
                Text::new("Goal pyramid struck by laser!"),
                TextFont::from_font_size(16.0),
                TextColor(theme::TEXT_PRIMARY),
            ));

            modal.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|actions| {
                spawn_action_button(actions, NextLevelButton, "Next Level >", 15.0, theme::BTN_SUCCESS);
                spawn_action_button(actions, ReplayButton, "Replay [R]", 15.0, theme::BTN_NORMAL);
                spawn_action_button(actions, ReturnToMenuButton, "Level Select [Esc]", 15.0, theme::BTN_NORMAL);
            });
        });
}

/// Spawns the modal game over overlay card.
pub fn spawn_game_over_overlay(commands: &mut Commands) {
    commands
        .spawn((
            GameOverOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(28.0),
                left: Val::Percent(25.0),
                right: Val::Percent(25.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.04, 0.04, 0.96)),
            BorderColor::all(theme::TEXT_DANGER),
            Visibility::Hidden,
        ))
        .with_children(|modal| {
            modal.spawn((
                Text::new("! LASER VAPORIZED PLAYER !"),
                TextFont::from_font_size(26.0),
                TextColor(theme::TEXT_DANGER),
            ));

            modal.spawn((
                GameOverOverlayText,
                Text::new("Player walked into the laser beam."),
                TextFont::from_font_size(15.0),
                TextColor(theme::TEXT_PRIMARY),
            ));

            modal.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|actions| {
                spawn_action_button(actions, UndoButton, "Undo Move [Z]", 15.0, theme::BTN_DANGER);
                spawn_action_button(actions, ResetButton, "Restart [R]", 15.0, theme::BTN_NORMAL);
                spawn_action_button(actions, ReturnToMenuButton, "Level Select [Esc]", 15.0, theme::BTN_NORMAL);
            });
        });
}
