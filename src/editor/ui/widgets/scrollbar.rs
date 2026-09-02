use bevy::prelude::*;
use crate::editor::ui::theme;

// ---------------------------------------------------------------------------
// Scrollbar Calculation Helpers
// ---------------------------------------------------------------------------

/// Calculate the height and vertical position of the scrollbar thumb in pixels.
pub fn calculate_thumb_layout(
    visible_count: usize,
    total_count: usize,
    scroll_offset: usize,
    track_height: f32,
) -> (f32, f32) {
    if total_count <= visible_count || total_count == 0 {
        return (track_height, 0.0);
    }

    let thumb_ratio = (visible_count as f32 / total_count as f32).clamp(0.08, 1.0);
    let thumb_height = (track_height * thumb_ratio).max(20.0);
    let max_scroll = (total_count - visible_count) as f32;
    let scroll_ratio = if max_scroll > 0.0 {
        (scroll_offset as f32 / max_scroll).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let thumb_top = scroll_ratio * (track_height - thumb_height);

    (thumb_height, thumb_top)
}

/// Calculate the updated scroll offset given a cursor vertical drag delta in pixels.
pub fn calculate_drag_scroll_offset(
    drag_start_offset: usize,
    cursor_delta_y: f32,
    track_height: f32,
    visible_count: usize,
    total_count: usize,
) -> usize {
    if total_count <= visible_count || track_height <= 0.0 {
        return 0;
    }

    let max_scroll = total_count - visible_count;
    let thumb_ratio = (visible_count as f32 / total_count as f32).clamp(0.08, 1.0);
    let thumb_height = (track_height * thumb_ratio).max(20.0);
    let scrollable_pixels = (track_height - thumb_height).max(1.0);

    let delta_ratio = cursor_delta_y / scrollable_pixels;
    let delta_items = (delta_ratio * max_scroll as f32).round() as i32;
    (drag_start_offset as i32 + delta_items).clamp(0, max_scroll as i32) as usize
}

/// Calculate the target scroll offset when clicking directly on a track coordinate.
pub fn calculate_click_jump_offset(
    click_ratio: f32,
    visible_count: usize,
    total_count: usize,
) -> usize {
    if total_count <= visible_count {
        return 0;
    }
    let max_scroll = total_count - visible_count;
    let target = (click_ratio * max_scroll as f32).round() as usize;
    target.min(max_scroll)
}

// ---------------------------------------------------------------------------
// Scrollbar UI Builder
// ---------------------------------------------------------------------------

/// Spawn a vertical scrollbar track and thumb into a parent UI container.
pub fn spawn_scrollbar_track<TrackMarker: Component, ThumbMarker: Component>(
    parent: &mut ChildSpawnerCommands,
    track_width_px: f32,
    track_marker: TrackMarker,
    thumb_marker: ThumbMarker,
) {
    parent
        .spawn((
            track_marker,
            Button,
            theme::scrollbar_track_node(track_width_px),
            BackgroundColor(theme::SCROLLBAR_TRACK),
        ))
        .with_children(|track| {
            track.spawn((
                thumb_marker,
                theme::scrollbar_thumb_node(track_width_px, 30.0, 0.0),
                BackgroundColor(theme::SCROLLBAR_THUMB),
            ));
        });
}
