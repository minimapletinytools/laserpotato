use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Laser Potato UI Theme: Color Palette
// ---------------------------------------------------------------------------

// Backgrounds
pub const BG_DARK: Color = Color::srgb(0.08, 0.09, 0.11);
pub const BG_PANEL: Color = Color::srgba(0.12, 0.13, 0.16, 0.95);
pub const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.94);
pub const BG_CARD: Color = Color::srgba(0.15, 0.17, 0.22, 0.95);
pub const BG_CARD_TRANSPARENT: Color = Color::srgba(0.12, 0.14, 0.18, 0.85);
pub const BG_ROW_ALT: Color = Color::srgba(0.14, 0.16, 0.20, 0.50);
pub const BG_ROW_HOVER: Color = Color::srgba(0.25, 0.35, 0.55, 0.60);
pub const BG_ROW_SELECTED: Color = Color::srgba(0.20, 0.38, 0.70, 0.75);
pub const BG_HOVER: Color = Color::srgba(0.22, 0.24, 0.30, 0.95);
pub const BG_ACTIVE: Color = Color::srgba(0.25, 0.45, 0.85, 0.95);
pub const BG_INPUT: Color = Color::srgba(0.10, 0.12, 0.16, 1.0);
pub const OVERLAY_BACKDROP: Color = Color::srgba(0.04, 0.04, 0.06, 0.75);

// Borders
pub const BORDER_COLOR: Color = Color::srgba(0.25, 0.28, 0.35, 0.80);
pub const BORDER_CARD: Color = Color::srgba(0.30, 0.35, 0.48, 0.80);
pub const BORDER_FOCUS: Color = Color::srgba(0.40, 0.65, 1.00, 0.90);
pub const BORDER_SUBTLE: Color = Color::srgba(0.20, 0.22, 0.28, 0.60);
pub const BORDER_SELECTED: Color = Color::srgb(0.35, 0.60, 1.00);

// Buttons
pub const BTN_NORMAL: Color = Color::srgba(0.18, 0.18, 0.24, 0.90);
pub const BTN_HOVER: Color = Color::srgba(0.30, 0.34, 0.42, 1.0);
pub const BTN_ACTIVE: Color = Color::srgba(0.22, 0.50, 0.85, 1.0);
pub const BTN_DISABLED: Color = Color::srgba(0.12, 0.12, 0.14, 0.50);
pub const BTN_PRESSED: Color = Color::srgba(0.15, 0.17, 0.22, 1.0);
pub const BTN_PRIMARY: Color = Color::srgba(0.20, 0.48, 0.90, 1.0);
pub const BTN_PRIMARY_HOVER: Color = Color::srgba(0.28, 0.58, 1.00, 1.0);
pub const BTN_SUCCESS: Color = Color::srgba(0.18, 0.55, 0.28, 1.0);
pub const BTN_SUCCESS_HOVER: Color = Color::srgba(0.24, 0.68, 0.35, 1.0);
pub const BTN_DANGER: Color = Color::srgba(0.65, 0.20, 0.20, 1.0);
pub const BTN_DANGER_HOVER: Color = Color::srgba(0.80, 0.25, 0.25, 1.0);
pub const BTN_WARNING: Color = Color::srgba(0.70, 0.50, 0.15, 1.0);

// Scrollbar
pub const SCROLLBAR_TRACK: Color = Color::srgba(0.12, 0.14, 0.18, 0.8);
pub const SCROLLBAR_THUMB: Color = Color::srgba(0.40, 0.45, 0.58, 0.9);
pub const SCROLLBAR_THUMB_HOVER: Color = Color::srgba(0.55, 0.62, 0.80, 1.0);

// Typography
pub const TEXT_PRIMARY: Color = Color::srgb(0.92, 0.92, 0.96);
pub const TEXT_SECONDARY: Color = Color::srgb(0.75, 0.78, 0.85);
pub const TEXT_MUTED: Color = Color::srgb(0.60, 0.60, 0.68);
pub const TEXT_DARK: Color = Color::srgb(0.40, 0.42, 0.48);
pub const TEXT_ACCENT: Color = Color::srgb(0.40, 0.70, 1.00);
pub const TEXT_GOLD: Color = Color::srgb(0.95, 0.80, 0.30);
pub const TEXT_CYAN: Color = Color::srgb(0.30, 0.85, 0.90);
pub const TEXT_SUCCESS: Color = Color::srgb(0.40, 0.88, 0.50);
pub const TEXT_DANGER: Color = Color::srgb(0.95, 0.40, 0.40);
pub const TEXT_WARNING: Color = Color::srgb(0.95, 0.75, 0.30);

// ---------------------------------------------------------------------------
// Standard Layout Style Builders
// ---------------------------------------------------------------------------

/// Root full-screen container node style.
pub fn root_layout() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

/// Top action bar node style.
pub fn top_bar_node(height_px: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(height_px),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        border: UiRect::bottom(Val::Px(1.0)),
        ..default()
    }
}

/// Floating modal backdrop node style.
pub fn modal_backdrop_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

/// Floating modal dialog card node style.
pub fn modal_card_node(min_width_px: f32, max_width_px: f32) -> Node {
    Node {
        min_width: Val::Px(min_width_px),
        max_width: Val::Px(max_width_px),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(20.0)),
        row_gap: Val::Px(14.0),
        border: UiRect::all(Val::Px(1.5)),
        ..default()
    }
}

/// Standard action button node style.
pub fn button_node(pad_x_px: f32, pad_y_px: f32) -> Node {
    Node {
        padding: UiRect::axes(Val::Px(pad_x_px), Val::Px(pad_y_px)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        ..default()
    }
}

/// Standard text input field node style.
pub fn input_box_node(min_height_px: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(min_height_px),
        padding: UiRect::all(Val::Px(10.0)),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Standard vertical scrollbar track node style.
pub fn scrollbar_track_node(width_px: f32) -> Node {
    Node {
        width: Val::Px(width_px),
        height: Val::Percent(100.0),
        margin: UiRect::left(Val::Px(4.0)),
        border_radius: BorderRadius::all(Val::Px(3.0)),
        ..default()
    }
}

/// Standard vertical scrollbar thumb node style.
pub fn scrollbar_thumb_node(width_px: f32, height_px: f32, top_px: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(top_px),
        left: Val::Px(0.0),
        width: Val::Px(width_px),
        height: Val::Px(height_px),
        border_radius: BorderRadius::all(Val::Px(3.0)),
        ..default()
    }
}

/// Standard small badge tag style.
pub fn badge_node(pad_x: f32, pad_y: f32) -> Node {
    Node {
        padding: UiRect::axes(Val::Px(pad_x), Val::Px(pad_y)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(Val::Px(3.0)),
        ..default()
    }
}
