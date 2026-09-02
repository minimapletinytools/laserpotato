//! In-game Play HUD header with level title, move counters, and action buttons.

use bevy::prelude::*;
use crate::editor::ui::theme;
use crate::editor::ui::widgets::button::spawn_action_button;
use crate::play_ui::overlay::{ResetButton, ReturnToMenuButton, UndoButton};

#[derive(Component)]
pub struct PlayHudRoot;

#[derive(Component)]
pub struct PlayHudLevelNameText;

#[derive(Component)]
pub struct PlayHudMovesText;

/// Spawns the in-game top HUD bar.
pub fn spawn_play_hud(commands: &mut Commands) {
    commands
        .spawn((
            PlayHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
            Visibility::Hidden,
        ))
        .with_children(|hud| {
            // Level Title & Moves
            hud.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    PlayHudLevelNameText,
                    Text::new("Level 1: First Light"),
                    TextFont::from_font_size(16.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
                left.spawn((
                    PlayHudMovesText,
                    Text::new("Steps: 0"),
                    TextFont::from_font_size(15.0),
                    TextColor(theme::TEXT_ACCENT),
                ));
            });

            // Quick Actions (Undo, Reset, Level Select)
            hud.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|right| {
                spawn_action_button(right, UndoButton, "Undo [Z]", 13.0, theme::BTN_NORMAL);
                spawn_action_button(right, ResetButton, "Reset [R]", 13.0, theme::BTN_NORMAL);
                spawn_action_button(right, ReturnToMenuButton, "Level Select [Esc]", 13.0, theme::BTN_NORMAL);
            });
        });
}
