use std::env;
use bevy::prelude::*;

pub use laserpotato::{
    block_types, camera, editor, input, laser, level, playback::{self, PlaybackState},
    play_ui::{self, VictoryBanner, VictoryBannerText}, render, sim, solver, turn, GameState,
};

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

/// Configuration for initial level loaded on startup.
#[derive(Resource, Default)]
pub struct InitialLevelConfig {
    pub path: Option<String>,
}

/// Configuration for initial app mode on startup.
#[derive(Resource, Default)]
pub struct InitialModeConfig(pub editor::AppMode);

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut replay_file = None;
    let mut screenshot_path = None;
    let mut level_file = None;
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
            "-l" | "--level" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    level_file = Some(args[i].clone());
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
            "--tester" => {
                initial_mode = editor::AppMode::LevelTester;
            }
            pos if !pos.starts_with('-') && pos.ends_with(".json") => {
                level_file = Some(pos.to_string());
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

    let initial_level_config = InitialLevelConfig {
        path: level_file,
    };

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.14)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Laser Potato - Level Editor & Engine".into(),
                resolution: (1200, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(playback)
        .insert_resource(screenshot_config)
        .insert_resource(initial_level_config)
        .insert_resource(InitialModeConfig(initial_mode))
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
                playback::playback_system.run_if(in_state(editor::AppMode::Playback)),
                input::keyboard_input_system.run_if(in_state(editor::AppMode::Playtest)),
                render::sync_bodies,
                render::sync_lasers.after(render::sync_bodies),
                render::animate_laser_pfx,
                render::draw_coordinate_gizmo,
                render::draw_grid_gizmos,
                render::draw_combined_group_gizmos,
                play_ui::update_victory_ui,
                screenshot_system,
            ),
        );

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

/// Create the simulation world from the test level or custom level, spawn camera, lights, and HUD.
fn setup_game(
    mut commands: Commands,
    initial_lvl: Res<InitialLevelConfig>,
    initial_mode_cfg: Res<InitialModeConfig>,
    mut editor: ResMut<editor::EditorState>,
    mut next_mode: ResMut<NextState<editor::AppMode>>,
) {
    if initial_mode_cfg.0 != editor::AppMode::Editor {
        next_mode.set(initial_mode_cfg.0);
    }
    // --- simulation -------------------------------------------------------
    let (world, path_str, sols, profile) = if let Some(path) = &initial_lvl.path {
        match level::load_level_from_file(path) {
            Ok(lvl) => {
                let p = path.clone();
                let s = lvl.solutions.clone();
                let prof = lvl.quality_profile.clone();
                println!("[✓] Loaded initial level: {} ({} bodies)", lvl.name, lvl.bodies.len());
                (lvl.to_world(), p, s, prof)
            }
            Err(e) => {
                eprintln!("[!] Failed to load level '{}': {}", path, e);
                (level::test_level(), "levels/custom_puzzle.json".to_string(), Vec::new(), None)
            }
        }
    } else {
        (level::test_level(), "levels/custom_puzzle.json".to_string(), Vec::new(), None)
    };

    editor.current_level_path = path_str;
    editor.solutions = sols;
    editor.puzzle_profile = profile;
    editor.last_saved_hash = level::compute_level_hash(&world);

    let engine = turn::TurnEngine::new(world);
    commands.insert_resource(GameState { engine });

    // --- camera with pan/zoom controller & 3D preview child --------------
    commands
        .spawn((
            Camera3d::default(),
            editor::camera::MainCamera,
            editor::camera::CameraController::default(),
            Transform::from_xyz(4.5, 17.5, 5.0).looking_at(Vec3::new(4.5, 0.0, -4.5), Vec3::Y),
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
    play_ui::spawn_victory_banner(&mut commands);

    // --- Coordinate Gizmo Legend (bottom left of viewport) --------------
    if render::SHOW_COORDINATE_LEGEND {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(260.0),
                    bottom: Val::Px(36.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Game Axes (RHS)\n+X: Right (Red)\n+Y: Forward (Green)\n+Z: Up (Blue)"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.9, 0.95, 1.0)),
                ));
            });
    }
}
