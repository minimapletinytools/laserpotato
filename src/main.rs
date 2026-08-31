use std::env;
use std::time::Duration;

use bevy::prelude::*;

mod editor;
mod input;
mod render;

pub use laserpotato::{block_types, laser, level, sim, solver, turn};

/// Bevy resource wrapping the pure-logic [`TurnEngine`](turn::TurnEngine).
#[derive(Resource)]
pub struct GameState {
    pub engine: turn::TurnEngine,
}

/// Resource controlling solution replay/playback mode.
#[derive(Resource)]
pub struct PlaybackState {
    pub is_playback: bool,
    pub actions: Vec<turn::PlayerAction>,
    pub current_index: usize,
    pub auto_playing: bool,
    pub step_timer: Timer,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playback: false,
            actions: Vec::new(),
            current_index: 0,
            auto_playing: true,
            step_timer: Timer::new(Duration::from_millis(400), TimerMode::Repeating),
        }
    }
}

/// Marker component for the playtest/playback banner.
#[derive(Component)]
pub struct VictoryBanner;

/// Marker for the text inside the playtest/playback victory banner.
#[derive(Component)]
pub struct VictoryBannerText;

/// Configuration for automated screenshot capture and self-render testing.
#[derive(Resource)]
pub struct ScreenshotConfig {
    pub output_path: Option<String>,
    pub frame_counter: u32,
    pub target_frame: u32,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            output_path: None,
            frame_counter: 0,
            target_frame: 20,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut replay_file = None;
    let mut screenshot_path = None;
    let mut target_frame = 20;
    let mut initial_mode = editor::AppMode::Editor;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--replay" | "--playback" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    replay_file = Some(args[i].clone());
                } else {
                    replay_file = Some(String::from("solution.json"));
                }
            }
            "-s" | "--screenshot" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    screenshot_path = Some(args[i].clone());
                } else {
                    screenshot_path = Some(String::from("screenshot.png"));
                }
            }
            "--frames" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(n) = args[i].parse::<u32>() {
                        target_frame = n;
                    }
                }
            }
            "--playtest" => {
                initial_mode = editor::AppMode::Playtest;
            }
            "--editor" => {
                initial_mode = editor::AppMode::Editor;
            }
            _ => {}
        }
        i += 1;
    }

    let mut playback = PlaybackState::default();

    if let Some(file_path) = replay_file {
        match solver::load_actions_from_file(&file_path) {
            Ok(actions) if !actions.is_empty() => {
                println!(
                    "[✓] Loaded solution playback from '{}' ({} steps)",
                    file_path,
                    actions.len()
                );
                playback.is_playback = true;
                playback.actions = actions;
                playback.auto_playing = true;
                initial_mode = editor::AppMode::Playback;
            }
            Ok(_) => {
                eprintln!("[!] Solution file '{}' contains no actions.", file_path);
            }
            Err(e) => {
                eprintln!("[!] Failed to load solution file '{}': {}", file_path, e);
            }
        }
    }

    let screenshot_config = ScreenshotConfig {
        output_path: screenshot_path,
        frame_counter: 0,
        target_frame,
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Laser Potato - Level Editor & Engine".into(),
                resolution: (1200, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(playback)
        .insert_resource(screenshot_config)
        .add_plugins(editor::EditorPlugin)
        .add_systems(
            Startup,
            (
                setup_game,
                render::setup_render_assets,
                render::setup_grid_labels,
            ),
        )
        .add_systems(
            Update,
            (
                playback_system.run_if(in_state(editor::AppMode::Playback)),
                input::keyboard_input_system.run_if(in_state(editor::AppMode::Playtest)),
                render::sync_bodies.after(input::keyboard_input_system),
                render::sync_lasers.after(input::keyboard_input_system),
                render::animate_laser_pfx,
                render::draw_coordinate_gizmo,
                render::draw_grid_gizmos,
                render::draw_combined_group_gizmos,
                update_victory_ui,
                screenshot_system,
            ),
        );

    // Initial state dispatch
    if initial_mode != editor::AppMode::Editor {
        app.insert_state(initial_mode);
    }

    app.run();
}

/// System for capturing in-engine screenshots and exiting cleanly for automated testing.
fn screenshot_system(
    mut commands: Commands,
    mut config: ResMut<ScreenshotConfig>,
) {
    let path = match &config.output_path {
        Some(p) => p.clone(),
        None => return,
    };
    config.frame_counter += 1;

    if config.frame_counter == config.target_frame {
        let p = path.clone();
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(move |event: bevy::ecs::observer::On<bevy::render::view::screenshot::ScreenshotCaptured>| {
                let img = event.image.clone();
                let dyn_res = img
                    .convert(bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb)
                    .and_then(|converted| converted.try_into_dynamic().ok())
                    .or_else(|| img.try_into_dynamic().ok());

                if let Some(dyn_img) = dyn_res {
                    let _ = dyn_img.save(&p);
                    println!("[✓] Automated screenshot captured to '{}'.", p);
                    std::process::exit(0);
                }
            });
    }
}

/// Create the simulation world from the test level, spawn camera, lights, and HUD.
fn setup_game(mut commands: Commands) {
    // --- simulation -------------------------------------------------------
    let world = level::test_level();
    let engine = turn::TurnEngine::new(world);
    commands.insert_resource(GameState { engine });

    // --- camera with pan/zoom controller & 3D preview child --------------
    commands
        .spawn((
            Camera3d::default(),
            editor::camera::MainCamera,
            editor::camera::CameraController::default(),
            Transform::from_xyz(3.5, 17.5, 5.5).looking_at(Vec3::new(3.5, 0.0, -3.0), Vec3::Y),
        ))
        .with_children(|cam| {
            cam.spawn((
                editor::Palette3dPreview,
                Mesh3d(Handle::default()),
                MeshMaterial3d(Handle::<StandardMaterial>::default()),
                Transform::from_xyz(-2.25, 0.95, -4.5)
                    .with_rotation(Quat::from_rotation_x(0.35))
                    .with_scale(Vec3::splat(0.42)),
                Visibility::Visible,
            ));
        });

    // --- light ------------------------------------------------------------
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // --- Playtest & Victory Banner (active during Playtest & Playback) ----
    commands
        .spawn((
            VictoryBanner,
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
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                VictoryBannerText,
                Text::new("Objective: Direct the laser to strike the Goal Pyramid\n[↑/↓/W/S] Move  |  [←/→/A/D] Turn  |  [Esc] Editor  |  [Z] Undo  |  [R] Reset"),
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
                    bottom: Val::Px(36.0),
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

/// Controls automated and manual step-by-step playback of a loaded solution.
fn playback_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<PlaybackState>,
    mut game: ResMut<GameState>,
) {
    if !playback.is_playback {
        return;
    }

    // Toggle Play / Pause
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyP) {
        playback.auto_playing = !playback.auto_playing;
    }

    // Manual Step Forward
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Period) {
        if playback.current_index < playback.actions.len() {
            let action = playback.actions[playback.current_index];
            game.engine.apply(action);
            playback.current_index += 1;
        }
    }

    // Manual Step Backward (Undo)
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::Comma) {
        if playback.current_index > 0 {
            game.engine.apply(turn::PlayerAction::Undo);
            playback.current_index -= 1;
        }
    }

    // Restart Playback
    if keys.just_pressed(KeyCode::KeyR) {
        game.engine.apply(turn::PlayerAction::Reset);
        playback.current_index = 0;
        playback.auto_playing = true;
    }

    // Automatic Step Progression
    if playback.auto_playing && !game.engine.outcome.is_game_over() {
        playback.step_timer.tick(time.delta());
        if playback.step_timer.just_finished() && playback.current_index < playback.actions.len() {
            let action = playback.actions[playback.current_index];
            game.engine.apply(action);
            playback.current_index += 1;
        }
    }
}

/// Update victory / objective / playback HUD banner during Playtest and Playback modes.
fn update_victory_ui(
    app_mode: Res<State<editor::AppMode>>,
    playback: Res<PlaybackState>,
    game: Res<GameState>,
    mut banner_query: Query<&mut Visibility, With<VictoryBanner>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<VictoryBannerText>>,
) {
    let mode = *app_mode.get();
    let is_active_mode = mode != editor::AppMode::Editor;

    for mut vis in &mut banner_query {
        *vis = if is_active_mode {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !is_active_mode {
        return;
    }

    for (mut text, mut color) in &mut text_query {
        if mode == editor::AppMode::Playback {
            if game.engine.is_won() {
                text.0 = format!(
                    "★ PLAYBACK COMPLETE: Goal Struck in {} Steps! ★\n[Esc] Return to Editor  |  [R] Replay  |  [←/→] Step",
                    playback.current_index
                );
                color.0 = Color::srgb(0.3, 1.0, 0.7);
            } else if game.engine.is_lost() {
                text.0 = format!(
                    "☠ PLAYBACK: Laser Vaporized Player at Step {} ☠\n[Esc] Return to Editor  |  [R] Restart  |  [←] Step Back",
                    playback.current_index
                );
                color.0 = Color::srgb(1.0, 0.3, 0.3);
            } else {
                let status_icon = if playback.auto_playing { "▶ [Playing]" } else { "⏸ [Paused]" };
                let next_action_str = if playback.current_index < playback.actions.len() {
                    format!("Next: {:?}", playback.actions[playback.current_index])
                } else {
                    "End of sequence".into()
                };

                text.0 = format!(
                    "{} Step {} / {} ({})\n[Space] Play/Pause  |  [←/→] Step  |  [Esc] Return to Editor  |  [R] Restart",
                    status_icon,
                    playback.current_index,
                    playback.actions.len(),
                    next_action_str
                );
                color.0 = Color::srgb(0.9, 0.95, 1.0);
            }
        } else if mode == editor::AppMode::Playtest {
            if let Some(err) = &game.engine.validation_error {
                text.0 = format!("⚠ INVALID LEVEL: {}\n[Esc] Return to Editor", err);
                color.0 = Color::srgb(1.0, 0.35, 0.35);
            } else {
                match game.engine.outcome {
                    turn::GameOutcome::Won => {
                        text.0 = "★ LEVEL COMPLETE! Laser Struck Goal Pyramid! ★\n[Esc] Return to Editor  |  [Z] Undo  |  [R] Reset".into();
                        color.0 = Color::srgb(0.3, 1.0, 0.7);
                    }
                    turn::GameOutcome::Lost => {
                        text.0 = "☠ GAME OVER! Laser Vaporized Player! ☠\n[Esc] Return to Editor  |  [Z] Undo  |  [R] Reset".into();
                        color.0 = Color::srgb(1.0, 0.3, 0.3);
                    }
                    turn::GameOutcome::InProgress => {
                        text.0 = "PLAYTEST MODE: Direct laser to Goal Pyramid\n[↑/↓/W/S] Move  |  [←/→/A/D] Turn  |  [Esc] Return to Editor  |  [Z] Undo  |  [R] Reset".into();
                        color.0 = Color::srgb(0.9, 0.95, 1.0);
                    }
                }
            }
        }
    }
}
