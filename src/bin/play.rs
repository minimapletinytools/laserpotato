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
    level::{self, LevelData},
    render,
    turn::TurnEngine,
    GameState,
};

// ---------------------------------------------------------------------------
// App States
// ---------------------------------------------------------------------------

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum PlayMode {
    #[default]
    LevelSelect,
    Playing,
}

// ---------------------------------------------------------------------------
// Level Catalog & Bundled Levels
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct LevelEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub level_data: LevelData,
}

const BUNDLED_LEVELS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "simple_1",
        "1. First Light",
        "An introductory obstacle course. Move the mirror to guide the laser beam into the golden pyramid.",
        "Introductory",
        include_str!("../../levels/simple_1.json"),
    ),
    (
        "default_puzzle",
        "2. Mirror Maze",
        "Multi-stage reflection puzzle with moveable mirrors, partition walls, and laser redirection.",
        "Medium",
        include_str!("../../levels/default_puzzle.json"),
    ),
    (
        "simple_2",
        "3. Laser Grid",
        "Complex grid navigation with precision block pushing and multiple reflection paths.",
        "Advanced",
        include_str!("../../levels/simple_2.json"),
    ),
];

#[derive(Resource, Clone, Debug)]
pub struct LevelCatalog {
    pub levels: Vec<LevelEntry>,
    pub current_level_index: usize,
}

impl Default for LevelCatalog {
    fn default() -> Self {
        let mut levels = Vec::new();

        // 1. Load embedded bundled levels
        for &(id, title, desc, diff, json_str) in BUNDLED_LEVELS {
            if let Ok(level_data) = serde_json::from_str::<LevelData>(json_str) {
                levels.push(LevelEntry {
                    id: id.to_string(),
                    title: title.to_string(),
                    description: desc.to_string(),
                    difficulty: diff.to_string(),
                    level_data,
                });
            }
        }

        // 2. On desktop, scan levels/ directory for additional JSON levels
        #[cfg(not(target_arch = "wasm32"))]
        {
            let disk_files = level::list_level_files();
            for file_path in disk_files {
                let file_name = std::path::Path::new(&file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                if !levels.iter().any(|l| l.id == file_name) {
                    if let Ok(level_data) = level::load_level_from_file(&file_path) {
                        let title = if level_data.name.is_empty() || level_data.name == "Custom Level" {
                            format!("Custom: {}", file_name)
                        } else {
                            level_data.name.clone()
                        };
                        levels.push(LevelEntry {
                            id: file_name,
                            title,
                            description: format!("Custom puzzle loaded from {}", file_path),
                            difficulty: "Custom".to_string(),
                            level_data,
                        });
                    }
                }
            }
        }

        Self {
            levels,
            current_level_index: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Screenshot Configuration
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// UI Component Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct LevelSelectRoot;

#[derive(Component)]
pub struct LevelCardButton(pub usize);

#[derive(Component)]
pub struct PlayHudRoot;

#[derive(Component)]
pub struct PlayHudLevelNameText;

#[derive(Component)]
pub struct PlayHudMovesText;

#[derive(Component)]
pub struct VictoryOverlayRoot;

#[derive(Component)]
pub struct VictoryOverlayText;

#[derive(Component)]
pub struct GameOverOverlayRoot;

#[derive(Component)]
pub struct GameOverOverlayText;

#[derive(Component)]
pub struct NextLevelButton;

#[derive(Component)]
pub struct ReplayButton;

#[derive(Component)]
pub struct ReturnToMenuButton;

#[derive(Component)]
pub struct UndoButton;

#[derive(Component)]
pub struct ResetButton;

// ---------------------------------------------------------------------------
// Main Application Setup
// ---------------------------------------------------------------------------

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
                catalog.levels.push(LevelEntry {
                    id: target.clone(),
                    title: data.name.clone(),
                    description: format!("Loaded from {}", target),
                    difficulty: "Custom".to_string(),
                    level_data: data,
                });
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
        .init_state::<PlayMode>()
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
                setup_ui,
            ),
        )
        .add_systems(
            Update,
            (
                camera::camera_controller_system,
                render::sync_bodies,
                render::sync_lasers,
                render::animate_laser_pfx,
                render::draw_grid_gizmos,
                render::draw_combined_group_gizmos,
                ui_button_system,
                update_hud_system,
                screenshot_system,
            ),
        )
        .add_systems(
            Update,
            (
                input::keyboard_input_system,
                gameplay_shortcuts_system,
            )
                .run_if(in_state(PlayMode::Playing)),
        );

    // Initial state dispatch
    if initial_mode != PlayMode::LevelSelect {
        app.insert_state(initial_mode);
    }

    app.run();
}

// ---------------------------------------------------------------------------
// Screenshot System
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// 3D Scene Setup (Camera, Light, World)
// ---------------------------------------------------------------------------

fn setup_scene(mut commands: Commands) {
    // 3D Camera with orbit, pan, zoom, and mouse drag tilt
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

    // Directional Sunlight
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// ---------------------------------------------------------------------------
// UI Setup (Level Picker & Play HUD)
// ---------------------------------------------------------------------------

fn setup_ui(mut commands: Commands, catalog: Res<LevelCatalog>) {
    // --- 1. Level Select Menu Screen ---
    commands
        .spawn((
            LevelSelectRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.95)),
        ))
        .with_children(|root| {
            // Header
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new("LASER POTATO"),
                    TextFont::from_font_size(36.0),
                    TextColor(Color::srgb(0.3, 0.85, 1.0)),
                ));
                header.spawn((
                    Text::new("Select a puzzle level to begin"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.7, 0.75, 0.85)),
                ));
            });

            // Level List Container
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                max_width: Val::Px(720.0),
                row_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|list| {
                for (i, lvl) in catalog.levels.iter().enumerate() {
                    let diff_color = match lvl.difficulty.as_str() {
                        "Introductory" => Color::srgb(0.3, 0.9, 0.5),
                        "Medium" => Color::srgb(1.0, 0.8, 0.2),
                        "Advanced" => Color::srgb(1.0, 0.4, 0.3),
                        _ => Color::srgb(0.5, 0.8, 1.0),
                    };

                    list.spawn((
                        Button,
                        LevelCardButton(i),
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.15, 0.22, 0.95)),
                        BorderColor::all(Color::srgba(0.25, 0.35, 0.50, 0.6)),
                    ))
                    .with_children(|card| {
                        card.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            max_width: Val::Percent(75.0),
                            ..default()
                        })
                        .with_children(|info| {
                            info.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|title_row| {
                                title_row.spawn((
                                    Text::new(&lvl.title),
                                    TextFont::from_font_size(18.0),
                                    TextColor(Color::srgb(0.95, 0.96, 1.0)),
                                ));
                                title_row.spawn((
                                    Text::new(format!("[{}]", lvl.difficulty)),
                                    TextFont::from_font_size(13.0),
                                    TextColor(diff_color),
                                ));
                            });

                            info.spawn((
                                Text::new(&lvl.description),
                                TextFont::from_font_size(13.0),
                                TextColor(Color::srgb(0.65, 0.70, 0.80)),
                            ));
                        });

                        // Play Arrow Button
                        card.spawn((
                            Node {
                                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.18, 0.35, 0.65, 0.9)),
                            BorderColor::all(Color::srgb(0.3, 0.6, 1.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("PLAY >"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                            ));
                        });
                    });
                }
            });

            // Footer Instructions
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(16.0)),
                row_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|footer| {
                footer.spawn((
                    Text::new("Controls: [W/S / Up/Down] Move & Push  |  [A/D] Turn  |  [Z] Undo  |  [R] Reset  |  [Esc] Menu"),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::srgb(0.6, 0.65, 0.75)),
                ));
                footer.spawn((
                    Text::new("Mouse Drag: Tilt 3D View  |  Scroll Wheel: Zoom Camera"),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.45, 0.50, 0.60)),
                ));
            });
        });

    // --- 2. In-Game Play HUD Header ---
    commands
        .spawn((
            PlayHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.88)),
            Visibility::Hidden,
        ))
        .with_children(|hud| {
            // Level Title & Moves
            hud.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|left| {
                left.spawn((
                    PlayHudLevelNameText,
                    Text::new("Level 1: First Light"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.95, 0.95, 1.0)),
                ));
                left.spawn((
                    PlayHudMovesText,
                    Text::new("Steps: 0"),
                    TextFont::from_font_size(15.0),
                    TextColor(Color::srgb(0.3, 0.85, 1.0)),
                ));
            });

            // Quick Actions (Menu, Reset, Undo)
            hud.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|right| {
                right
                    .spawn((
                        Button,
                        UndoButton,
                        Node {
                            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.18, 0.25, 0.9)),
                        BorderColor::all(Color::srgba(0.35, 0.45, 0.60, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Undo [Z]"),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        ));
                    });

                right
                    .spawn((
                        Button,
                        ResetButton,
                        Node {
                            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.18, 0.25, 0.9)),
                        BorderColor::all(Color::srgba(0.35, 0.45, 0.60, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Reset [R]"),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        ));
                    });

                right
                    .spawn((
                        Button,
                        ReturnToMenuButton,
                        Node {
                            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.22, 0.15, 0.25, 0.9)),
                        BorderColor::all(Color::srgba(0.60, 0.35, 0.60, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Level Select [Esc]"),
                            TextFont::from_font_size(13.0),
                            TextColor(Color::srgb(0.95, 0.85, 1.0)),
                        ));
                    });
            });
        });

    // --- 3. Victory Overlay Modal ---
    commands
        .spawn((
            VictoryOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(28.0),
                left: Val::Percent(25.0),
                right: Val::Percent(25.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.12, 0.08, 0.96)),
            BorderColor::all(Color::srgb(0.25, 0.95, 0.65)),
            Visibility::Hidden,
        ))
        .with_children(|modal| {
            modal.spawn((
                Text::new("*** LEVEL COMPLETE! ***"),
                TextFont::from_font_size(28.0),
                TextColor(Color::srgb(0.3, 1.0, 0.6)),
            ));

            modal.spawn((
                VictoryOverlayText,
                Text::new("Goal pyramid struck by laser!"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.9, 0.95, 0.95)),
            ));

            modal.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|actions| {
                actions
                    .spawn((
                        Button,
                        NextLevelButton,
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(10.0), Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.45, 0.25, 0.95)),
                        BorderColor::all(Color::srgb(0.4, 1.0, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Next Level >"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });

                actions
                    .spawn((
                        Button,
                        ReplayButton,
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(10.0), Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.16, 0.22, 0.9)),
                        BorderColor::all(Color::srgba(0.4, 0.5, 0.6, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Replay [R]"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        ));
                    });

                actions
                    .spawn((
                        Button,
                        ReturnToMenuButton,
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(10.0), Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.16, 0.22, 0.9)),
                        BorderColor::all(Color::srgba(0.4, 0.5, 0.6, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Level Select [Esc]"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        ));
                    });
            });
        });

    // --- 4. Game Over Overlay Modal ---
    commands
        .spawn((
            GameOverOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(28.0),
                left: Val::Percent(25.0),
                right: Val::Percent(25.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.04, 0.04, 0.96)),
            BorderColor::all(Color::srgb(1.0, 0.3, 0.3)),
            Visibility::Hidden,
        ))
        .with_children(|modal| {
            modal.spawn((
                Text::new("! LASER VAPORIZED PLAYER !"),
                TextFont::from_font_size(26.0),
                TextColor(Color::srgb(1.0, 0.35, 0.35)),
            ));

            modal.spawn((
                GameOverOverlayText,
                Text::new("Player walked into the laser beam."),
                TextFont::from_font_size(15.0),
                TextColor(Color::srgb(0.95, 0.90, 0.90)),
            ));

            modal.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|actions| {
                actions
                    .spawn((
                        Button,
                        UndoButton,
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(10.0), Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.40, 0.15, 0.15, 0.95)),
                        BorderColor::all(Color::srgb(1.0, 0.4, 0.4)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Undo Move [Z]"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });

                actions
                    .spawn((
                        Button,
                        ResetButton,
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(10.0), Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.18, 0.25, 0.9)),
                        BorderColor::all(Color::srgba(0.4, 0.5, 0.6, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Restart [R]"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        ));
                    });

                actions
                    .spawn((
                        Button,
                        ReturnToMenuButton,
                        Node {
                            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(10.0), Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.18, 0.25, 0.9)),
                        BorderColor::all(Color::srgba(0.4, 0.5, 0.6, 0.6)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Level Select [Esc]"),
                            TextFont::from_font_size(15.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        ));
                    });
            });
        });
}

// ---------------------------------------------------------------------------
// UI Systems & Event Handlers
// ---------------------------------------------------------------------------

fn ui_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&LevelCardButton>,
            Option<&NextLevelButton>,
            Option<&ReplayButton>,
            Option<&ReturnToMenuButton>,
            Option<&UndoButton>,
            Option<&ResetButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut catalog: ResMut<LevelCatalog>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<PlayMode>>,
) {
    for (interaction, lvl_btn, next_btn, replay_btn, menu_btn, undo_btn, reset_btn) in
        &mut interaction_query
    {
        if *interaction == Interaction::Pressed {
            if let Some(card) = lvl_btn {
                let idx = card.0;
                if idx < catalog.levels.len() {
                    catalog.current_level_index = idx;
                    let world = catalog.levels[idx].level_data.to_world();
                    game.engine = TurnEngine::new(world);
                    next_mode.set(PlayMode::Playing);
                }
            } else if next_btn.is_some() {
                if catalog.current_level_index + 1 < catalog.levels.len() {
                    catalog.current_level_index += 1;
                } else {
                    catalog.current_level_index = 0;
                }
                let idx = catalog.current_level_index;
                let world = catalog.levels[idx].level_data.to_world();
                game.engine = TurnEngine::new(world);
                next_mode.set(PlayMode::Playing);
            } else if replay_btn.is_some() || reset_btn.is_some() {
                let idx = catalog.current_level_index;
                if idx < catalog.levels.len() {
                    let world = catalog.levels[idx].level_data.to_world();
                    game.engine = TurnEngine::new(world);
                } else {
                    game.engine.apply(laserpotato::turn::PlayerAction::Reset);
                }
            } else if undo_btn.is_some() {
                game.engine.apply(laserpotato::turn::PlayerAction::Undo);
            } else if menu_btn.is_some() {
                next_mode.set(PlayMode::LevelSelect);
            }
        }
    }
}

fn gameplay_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut catalog: ResMut<LevelCatalog>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<PlayMode>>,
) {
    // Esc -> Return to Level Select
    if keys.just_pressed(KeyCode::Escape) {
        next_mode.set(PlayMode::LevelSelect);
    }

    // When level is won: Space / Enter / N advances to next level
    if game.engine.is_won() {
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyN) {
            if catalog.current_level_index + 1 < catalog.levels.len() {
                catalog.current_level_index += 1;
            } else {
                catalog.current_level_index = 0;
            }
            let idx = catalog.current_level_index;
            let world = catalog.levels[idx].level_data.to_world();
            game.engine = TurnEngine::new(world);
        }
    }
}

fn update_hud_system(
    mode: Res<State<PlayMode>>,
    catalog: Res<LevelCatalog>,
    game: Res<GameState>,
    mut level_select_query: Query<&mut Visibility, (With<LevelSelectRoot>, Without<PlayHudRoot>, Without<VictoryOverlayRoot>, Without<GameOverOverlayRoot>)>,
    mut hud_query: Query<&mut Visibility, (With<PlayHudRoot>, Without<LevelSelectRoot>, Without<VictoryOverlayRoot>, Without<GameOverOverlayRoot>)>,
    mut victory_query: Query<&mut Visibility, (With<VictoryOverlayRoot>, Without<LevelSelectRoot>, Without<PlayHudRoot>, Without<GameOverOverlayRoot>)>,
    mut game_over_query: Query<&mut Visibility, (With<GameOverOverlayRoot>, Without<LevelSelectRoot>, Without<PlayHudRoot>, Without<VictoryOverlayRoot>)>,
    mut level_name_text: Query<&mut Text, (With<PlayHudLevelNameText>, Without<PlayHudMovesText>, Without<VictoryOverlayText>, Without<GameOverOverlayText>)>,
    mut moves_text: Query<&mut Text, (With<PlayHudMovesText>, Without<PlayHudLevelNameText>, Without<VictoryOverlayText>, Without<GameOverOverlayText>)>,
    mut victory_text: Query<&mut Text, (With<VictoryOverlayText>, Without<PlayHudLevelNameText>, Without<PlayHudMovesText>, Without<GameOverOverlayText>)>,
) {
    let is_select = *mode.get() == PlayMode::LevelSelect;
    let is_playing = *mode.get() == PlayMode::Playing;

    for mut vis in &mut level_select_query {
        *vis = if is_select { Visibility::Visible } else { Visibility::Hidden };
    }

    for mut vis in &mut hud_query {
        *vis = if is_playing { Visibility::Visible } else { Visibility::Hidden };
    }

    let is_won = is_playing && game.engine.is_won();
    let is_lost = is_playing && game.engine.is_lost();

    for mut vis in &mut victory_query {
        *vis = if is_won { Visibility::Visible } else { Visibility::Hidden };
    }

    for mut vis in &mut game_over_query {
        *vis = if is_lost { Visibility::Visible } else { Visibility::Hidden };
    }

    if is_playing {
        let current_lvl = catalog.levels.get(catalog.current_level_index);
        let title = current_lvl.map(|l| l.title.as_str()).unwrap_or("Puzzle Level");
        let steps = game.engine.action_history.len();

        for mut text in &mut level_name_text {
            text.0 = title.to_string();
        }

        for mut text in &mut moves_text {
            text.0 = format!("Moves: {}", steps);
        }

        for mut text in &mut victory_text {
            text.0 = format!("Goal pyramid energized in {} move(s)!", steps);
        }
    }
}
