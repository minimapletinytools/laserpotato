use bevy::prelude::*;

mod block_types;
mod input;
mod laser;
mod level;
mod render;
mod sim;
mod turn;

/// Bevy resource wrapping the pure-logic [`TurnEngine`](turn::TurnEngine).
#[derive(Resource)]
pub struct GameState {
    pub engine: turn::TurnEngine,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Laser Potato".into(),
                resolution: (1024, 768).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<input::PendingAction>()
        .add_systems(Startup, (setup_game, render::setup_render_assets))
        .add_systems(
            Update,
            (
                input::keyboard_input_system,
                apply_action.after(input::keyboard_input_system),
                render::sync_bodies.after(apply_action),
                render::sync_lasers.after(apply_action),
                render::animate_laser_pfx,
                render::draw_coordinate_gizmo,
            ),
        )
        .run();
}

/// Create the simulation world from the test level, spawn camera and lights.
fn setup_game(mut commands: Commands) {
    // --- simulation -------------------------------------------------------
    let world = level::test_level();
    let engine = turn::TurnEngine::new(world);
    commands.insert_resource(GameState { engine });

    // --- camera (top-down-ish isometric view) ------------------------------
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 12.0, 4.5)
            .looking_at(Vec3::new(2.0, 0.0, -1.5), Vec3::Y),
    ));

    // --- light ------------------------------------------------------------
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // --- coordinate gizmo legend overlay (bottom left) -------------------
    if render::SHOW_COORDINATE_LEGEND {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Game Axes (RHS)\n+X: Right (Red)\n+Y: Forward (Green)\n+Z: Up (Blue)"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.9, 0.9, 0.95)),
                ));
            });
    }
}

/// Bridge between the Bevy input system and the pure-logic turn engine.
fn apply_action(pending: Res<input::PendingAction>, mut game: ResMut<GameState>) {
    if let Some(action) = pending.0 {
        game.engine.apply(action);
    }
}
