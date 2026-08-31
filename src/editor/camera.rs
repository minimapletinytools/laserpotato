//! Camera navigation (pan, zoom, and 90° level view rotation) for Level Editor and Playtest modes.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use super::AppMode;

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
        }
    }
}

/// System controlling camera zoom, pan, and 90° level view rotation.
pub fn camera_controller_system(
    time: Res<Time>,
    app_mode: Option<Res<State<AppMode>>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut CameraController), With<MainCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    // 1. Mouse Wheel Zoom (active in all modes)
    for event in mouse_wheel.read() {
        let zoom_amount = match event.unit {
            MouseScrollUnit::Line => event.y * 1.2,
            MouseScrollUnit::Pixel => event.y * 0.05,
        };
        controller.distance = (controller.distance - zoom_amount)
            .clamp(controller.min_distance, controller.max_distance);
    }

    // 2. Keyboard Level View Rotation (Q/E in Editor mode: 90° CCW / CW)
    let is_editor = app_mode.as_ref().map(|s| *s.get() == AppMode::Editor).unwrap_or(true);
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

    // 3. Keyboard Panning (WASD active only in Editor mode, screen-relative to current view angle)
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
            pan_delta += right * pan_speed;
        }

        controller.target += pan_delta;
    }

    // 4. Update Camera Transform from spherical orbit around target
    let pitch = controller.pitch;
    let yaw = controller.yaw;
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
