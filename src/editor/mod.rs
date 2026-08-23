//! Level Editor module for *Laser Potato*.
//!
//! Provides palette selection, property toggles with validation, 3D grid
//! raycasting and placement, block inspector, folderized level management,
//! background thread solver integration, and seamless transitions to playtest
//! and solution replay modes.

pub mod camera;
pub mod ui;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use bevy::prelude::*;
use glam::{IVec2, IVec3};

use crate::block_types::BlockKind;
use crate::level::{self, compute_level_hash, LevelData};
use crate::sim::{BodyId, CubeRot, TagKind, TagValue, World};
use crate::solver::{self, SolveResult, SolverConfig};
use crate::turn::{PlayerAction, TurnEngine};
use crate::GameState;

// ---------------------------------------------------------------------------
// App Modes
// ---------------------------------------------------------------------------

/// High-level application mode.
#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AppMode {
    #[default]
    Editor,
    Playtest,
    Playback,
}

// ---------------------------------------------------------------------------
// Editor Actions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAction {
    NewLevel,
    Save,
    SaveAs,
    ToggleLevelsMenu,
    AttemptSolve,
    TestPlay,
    TestWithSolution,
}

// ---------------------------------------------------------------------------
// Editor State Resource
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct EditorState {
    /// Currently selected base block kind from palette.
    pub selected_kind: BlockKind,
    /// Property toggle: true = stationary (fixed), false = moveable.
    pub is_fixed: bool,
    /// Currently selected body on grid for Inspector.
    pub selected_body_id: Option<BodyId>,
    /// Currently dragging body ID on grid.
    pub dragging_body_id: Option<BodyId>,
    /// Hovered grid coordinate (X, Y) on the ground plane.
    pub hovered_cell: Option<IVec2>,
    /// Current level file path.
    pub current_level_path: String,
    /// Background solver worker receiver.
    pub solver_rx: Option<Arc<Mutex<Receiver<(u64, SolveResult)>>>>,
    /// Level hash at time solver was launched.
    pub solving_hash: Option<u64>,
    /// Human-readable solver status badge.
    pub solver_status: String,
    /// Cached solution keyed to level hash: (hash, actions).
    pub cached_solution: Option<(u64, Vec<PlayerAction>)>,
    /// World state snapshot when entering Playtest / Playback mode.
    pub backup_world: Option<World>,
    /// Toast notification banner message & decay timer.
    pub status_message: Option<(String, Timer)>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            selected_kind: BlockKind::Mirror,
            is_fixed: false,
            selected_body_id: None,
            dragging_body_id: None,
            hovered_cell: None,
            current_level_path: String::from("levels/default_puzzle.json"),
            solver_rx: None,
            solving_hash: None,
            solver_status: String::from("Idle"),
            cached_solution: None,
            backup_world: None,
            status_message: None,
        }
    }
}

impl EditorState {
    /// Allowed (moveable, fixed) property configuration for a given block kind.
    pub fn allowed_fixed_state(&self, kind: BlockKind) -> (bool, bool) {
        match kind {
            BlockKind::Wall | BlockKind::Goal => (false, true),
            BlockKind::Player => (true, false),
            BlockKind::Mirror | BlockKind::LaserSource | BlockKind::Pushable => (true, true),
        }
    }

    /// Set a temporary notification toast in the status bar.
    pub fn toast(&mut self, message: impl Into<String>) {
        self.status_message = Some((
            message.into(),
            Timer::new(Duration::from_secs(3), TimerMode::Once),
        ));
    }
}

/// Marker for the live rotating 3D block preview in the palette sidebar.
#[derive(Component)]
pub struct Palette3dPreview;

// ---------------------------------------------------------------------------
// Editor Plugin
// ---------------------------------------------------------------------------

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppMode>()
            .init_resource::<EditorState>()
            .add_systems(
                Startup,
                ui::setup_editor_ui,
            )
            .add_systems(
                Update,
                (
                    camera::camera_controller_system,
                    ui::update_editor_ui_system,
                    update_palette_3d_preview,
                    editor_grid_interaction_system.run_if(in_state(AppMode::Editor)),
                    editor_button_clicks_system.run_if(in_state(AppMode::Editor)),
                    background_solver_poll_system,
                    draw_editor_selection_gizmos,
                    editor_keyboard_shortcuts_system,
                    toast_decay_system,
                ),
            );
    }
}

pub fn update_palette_3d_preview(
    time: Res<Time>,
    app_mode: Res<State<AppMode>>,
    editor: Res<EditorState>,
    render_assets: Option<Res<crate::render::RenderAssets>>,
    mut query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>, &mut Transform, &mut Visibility), With<Palette3dPreview>>,
) {
    let Some(render_assets) = render_assets else { return };

    for (mut mesh, mut mat, mut transform, mut vis) in &mut query {
        if *app_mode.get() != AppMode::Editor {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;

        // Rotate slowly around Y
        transform.rotate_local_y(1.2 * time.delta_secs());

        let (can_moveable, can_fixed) = editor.allowed_fixed_state(editor.selected_kind);
        let is_moveable = if !can_moveable {
            false
        } else if !can_fixed {
            true
        } else {
            !editor.is_fixed
        };

        let (target_mesh, target_mat) = match editor.selected_kind {
            BlockKind::Player => (render_assets.player_mesh.clone(), render_assets.player_mat.clone()),
            BlockKind::Goal => (render_assets.pyramid_mesh.clone(), render_assets.goal_mat.clone()),
            BlockKind::Wall => (render_assets.cube_mesh.clone(), render_assets.fixed_wall_mat.clone()),
            BlockKind::Pushable => {
                let m = if is_moveable {
                    render_assets.moveable_pushable_mat.clone()
                } else {
                    render_assets.fixed_pushable_mat.clone()
                };
                (render_assets.cube_mesh.clone(), m)
            }
            BlockKind::Mirror => {
                let m = if is_moveable {
                    render_assets.moveable_mirror_mat.clone()
                } else {
                    render_assets.fixed_mirror_mat.clone()
                };
                (render_assets.mirror_mesh.clone(), m)
            }
            BlockKind::LaserSource => {
                let m = if is_moveable {
                    render_assets.moveable_laser_mat.clone()
                } else {
                    render_assets.fixed_laser_mat.clone()
                };
                (render_assets.cube_mesh.clone(), m)
            }
        };

        mesh.0 = target_mesh;
        mat.0 = target_mat;
    }
}

// ---------------------------------------------------------------------------
// 3D Grid Raycasting & Mouse Interaction
// ---------------------------------------------------------------------------

fn raycast_ground_plane(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
) -> Option<IVec2> {
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    if ray.direction.y.abs() < 1e-5 {
        return None;
    }
    // Floor plane is at Y = 0 in Bevy space
    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }
    let world_pt = ray.origin + ray.direction * t;
    let gx = (world_pt.x + 0.5).floor() as i32;
    let gy = (-world_pt.z + 0.5).floor() as i32; // In sim coordinates, +Y is -Z in Bevy
    Some(IVec2::new(gx, gy))
}

/// Helper to check if a body can be moved to a target anchor without colliding with other bodies.
fn can_move_body_to(world: &World, body_id: BodyId, target_anchor: IVec3) -> bool {
    let body = match world.body(body_id) {
        Some(b) => b,
        None => return false,
    };
    for local in &body.shape {
        let world_cell = target_anchor + body.orientation.apply(*local);
        if let Some(occ) = world.body_at(world_cell) {
            if occ.id != body_id {
                return false;
            }
        }
    }
    true
}

fn editor_grid_interaction_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<camera::MainCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
) {
    // Keyboard shortcuts for selected block in editor
    if let Some(id) = editor.selected_body_id {
        if keyboard.just_pressed(KeyCode::KeyR) {
            if let Some(b) = game.engine.world.body_mut(id) {
                b.orientation = b.orientation.rotate_z_cw();
                game.engine.world.sync_grid();
                editor.cached_solution = None;
                editor.toast("Rotated block CW (Key: R).");
            }
        } else if keyboard.just_pressed(KeyCode::KeyX) {
            if let Some(b) = game.engine.world.body_mut(id) {
                b.orientation = b.orientation.reflect_x();
                game.engine.world.sync_grid();
                editor.cached_solution = None;
                editor.toast("Flipped block across X axis (Key: X).");
            }
        } else if keyboard.just_pressed(KeyCode::KeyY) {
            if let Some(b) = game.engine.world.body_mut(id) {
                b.orientation = b.orientation.reflect_y();
                game.engine.world.sync_grid();
                editor.cached_solution = None;
                editor.toast("Flipped block across Y axis (Key: Y).");
            }
        } else if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
            game.engine.world.despawn(id);
            editor.selected_body_id = None;
            editor.cached_solution = None;
            editor.toast("Deleted selected block (Key: Del).");
        }
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        editor.hovered_cell = None;
        return;
    };

    // Ignore clicks if cursor is over sidebars or top bar
    if cursor_pos.x < 260.0 && cursor_pos.y > 55.0 {
        editor.hovered_cell = None;
        return;
    }
    if cursor_pos.x > (window.width() - 260.0) && cursor_pos.y > 55.0 && editor.selected_body_id.is_some() {
        editor.hovered_cell = None;
        return;
    }
    if cursor_pos.y < 55.0 {
        editor.hovered_cell = None;
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(cell) = raycast_ground_plane(camera, camera_transform, cursor_pos) else {
        editor.hovered_cell = None;
        return;
    };

    editor.hovered_cell = Some(cell);
    let cell_pos = IVec3::new(cell.x, cell.y, 0);

    // Left Click: Place or Select & Start Dragging
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(body) = game.engine.world.body_at(cell_pos) {
            let body_id = body.id;
            let kind = body.kind;
            editor.selected_body_id = Some(body_id);
            editor.dragging_body_id = Some(body_id);
            editor.toast(format!("Selected {:?} at ({}, {})", kind, cell.x, cell.y));
        } else {
            // Cell empty -> place currently selected block from palette
            let kind = editor.selected_kind;
            let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
            let is_fixed = if !can_moveable {
                true
            } else if !can_fixed {
                false
            } else {
                editor.is_fixed
            };

            // If placing Player and player already exists, relocate player
            if kind == BlockKind::Player {
                if let Some(player_id) = game.engine.world.player_id() {
                    if let Some(p) = game.engine.world.body_mut(player_id) {
                        p.anchor = cell_pos;
                        game.engine.world.sync_grid();
                        editor.selected_body_id = Some(player_id);
                        editor.dragging_body_id = Some(player_id);
                        editor.toast("Relocated player character.");
                        editor.cached_solution = None;
                        return;
                    }
                }
            }

            let new_id = game.engine.world.spawn(kind, cell_pos, vec![IVec3::ZERO]);
            if let Some(b) = game.engine.world.body_mut(new_id) {
                if is_fixed {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
            }
            game.engine.world.sync_grid();
            editor.selected_body_id = Some(new_id);
            editor.dragging_body_id = Some(new_id);
            editor.cached_solution = None;
            editor.toast(format!("Placed {:?} at ({}, {})", kind, cell.x, cell.y));
        }
    } else if mouse_button.pressed(MouseButton::Left) {
        // Continuous Dragging of selected / placed block across grid
        if let Some(drag_id) = editor.dragging_body_id {
            if let Some(body) = game.engine.world.body(drag_id) {
                if body.anchor != cell_pos {
                    if can_move_body_to(&game.engine.world, drag_id, cell_pos) {
                        if let Some(b) = game.engine.world.body_mut(drag_id) {
                            b.anchor = cell_pos;
                        }
                        game.engine.world.sync_grid();
                        editor.cached_solution = None;
                    }
                }
            }
        }
    }

    // Release Left Click: End Dragging
    if mouse_button.just_released(MouseButton::Left) {
        if let Some(drag_id) = editor.dragging_body_id.take() {
            if let Some(body) = game.engine.world.body(drag_id) {
                editor.toast(format!("Moved {:?} to ({}, {})", body.kind, body.anchor.x, body.anchor.y));
            }
        }
    }

    // Right Click: Delete Block
    if mouse_button.just_pressed(MouseButton::Right) {
        if let Some(body) = game.engine.world.body_at(cell_pos) {
            let id = body.id;
            game.engine.world.despawn(id);
            if editor.selected_body_id == Some(id) {
                editor.selected_body_id = None;
            }
            if editor.dragging_body_id == Some(id) {
                editor.dragging_body_id = None;
            }
            editor.cached_solution = None;
            editor.toast(format!("Deleted block at ({}, {})", cell.x, cell.y));
        }
    }
}

// ---------------------------------------------------------------------------
// Button Click Dispatcher
// ---------------------------------------------------------------------------

fn editor_button_clicks_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ui::PaletteButton>,
            Option<&ui::PropertyToggleButton>,
            Option<&ui::ActionButton>,
            Option<&ui::RotateCwButton>,
            Option<&ui::RotateCcwButton>,
            Option<&ui::ReflectXButton>,
            Option<&ui::ReflectYButton>,
            Option<&ui::ToggleFixedButton>,
            Option<&ui::DeleteBlockButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<AppMode>>,
    mut playback: ResMut<crate::PlaybackState>,
) {
    for (interaction, palette_btn, prop_btn, action_btn, rot_cw, rot_ccw, ref_x, ref_y, toggle_fixed, del_btn) in
        &mut interaction_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 1. Palette block selection
        if let Some(btn) = palette_btn {
            editor.selected_kind = btn.0;
            let (can_moveable, can_fixed) = editor.allowed_fixed_state(btn.0);
            if !can_moveable {
                editor.is_fixed = true;
            } else if !can_fixed {
                editor.is_fixed = false;
            }
            let label = format!("Selected tool: {:?}", btn.0);
            editor.toast(label);
        }

        // 2. Property toggle (Moveable vs Stationary)
        if let Some(btn) = prop_btn {
            let (can_moveable, can_fixed) = editor.allowed_fixed_state(editor.selected_kind);
            if btn.0 && can_fixed {
                editor.is_fixed = true;
                editor.toast("Tool property: Stationary (Fixed)");
            } else if !btn.0 && can_moveable {
                editor.is_fixed = false;
                editor.toast("Tool property: Moveable");
            }
        }

        // 3. Top Action buttons
        if let Some(btn) = action_btn {
            match btn.0 {
                EditorAction::NewLevel => {
                    let mut world = World::new();
                    world.spawn(BlockKind::Player, IVec3::new(2, 2, 0), vec![IVec3::ZERO]);
                    let gid = world.spawn(BlockKind::Goal, IVec3::new(6, 6, 0), vec![IVec3::ZERO]);
                    world.body_mut(gid).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
                    for x in -1..=7 {
                        let w1 = world.spawn(BlockKind::Wall, IVec3::new(x, -1, 0), vec![IVec3::ZERO]);
                        world.body_mut(w1).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
                        let w2 = world.spawn(BlockKind::Wall, IVec3::new(x, 7, 0), vec![IVec3::ZERO]);
                        world.body_mut(w2).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
                    }
                    for y in 0..=6 {
                        let w1 = world.spawn(BlockKind::Wall, IVec3::new(-1, y, 0), vec![IVec3::ZERO]);
                        world.body_mut(w1).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
                        let w2 = world.spawn(BlockKind::Wall, IVec3::new(7, y, 0), vec![IVec3::ZERO]);
                        world.body_mut(w2).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
                    }
                    world.sync_grid();
                    game.engine = TurnEngine::new(world);
                    editor.selected_body_id = None;
                    editor.cached_solution = None;
                    editor.solver_status = "Idle".into();
                    editor.toast("Created new blank puzzle room.");
                }
                EditorAction::Save => {
                    let path = editor.current_level_path.clone();
                    let level_data = LevelData::from_world("Custom Level", &game.engine.world);
                    match level::save_level_to_file(&path, &level_data) {
                        Ok(_) => editor.toast(format!("Saved level to {}", path)),
                        Err(e) => editor.toast(format!("Save error: {}", e)),
                    }
                }
                EditorAction::SaveAs => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let new_path = format!("levels/puzzle_{}.json", timestamp);
                    let level_data = LevelData::from_world("Custom Level", &game.engine.world);
                    match level::save_level_to_file(&new_path, &level_data) {
                        Ok(_) => {
                            editor.current_level_path = new_path.clone();
                            editor.toast(format!("Saved new level to {}", new_path));
                        }
                        Err(e) => editor.toast(format!("Save error: {}", e)),
                    }
                }
                EditorAction::ToggleLevelsMenu => {
                    let files = level::list_level_files();
                    if files.is_empty() {
                        editor.toast("No saved levels found in levels/ directory.");
                    } else {
                        let curr_idx = files.iter().position(|p| p == &editor.current_level_path).unwrap_or(0);
                        let next_idx = (curr_idx + 1) % files.len();
                        let target_path = &files[next_idx];
                        if let Ok(lvl) = level::load_level_from_file(target_path) {
                            game.engine = TurnEngine::new(lvl.to_world());
                            editor.current_level_path = target_path.clone();
                            editor.selected_body_id = None;
                            editor.cached_solution = None;
                            editor.solver_status = "Idle".into();
                            editor.toast(format!("Loaded: {}", target_path));
                        }
                    }
                }
                EditorAction::AttemptSolve => {
                    let current_hash = compute_level_hash(&game.engine.world);
                    let world_clone = game.engine.world.clone();
                    let (tx, rx) = mpsc::channel();
                    editor.solver_rx = Some(Arc::new(Mutex::new(rx)));
                    editor.solving_hash = Some(current_hash);
                    editor.solver_status = "Solving in background (30s)...".into();
                    editor.toast("Dispatched solver background worker.");

                    std::thread::spawn(move || {
                        let config = SolverConfig {
                            timeout: Some(Duration::from_secs(30)),
                            ..default()
                        };
                        let result = solver::solve_with_config(world_clone, &config);
                        let _ = tx.send((current_hash, result));
                    });
                }
                EditorAction::TestPlay => {
                    editor.backup_world = Some(game.engine.world.clone());
                    next_mode.set(AppMode::Playtest);
                }
                EditorAction::TestWithSolution => {
                    let current_hash = compute_level_hash(&game.engine.world);
                    let cached_actions = editor
                        .cached_solution
                        .as_ref()
                        .filter(|(h, acts)| *h == current_hash && !acts.is_empty())
                        .map(|(_, acts)| acts.clone());

                    if let Some(actions) = cached_actions {
                        editor.backup_world = Some(game.engine.world.clone());
                        playback.is_playback = true;
                        playback.actions = actions;
                        playback.current_index = 0;
                        playback.auto_playing = true;
                        next_mode.set(AppMode::Playback);
                    } else {
                        editor.toast("No valid solution cached for current level. Click 'Attempt to Solve'.");
                    }
                }
            }
        }

        // 4. Inspector rotation buttons
        if rot_cw.is_some() {
            if let Some(id) = editor.selected_body_id {
                if let Some(body) = game.engine.world.body_mut(id) {
                    body.orientation = body.orientation.then(CubeRot::ROT_Z_270);
                    game.engine.world.sync_grid();
                    editor.cached_solution = None;
                    editor.toast("Rotated block 90° Clockwise.");
                }
            }
        }

        if rot_ccw.is_some() {
            if let Some(id) = editor.selected_body_id {
                if let Some(body) = game.engine.world.body_mut(id) {
                    body.orientation = body.orientation.then(CubeRot::ROT_Z_90);
                    game.engine.world.sync_grid();
                    editor.cached_solution = None;
                    editor.toast("Rotated block 90° Counter-Clockwise.");
                }
            }
        }

        // 5. Inspector reflection buttons (Flip X, Flip Y)
        if ref_x.is_some() {
            if let Some(id) = editor.selected_body_id {
                if let Some(body) = game.engine.world.body_mut(id) {
                    body.orientation = body.orientation.reflect_x();
                    game.engine.world.sync_grid();
                    editor.cached_solution = None;
                    editor.toast("Reflected block across X axis (Flip X).");
                }
            }
        }

        if ref_y.is_some() {
            if let Some(id) = editor.selected_body_id {
                if let Some(body) = game.engine.world.body_mut(id) {
                    body.orientation = body.orientation.reflect_y();
                    game.engine.world.sync_grid();
                    editor.cached_solution = None;
                    editor.toast("Reflected block across Y axis (Flip Y).");
                }
            }
        }

        // 6. Inspector toggle fixed property
        if toggle_fixed.is_some() {
            if let Some(id) = editor.selected_body_id {
                if let Some(body) = game.engine.world.body_mut(id) {
                    let (can_moveable, can_fixed) = editor.allowed_fixed_state(body.kind);
                    if can_moveable && can_fixed {
                        if body.is_fixed() {
                            body.tags.remove(TagKind::Fixed);
                            editor.toast("Changed property to Moveable.");
                        } else {
                            body.tags.set(TagKind::Fixed, TagValue::Unit);
                            editor.toast("Changed property to Stationary (Fixed).");
                        }
                        game.engine.world.sync_grid();
                        editor.cached_solution = None;
                    } else {
                        editor.toast("This block kind cannot change its stationary/moveable property.");
                    }
                }
            }
        }

        // 6. Inspector delete block button
        if del_btn.is_some() {
            if let Some(id) = editor.selected_body_id {
                game.engine.world.despawn(id);
                editor.selected_body_id = None;
                editor.cached_solution = None;
                editor.toast("Deleted selected block.");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Background Solver Receiver
// ---------------------------------------------------------------------------

fn background_solver_poll_system(
    mut editor: ResMut<EditorState>,
    game: Res<GameState>,
) {
    let result_pair = if let Some(rx_arc) = &editor.solver_rx {
        if let Ok(rx) = rx_arc.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    } else {
        None
    };

    if let Some((solved_hash, result)) = result_pair {
        editor.solver_rx = None;
        let current_hash = compute_level_hash(&game.engine.world);

        if solved_hash == current_hash {
            if result.is_solved() {
                editor.solver_status = format!(
                    "✓ Solved in {} steps ({:.2?})",
                    result.actions.len(),
                    result.duration
                );
                editor.cached_solution = Some((current_hash, result.actions.clone()));
                let msg = format!("Solver: Found {}-step solution!", result.actions.len());
                editor.toast(msg);
            } else {
                editor.solver_status = format!("✗ No Solution ({:.2?})", result.duration);
                editor.cached_solution = None;
                editor.toast("Solver: Proved no solution exists within depth/timeout.");
            }
        } else {
            editor.solver_status = "Level modified during solve. Invalidated.".into();
            editor.cached_solution = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Selection and Hover Gizmos
// ---------------------------------------------------------------------------

fn draw_editor_selection_gizmos(
    app_mode: Res<State<AppMode>>,
    editor: Res<EditorState>,
    game: Res<GameState>,
    mut gizmos: Gizmos,
) {
    if *app_mode.get() != AppMode::Editor {
        return;
    }

    // 1. Draw hovered cell highlight outline
    if let Some(cell) = editor.hovered_cell {
        let x = cell.x as f32;
        let z = -cell.y as f32;
        let y = 0.05;
        let d = 0.48;
        let col = Color::srgba(1.0, 0.9, 0.2, 0.85);

        gizmos.line(Vec3::new(x - d, y, z - d), Vec3::new(x + d, y, z - d), col);
        gizmos.line(Vec3::new(x + d, y, z - d), Vec3::new(x + d, y, z + d), col);
        gizmos.line(Vec3::new(x + d, y, z + d), Vec3::new(x - d, y, z + d), col);
        gizmos.line(Vec3::new(x - d, y, z + d), Vec3::new(x - d, y, z - d), col);
    }

    // 2. Draw selected body bounding box
    if let Some(id) = editor.selected_body_id {
        if let Some(body) = game.engine.world.body(id) {
            let col = Color::srgba(0.3, 0.8, 1.0, 0.95);
            for &cell in &body.world_cells() {
                let x = cell.x as f32;
                let z = -cell.y as f32;
                let y0 = -0.45;
                let y1 = 0.45;
                let d = 0.52;

                // Bottom square
                gizmos.line(Vec3::new(x - d, y0, z - d), Vec3::new(x + d, y0, z - d), col);
                gizmos.line(Vec3::new(x + d, y0, z - d), Vec3::new(x + d, y0, z + d), col);
                gizmos.line(Vec3::new(x + d, y0, z + d), Vec3::new(x - d, y0, z + d), col);
                gizmos.line(Vec3::new(x - d, y0, z + d), Vec3::new(x - d, y0, z - d), col);

                // Top square
                gizmos.line(Vec3::new(x - d, y1, z - d), Vec3::new(x + d, y1, z - d), col);
                gizmos.line(Vec3::new(x + d, y1, z - d), Vec3::new(x + d, y1, z + d), col);
                gizmos.line(Vec3::new(x + d, y1, z + d), Vec3::new(x - d, y1, z + d), col);
                gizmos.line(Vec3::new(x - d, y1, z + d), Vec3::new(x - d, y1, z - d), col);

                // Vertical edges
                gizmos.line(Vec3::new(x - d, y0, z - d), Vec3::new(x - d, y1, z - d), col);
                gizmos.line(Vec3::new(x + d, y0, z - d), Vec3::new(x + d, y1, z - d), col);
                gizmos.line(Vec3::new(x + d, y0, z + d), Vec3::new(x + d, y1, z + d), col);
                gizmos.line(Vec3::new(x - d, y0, z + d), Vec3::new(x - d, y1, z + d), col);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Keyboard Shortcuts & Mode Returning
// ---------------------------------------------------------------------------

fn editor_keyboard_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    app_mode: Res<State<AppMode>>,
    mut next_mode: ResMut<NextState<AppMode>>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
    mut playback: ResMut<crate::PlaybackState>,
) {
    // Return to Editor from Playtest or Playback mode with Escape
    if keys.just_pressed(KeyCode::Escape) {
        if *app_mode.get() != AppMode::Editor {
            if let Some(backup) = editor.backup_world.take() {
                game.engine = TurnEngine::new(backup);
            }
            playback.is_playback = false;
            next_mode.set(AppMode::Editor);
            editor.toast("Returned to Level Editor.");
            return;
        } else if editor.selected_body_id.is_some() {
            editor.selected_body_id = None;
            editor.toast("Deselected block.");
        }
    }

    // Delete selected block with Delete / Backspace
    if *app_mode.get() == AppMode::Editor && (keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace)) {
        if let Some(id) = editor.selected_body_id {
            game.engine.world.despawn(id);
            editor.selected_body_id = None;
            editor.cached_solution = None;
            editor.toast("Deleted selected block.");
        }
    }
}

fn toast_decay_system(time: Res<Time>, mut editor: ResMut<EditorState>) {
    if let Some((_, timer)) = &mut editor.status_message {
        timer.tick(time.delta());
        if timer.is_finished() {
            editor.status_message = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_move_body_validation() {
        let mut world = World::new();
        let b1 = world.spawn(BlockKind::Mirror, IVec3::new(1, 1, 0), vec![IVec3::ZERO]);
        let _b2 = world.spawn(BlockKind::Wall, IVec3::new(2, 1, 0), vec![IVec3::ZERO]);

        // Moving b1 to free cell (1, 2, 0) should be allowed
        assert!(can_move_body_to(&world, b1, IVec3::new(1, 2, 0)));

        // Moving b1 to cell occupied by b2 (2, 1, 0) should be prevented
        assert!(!can_move_body_to(&world, b1, IVec3::new(2, 1, 0)));

        // Moving b1 to its own current cell (1, 1, 0) is valid
        assert!(can_move_body_to(&world, b1, IVec3::new(1, 1, 0)));
    }
}
