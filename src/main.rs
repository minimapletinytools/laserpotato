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

/// Marker component for the top status/victory banner.
#[derive(Component)]
pub struct VictoryBanner;

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
                update_victory_ui,
            ),
        )
        .run();
}

/// Create the simulation world from the test level, spawn camera, lights, and HUD.
fn setup_game(mut commands: Commands) {
    // --- simulation -------------------------------------------------------
    let world = level::test_level();
    let engine = turn::TurnEngine::new(world);
    commands.insert_resource(GameState { engine });

    // --- camera (isometric top-down view centered on the larger level) ----
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3.5, 16.0, 5.0)
            .looking_at(Vec3::new(3.5, 0.0, -3.0), Vec3::Y),
    ));

    // --- light ------------------------------------------------------------
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // --- Top Objective & Victory Banner -----------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Percent(15.0),
                right: Val::Percent(15.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.88)),
        ))
        .with_children(|parent| {
            parent.spawn((
                VictoryBanner,
                Text::new("Objective: Direct the laser to strike the Goal Pyramid\n[↑/↓] Forward/Back  |  [←/→] Turn  |  [Z] Undo  |  [R] Reset"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
            ));
        });

    // --- Coordinate Gizmo Legend (bottom left) ---------------------------
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

/// Update victory / objective HUD banner based on level completion status.
fn update_victory_ui(
    game: Res<GameState>,
    mut query: Query<(&mut Text, &mut TextColor), With<VictoryBanner>>,
) {
    for (mut text, mut color) in &mut query {
        if game.engine.is_won {
            text.0 = "★ LEVEL COMPLETE! Laser Struck Goal Pyramid! ★\n[Z] Undo  |  [R] Reset".into();
            color.0 = Color::srgb(0.3, 1.0, 0.7);
        } else {
            text.0 = "Objective: Direct the laser to strike the Goal Pyramid\n[↑/↓] Forward/Back  |  [←/→] Turn  |  [Z] Undo  |  [R] Reset".into();
            color.0 = Color::srgb(0.9, 0.9, 0.95);
        }
    }
}
