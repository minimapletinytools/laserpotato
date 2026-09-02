//! Laser Potato — Standalone Player & Level Picker.
//!
//! Provides a standalone player experience featuring:
//! - An interactive Level Picker menu with bundled and locally discovered levels.
//! - Direct CLI / URL parameter bypass (`play [level_file_or_name]`).
//! - Turn-based puzzle play with undo, reset, and step counters.
//! - Victory celebration banner with "Next Level" progression.
//! - Responsive 3D camera orbiting, zooming, and play-mode tilting.
//! - 100% Web (WASM) and desktop compatible with zero backend services.

#[cfg(not(target_arch = "wasm32"))]
use std::env;

use bevy::prelude::*;
use laserpotato::{
    camera::{self, CameraController, MainCamera},
    input,
    level,
    play_ui::{LevelCatalog, PlayMode, PlayUiPlugin},
    render,
    turn::TurnEngine,
    GameState,
};
#[cfg(not(target_arch = "wasm32"))]
use laserpotato::play_ui::LevelEntry;

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
    let mut initial_level_arg = None;
    #[allow(unused_mut)]
    let mut screenshot_path = None;
    #[allow(unused_mut)]
    let mut target_frame = 20;

    // 1. Desktop CLI Argument Parsing
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-s" | "--screenshot" => {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        i += 1;
                        screenshot_path = Some(args[i].clone());
                    } else {
                        screenshot_path = Some(String::from("play_screenshot.png"));
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
                "-l" | "--level" => {
                    if i + 1 < args.len() {
                        i += 1;
                        initial_level_arg = Some(args[i].clone());
                    }
                }
                arg if !arg.starts_with('-') => {
                    initial_level_arg = Some(arg.to_string());
                }
                _ => {}
            }
            i += 1;
        }
    }

    // 2. Web URL Parameter Parsing (e.g. ?level=simple_1 or ?level=2)
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                if let Some(query) = search.strip_prefix('?') {
                    for pair in query.split('&') {
                        let mut parts = pair.split('=');
                        if let (Some(key), Some(val)) = (parts.next(), parts.next()) {
                            if key == "level" || key == "l" {
                                initial_level_arg = Some(val.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut catalog = LevelCatalog::default();
    let mut initial_mode = PlayMode::LevelSelect;
    let mut selected_index = 0;

    // Check if initial level was requested
    if let Some(target) = initial_level_arg {
        let clean_target = target.trim_end_matches(".json");
        let found = catalog.levels.iter().position(|l| {
            l.id == target
                || l.id == clean_target
                || l.title.to_lowercase().contains(&clean_target.to_lowercase())
                || target.ends_with(&format!("{}.json", l.id))
        });

        if let Some(idx) = found {
            selected_index = idx;
            initial_mode = PlayMode::Playing;
        } else if let Ok(idx) = target.parse::<usize>() {
            if idx > 0 && idx <= catalog.levels.len() {
                selected_index = idx - 1;
                initial_mode = PlayMode::Playing;
            }
        } else {
            // Try loading from file path on desktop
            #[cfg(not(target_arch = "wasm32"))]
            if let Ok(data) = level::load_level_from_file(&target) {
                let filename = std::path::Path::new(&target)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("custom.json")
                    .to_string();
                catalog.levels.push(LevelEntry::new(
                    target.clone(),
                    String::new(),
                    filename,
                    data,
                ));
                selected_index = catalog.levels.len() - 1;
                initial_mode = PlayMode::Playing;
            }
        }
    }

    catalog.current_level_index = selected_index;

    let initial_world = if !catalog.levels.is_empty() {
        catalog.levels[selected_index].level_data.to_world()
    } else {
        level::test_level()
    };

    let screenshot_config = ScreenshotConfig {
        output_path: screenshot_path,
        frame_counter: 0,
        target_frame,
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Laser Potato".into(),
                canvas: Some("#bevy-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                resolution: (1200, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.14)))
        .add_plugins(PlayUiPlugin)
        .insert_resource(catalog)
        .insert_resource(screenshot_config)
        .insert_resource(GameState {
            engine: TurnEngine::new(initial_world),
        })
        .add_systems(
            Startup,
            (
                setup_scene,
                render::setup_render_assets,
                render::setup_grid_labels,
            ),
        )
        .add_systems(
            Update,
            (
                camera::camera_controller_system,
                render::sync_bodies,
                render::sync_lasers.after(render::sync_bodies),
                render::animate_laser_pfx,
                render::draw_grid_gizmos,
                render::draw_combined_group_gizmos,
                screenshot_system,
            ),
        )
        .add_systems(
            Update,
            input::keyboard_input_system.run_if(in_state(PlayMode::Playing)),
        );

    // Initial state dispatch
    if initial_mode != PlayMode::LevelSelect {
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

/// 3D Camera and scene lighting setup.
fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        CameraController {
            target: Vec3::new(4.5, 0.0, -4.5),
            distance: 18.0,
            pitch: 62.0_f32.to_radians(),
            yaw: 0.0,
            target_yaw: 0.0,
            min_distance: 6.0,
            max_distance: 40.0,
            tilt_yaw: 0.0,
            tilt_pitch: 0.0,
        },
        Transform::from_xyz(4.5, 17.5, 5.0).looking_at(Vec3::new(4.5, 0.0, -4.5), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
