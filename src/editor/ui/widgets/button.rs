use bevy::prelude::*;
use crate::editor::ui::theme;

// ---------------------------------------------------------------------------
// Reusable Button Builders
// ---------------------------------------------------------------------------

/// Spawn a standard text button with a component marker.
pub fn spawn_action_button<Marker: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: Marker,
    label: &str,
    font_size: f32,
    bg_color: Color,
) {
    parent
        .spawn((
            marker,
            Button,
            theme::button_node(10.0, 6.0),
            BorderColor::all(theme::BORDER_SUBTLE),
            BackgroundColor(bg_color),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont::from_font_size(font_size),
                TextColor(theme::TEXT_PRIMARY),
            ));
        });
}

/// Spawn a button with an icon prefix and text label.
pub fn spawn_button_with_icon<Marker: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: Marker,
    icon: &str,
    label: &str,
    font_size: f32,
    bg_color: Color,
) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(theme::BORDER_SUBTLE),
            BackgroundColor(bg_color),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(icon),
                TextFont::from_font_size(font_size),
                TextColor(theme::TEXT_GOLD),
            ));
            b.spawn((
                Text::new(label),
                TextFont::from_font_size(font_size),
                TextColor(theme::TEXT_PRIMARY),
            ));
        });
}

/// Spawn a compact icon-only button.
pub fn spawn_icon_button<Marker: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: Marker,
    icon: &str,
    font_size: f32,
    bg_color: Color,
    pad_px: f32,
) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                padding: UiRect::all(Val::Px(pad_px)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(theme::BORDER_SUBTLE),
            BackgroundColor(bg_color),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(icon),
                TextFont::from_font_size(font_size),
                TextColor(theme::TEXT_PRIMARY),
            ));
        });
}
