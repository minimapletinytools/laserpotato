use bevy::prelude::*;
use crate::editor::ui::theme;
use crate::editor::ui::widgets::button::spawn_action_button;

// ---------------------------------------------------------------------------
// Reusable Modal Dialog Builder
// ---------------------------------------------------------------------------

/// Spawn a full-screen semi-transparent backdrop containing a centered floating dialog card.
pub fn spawn_modal_backdrop<ModalMarker: Component, F: FnOnce(&mut ChildSpawnerCommands)>(
    parent: &mut ChildSpawnerCommands,
    marker: ModalMarker,
    min_width_px: f32,
    max_width_px: f32,
    build_card: F,
) {
    parent
        .spawn((
            marker,
            theme::modal_backdrop_node(),
            BackgroundColor(theme::OVERLAY_BACKDROP),
            Visibility::Hidden,
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    theme::modal_card_node(min_width_px, max_width_px),
                    BorderColor::all(theme::BORDER_CARD),
                    BackgroundColor(theme::BG_CARD),
                ))
                .with_children(build_card);
        });
}

/// Spawn a standard modal header with title and optional subtitle text.
pub fn spawn_modal_header(
    card: &mut ChildSpawnerCommands,
    title: &str,
    subtitle: Option<&str>,
) {
    card.spawn((
        Text::new(title),
        TextFont::from_font_size(15.0),
        TextColor(theme::TEXT_PRIMARY),
    ));

    if let Some(sub) = subtitle {
        card.spawn((
            Text::new(sub),
            TextFont::from_font_size(12.0),
            TextColor(theme::TEXT_MUTED),
        ));
    }
}

/// Spawn a modal footer button container aligned to the right.
pub fn spawn_modal_footer<F: FnOnce(&mut ChildSpawnerCommands)>(
    card: &mut ChildSpawnerCommands,
    build_buttons: F,
) {
    card.spawn(Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::FlexEnd,
        column_gap: Val::Px(8.0),
        margin: UiRect::top(Val::Px(6.0)),
        ..default()
    })
    .with_children(build_buttons);
}

/// Spawn standard Cancel and Confirm action buttons in a modal footer.
pub fn spawn_modal_action_buttons<CancelM: Component, ConfirmM: Component>(
    footer: &mut ChildSpawnerCommands,
    cancel_marker: CancelM,
    confirm_marker: ConfirmM,
    cancel_label: &str,
    confirm_label: &str,
    confirm_bg: Color,
) {
    spawn_action_button(footer, cancel_marker, cancel_label, 11.0, theme::BTN_NORMAL);
    spawn_action_button(footer, confirm_marker, confirm_label, 11.0, confirm_bg);
}
