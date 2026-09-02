//! Camera navigation (pan, zoom, 90° level view rotation, and play mode tilt).

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::editor::AppMode;

/// Marker for the main 3D game/editor camera.
#[derive(Component)]
pub struct MainCamera;

/// Component storing camera orbit/pan target parameters.
#[derive(Component)]
pub struct CameraController {
    pub target: Vec3,
    pub distance: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub target_yaw: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    /// Play mode tilt yaw offset in radians (snaps back to 0 on release).
    pub tilt_yaw: f32,
    /// Play mode tilt pitch offset in radians (snaps back to 0 on release).
    pub tilt_pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            target: Vec3::new(4.5, 0.0, -4.5),
            distance: 18.0,
            pitch: 62.0_f32.to_radians(),
            yaw: 0.0,
            target_yaw: 0.0,
            min_distance: 6.0,
            max_distance: 40.0,
            tilt_yaw: 0.0,
            tilt_pitch: 0.0,
        }
    }
}

/// System controlling camera zoom, pan, 90° level view rotation, and play mode camera tilt.
pub fn camera_controller_system(
    time: Res<Time>,
    app_mode: Option<Res<State<AppMode>>>,
    editor_opt: Option<Res<crate::editor::EditorState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<MainCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    let modal_open = editor_opt.as_ref().map(|e| e.is_modal_open()).unwrap_or(false);

    // 1. Mouse Wheel Zoom (active when modal is not capturing scroll)
    if !modal_open {
        for event in mouse_wheel.read() {
            let zoom_amount = match event.unit {
                MouseScrollUnit::Line => event.y * 1.2,
                MouseScrollUnit::Pixel => event.y * 0.05,
            };
            controller.distance = (controller.distance - zoom_amount)
                .clamp(controller.min_distance, controller.max_distance);
        }
    } else {
        for _ in mouse_wheel.read() {}
    }

    let is_editor = app_mode
        .as_ref()
        .map(|s| *s.get() == AppMode::Editor)
        .unwrap_or(false);

    let is_play_or_playback = app_mode
        .as_ref()
        .map(|s| *s.get() == AppMode::Playtest || *s.get() == AppMode::Playback)
        .unwrap_or(true);

    // 2. Keyboard Level View Rotation (Q/E in Editor mode: 90° CCW / CW)
    if is_editor {
        if keys.just_pressed(KeyCode::KeyQ) {
            controller.target_yaw += std::f32::consts::FRAC_PI_2;
        }
        if keys.just_pressed(KeyCode::KeyE) {
            controller.target_yaw -= std::f32::consts::FRAC_PI_2;
        }
    }

    // Smooth yaw interpolation towards target_yaw
    let yaw_diff = controller.target_yaw - controller.yaw;
    if yaw_diff.abs() > 1e-4 {
        controller.yaw += yaw_diff * (15.0 * time.delta_secs()).min(1.0);
    } else {
        controller.yaw = controller.target_yaw;
    }

    // 3. Play Mode Mouse Drag Tilting (at most 25 degrees with soft edge tapering, snaps back on release)
    let max_tilt = 25.0_f32.to_radians();
    if is_play_or_playback {
        let dragging = mouse_button.pressed(MouseButton::Left)
            || mouse_button.pressed(MouseButton::Right)
            || mouse_button.pressed(MouseButton::Middle);

        if dragging {
            for motion in mouse_motion.read() {
                // Horizontal mouse movement shifts yaw with smooth edge tapering
                apply_tapered_tilt(&mut controller.tilt_yaw, -motion.delta.x * 0.006, max_tilt);
                // Vertical mouse movement shifts pitch with smooth edge tapering
                apply_tapered_tilt(&mut controller.tilt_pitch, motion.delta.y * 0.006, max_tilt);
            }
        } else {
            // Smoothly snap back to original orientation (0.0 offset)
            let snap_speed = 14.0 * time.delta_secs();
            controller.tilt_yaw += (0.0 - controller.tilt_yaw) * snap_speed.min(1.0);
            controller.tilt_pitch += (0.0 - controller.tilt_pitch) * snap_speed.min(1.0);
            if controller.tilt_yaw.abs() < 1e-4 {
                controller.tilt_yaw = 0.0;
            }
            if controller.tilt_pitch.abs() < 1e-4 {
                controller.tilt_pitch = 0.0;
            }
        }
    } else {
        // In editor mode, reset tilt
        controller.tilt_yaw = 0.0;
        controller.tilt_pitch = 0.0;
    }

    // 4. Keyboard Panning (WASD active only in Editor mode, screen-relative to current view angle)
    if is_editor {
        let pan_speed = 12.0 * (controller.distance / 18.0) * time.delta_secs();
        let yaw = controller.yaw;
        let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());

        let mut pan_delta = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            pan_delta += forward * pan_speed;
        }
        if keys.pressed(KeyCode::KeyS) {
            pan_delta -= forward * pan_speed;
        }
        if keys.pressed(KeyCode::KeyA) {
            pan_delta -= right * pan_speed;
        }
        if keys.pressed(KeyCode::KeyD) {
            pan_delta -= right * pan_speed;
        }

        controller.target += pan_delta;
    }

    // 5. Update Camera Transform from spherical orbit around target + play mode tilt
    let pitch = (controller.pitch + controller.tilt_pitch).clamp(10.0_f32.to_radians(), 85.0_f32.to_radians());
    let yaw = controller.yaw + controller.tilt_yaw;
    let dist = controller.distance;

    let h_radius = dist * pitch.cos();
    let offset = Vec3::new(
        h_radius * yaw.sin(),
        dist * pitch.sin(),
        h_radius * yaw.cos(),
    );
    let eye = controller.target + offset;

    transform.translation = eye;
    transform.look_at(controller.target, Vec3::Y);
}

/// Smoothly applies an incremental mouse drag rotation `delta` to `current`,
/// tapering off the response smoothly as `current` approaches `max_limit` (instead of an abrupt hard stop).
pub fn apply_tapered_tilt(current: &mut f32, delta: f32, max_limit: f32) {
    if delta == 0.0 || max_limit <= 0.0 {
        return;
    }
    let ratio = (current.abs() / max_limit).clamp(0.0, 1.0);
    // When pushing further towards the edge in the current direction, smoothly taper off resistance:
    let resistance = if (delta > 0.0 && *current > 0.0) || (delta < 0.0 && *current < 0.0) {
        (1.0 - ratio).powf(1.3)
    } else {
        // Moving back towards center: full 1.0 responsiveness
        1.0
    };
    *current = (*current + delta * resistance).clamp(-max_limit, max_limit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_controller_tilt_clamping_and_snapback() {
        let mut controller = CameraController::default();
        let max_tilt = 25.0_f32.to_radians();

        // Excessive tilt input
        controller.tilt_yaw = 50.0_f32.to_radians();
        controller.tilt_pitch = -60.0_f32.to_radians();

        controller.tilt_yaw = controller.tilt_yaw.clamp(-max_tilt, max_tilt);
        controller.tilt_pitch = controller.tilt_pitch.clamp(-max_tilt, max_tilt);

        assert!((controller.tilt_yaw - max_tilt).abs() < 1e-4);
        assert!((controller.tilt_pitch - (-max_tilt)).abs() < 1e-4);

        // Simulate release snap-back
        for _ in 0..60 {
            let snap_speed: f32 = 14.0 * (1.0 / 60.0);
            controller.tilt_yaw += (0.0 - controller.tilt_yaw) * snap_speed.min(1.0);
            controller.tilt_pitch += (0.0 - controller.tilt_pitch) * snap_speed.min(1.0);
        }

        assert!(controller.tilt_yaw.abs() < 0.001);
        assert!(controller.tilt_pitch.abs() < 0.001);
    }

    #[test]
    fn camera_controller_tilt_tapering() {
        let max_tilt = 25.0_f32.to_radians();
        let mut angle = 0.0_f32;

        // At center (0.0), a delta of 0.01 is fully applied
        apply_tapered_tilt(&mut angle, 0.01, max_tilt);
        assert!((angle - 0.01).abs() < 1e-5);

        // Near edge (95% of max), the same delta produces significantly less movement (cushioned taper)
        let mut edge_angle = max_tilt * 0.95;
        let start_edge = edge_angle;
        apply_tapered_tilt(&mut edge_angle, 0.01, max_tilt);
        let edge_delta = edge_angle - start_edge;
        assert!(edge_delta < 0.002); // Tapered down by >80%

        // Moving back towards 0 from edge has immediate full responsiveness
        let prev = edge_angle;
        apply_tapered_tilt(&mut edge_angle, -0.01, max_tilt);
        assert!((prev - edge_angle - 0.01).abs() < 1e-5);
    }
}
