//! Camera navigation (pan & zoom) for Level Editor and Playtest modes.

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
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            target: Vec3::new(3.5, 0.0, -2.5),
            distance: 18.0,
            pitch: 62.0_f32.to_radians(),
            min_distance: 6.0,
            max_distance: 40.0,
        }
    }
}

/// System controlling camera zoom and pan.
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

    // 2. Keyboard Panning (WASD active only in Editor mode)
    let is_editor = app_mode.as_ref().map(|s| *s.get() == AppMode::Editor).unwrap_or(true);
    if is_editor {
        let mut pan_delta = Vec3::ZERO;
        let pan_speed = 12.0 * (controller.distance / 18.0) * time.delta_secs();

        if keys.pressed(KeyCode::KeyW) {
            pan_delta.z -= pan_speed;
        }
        if keys.pressed(KeyCode::KeyS) {
            pan_delta.z += pan_speed;
        }
        if keys.pressed(KeyCode::KeyA) {
            pan_delta.x -= pan_speed;
        }
        if keys.pressed(KeyCode::KeyD) {
            pan_delta.x += pan_speed;
        }

        controller.target += pan_delta;
    }

    // 3. Update Camera Transform from spherical orbit around target
    let pitch = controller.pitch;
    let dist = controller.distance;

    let offset = Vec3::new(0.0, dist * pitch.sin(), dist * pitch.cos());
    let eye = controller.target + offset;

    transform.translation = eye;
    transform.look_at(controller.target, Vec3::Y);
}
