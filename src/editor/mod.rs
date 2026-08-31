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
use glam::IVec3;

use crate::block_types::BlockKind;
use crate::level::{self, compute_level_hash, LevelData};
use crate::sim::{BodyId, TagKind, TagValue, World};
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
    ToggleFloorplanModal,
    ToggleFramePreview,
    RotateViewCcw,
    RotateViewCw,
    AttemptSolve,
    TestPlay,
    TestWithSolution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZPlacementMode {
    #[default]
    StackOnTop,
    FixedLayer,
}

// ---------------------------------------------------------------------------
// Editor State Resource
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct EditorState {
    /// Currently selected base block kind from palette (None = Select-Only mode).
    pub selected_kind: Option<BlockKind>,
    /// Property toggle: true = stationary (fixed), false = moveable.
    pub is_fixed: bool,
    /// Active Z placement mode (StackOnTop vs FixedLayer).
    pub z_mode: ZPlacementMode,
    /// Active Z layer for FixedLayer mode (default 0).
    pub current_z: i32,
    /// Currently selected bodies on grid for Inspector.
    pub selected_body_ids: Vec<BodyId>,
    /// Currently dragging body ID on grid (for StackOnTop mode).
    pub dragging_body_id: Option<BodyId>,
    /// Screen start coordinate for drag box selection in FixedLayer mode.
    pub box_select_start: Option<Vec2>,
    /// Whether box selection is currently active.
    pub box_select_active: bool,
    /// Set of locked Z layers (blocks on these layers are not selectable/editable).
    pub locked_z_layers: std::collections::HashSet<i32>,
    /// Whether the floorplan dialog is currently open.
    pub floorplan_open: bool,
    /// Level width for floorplan fill.
    pub floorplan_width: i32,
    /// Level height for floorplan fill.
    pub floorplan_height: i32,
    /// Target Z layer for floorplan fill.
    pub floorplan_z: i32,
    /// Whether Frame 1 laser preview is enabled in editor.
    pub show_frame1_preview: bool,
    /// Hovered grid coordinate (X, Y, Z) in simulation space.
    pub hovered_cell: Option<IVec3>,
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
            selected_kind: None, // Default to Select Mode [S]
            is_fixed: false,
            z_mode: ZPlacementMode::StackOnTop,
            current_z: 0,
            selected_body_ids: Vec::new(),
            dragging_body_id: None,
            box_select_start: None,
            box_select_active: false,
            locked_z_layers: std::collections::HashSet::new(),
            floorplan_open: false,
            floorplan_width: 10,
            floorplan_height: 10,
            floorplan_z: -1,
            show_frame1_preview: true,
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
    /// Return the stage ground level Z (where items sit on top of the floor).
    pub fn stage_ground_z(&self) -> i32 {
        self.floorplan_z + 1
    }

    /// Return the primary selected body ID (first selected block).
    #[allow(dead_code)]
    pub fn primary_selection(&self) -> Option<BodyId> {
        self.selected_body_ids.first().copied()
    }

    /// Check if a specific body is currently selected.
    pub fn is_selected(&self, id: BodyId) -> bool {
        self.selected_body_ids.contains(&id)
    }

    /// Select a single body and clear previous selections.
    pub fn select_single(&mut self, id: BodyId) {
        self.selected_body_ids.clear();
        self.selected_body_ids.push(id);
    }

    /// Toggle a body in or out of the current selection set.
    pub fn toggle_selection(&mut self, id: BodyId) {
        if let Some(pos) = self.selected_body_ids.iter().position(|&x| x == id) {
            self.selected_body_ids.remove(pos);
        } else {
            self.selected_body_ids.push(id);
        }
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        self.selected_body_ids.clear();
    }

    /// Check if a Z layer is locked.
    pub fn is_layer_locked(&self, z: i32) -> bool {
        self.locked_z_layers.contains(&z)
    }

    /// Allowed (moveable, fixed) property configuration for a given block kind.
    pub fn allowed_fixed_state(&self, kind: BlockKind) -> (bool, bool) {
        match kind {
            BlockKind::Wall | BlockKind::Floor => (false, true),
            BlockKind::Player => (true, false),
            BlockKind::Mirror | BlockKind::LaserSource | BlockKind::Pushable
                | BlockKind::Goal | BlockKind::Glass => (true, true),
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
                    ui::update_editor_status_and_modal_ui_system,
                    update_palette_3d_preview,
                    background_solver_poll_system,
                    draw_editor_selection_gizmos,
                    editor_keyboard_shortcuts_system,
                    toast_decay_system,
                ),
            )
            .add_systems(
                Update,
                (
                    editor_grid_interaction_system,
                    editor_button_clicks_system,
                )
                    .run_if(in_state(AppMode::Editor)),
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

        let Some(selected_kind) = editor.selected_kind else {
            *vis = Visibility::Hidden;
            continue;
        };
        *vis = Visibility::Visible;

        // Rotate slowly around Y
        transform.rotate_local_y(1.2 * time.delta_secs());

        let (can_moveable, can_fixed) = editor.allowed_fixed_state(selected_kind);
        let is_moveable = if !can_moveable {
            false
        } else if !can_fixed {
            true
        } else {
            !editor.is_fixed
        };

        let (target_mesh, target_mat) = match selected_kind {
            BlockKind::Player => (render_assets.player_mesh.clone(), render_assets.player_mat.clone()),
            BlockKind::Goal => {
                let m = if is_moveable {
                    render_assets.moveable_goal_mat.clone()
                } else {
                    render_assets.goal_mat.clone()
                };
                let mesh = if is_moveable {
                    render_assets.rounded_pyramid_mesh.clone()
                } else {
                    render_assets.pyramid_mesh.clone()
                };
                (mesh, m)
            }
            BlockKind::Wall => (render_assets.cube_mesh.clone(), render_assets.fixed_wall_mat.clone()),
            BlockKind::Floor => (render_assets.cube_mesh.clone(), render_assets.floor_mat.clone()),
            BlockKind::Glass => {
                let m = if is_moveable {
                    render_assets.moveable_glass_mat.clone()
                } else {
                    render_assets.fixed_glass_mat.clone()
                };
                let mesh = if is_moveable {
                    render_assets.rounded_cube_mesh.clone()
                } else {
                    render_assets.cube_mesh.clone()
                };
                (mesh, m)
            }
            BlockKind::Pushable => {
                let m = if is_moveable {
                    render_assets.moveable_pushable_mat.clone()
                } else {
                    render_assets.fixed_pushable_mat.clone()
                };
                let mesh = if is_moveable {
                    render_assets.rounded_cube_mesh.clone()
                } else {
                    render_assets.cube_mesh.clone()
                };
                (mesh, m)
            }
            BlockKind::Mirror => {
                let m = if is_moveable {
                    render_assets.moveable_mirror_mat.clone()
                } else {
                    render_assets.fixed_mirror_mat.clone()
                };
                let mesh = if is_moveable {
                    render_assets.rounded_mirror_mesh.clone()
                } else {
                    render_assets.mirror_mesh.clone()
                };
                (mesh, m)
            }
            BlockKind::LaserSource => {
                let m = if is_moveable {
                    render_assets.moveable_laser_mat.clone()
                } else {
                    render_assets.fixed_laser_mat.clone()
                };
                let mesh = if is_moveable {
                    render_assets.rounded_cube_mesh.clone()
                } else {
                    render_assets.cube_mesh.clone()
                };
                (mesh, m)
            }
        };

        mesh.0 = target_mesh;
        mat.0 = target_mat;
    }
}

// ---------------------------------------------------------------------------
// 3D Grid Raycasting & Mouse Interaction
// ---------------------------------------------------------------------------

fn ray_intersect_aabb(
    ray_origin: Vec3,
    ray_dir: Vec3,
    min: Vec3,
    max: Vec3,
) -> Option<f32> {
    let inv_d = Vec3::new(
        if ray_dir.x.abs() > 1e-6 { 1.0 / ray_dir.x } else { f32::INFINITY },
        if ray_dir.y.abs() > 1e-6 { 1.0 / ray_dir.y } else { f32::INFINITY },
        if ray_dir.z.abs() > 1e-6 { 1.0 / ray_dir.z } else { f32::INFINITY },
    );

    let t0 = (min - ray_origin) * inv_d;
    let t1 = (max - ray_origin) * inv_d;

    let tmin_v = t0.min(t1);
    let tmax_v = t0.max(t1);

    let tmin = tmin_v.x.max(tmin_v.y).max(tmin_v.z);
    let tmax = tmax_v.x.min(tmax_v.y).min(tmax_v.z);

    if tmin <= tmax && tmax >= 0.0 {
        Some(if tmin >= 0.0 { tmin } else { tmax })
    } else {
        None
    }
}

fn raycast_plane_at_z(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
    z_level: i32,
) -> Option<IVec3> {
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    let ray_dir: Vec3 = ray.direction.into();
    if ray_dir.y.abs() < 1e-5 {
        return None;
    }
    // In Bevy coordinates, Sim Z is along the Y axis: Y_bevy = z_level as f32
    let target_y = z_level as f32;
    let t = (target_y - ray.origin.y) / ray_dir.y;
    if t < 0.0 {
        return None;
    }
    let world_pt = ray.origin + ray_dir * t;
    let gx = (world_pt.x + 0.5).floor() as i32;
    let gy = (-world_pt.z + 0.5).floor() as i32;
    Some(IVec3::new(gx, gy, z_level))
}

fn raycast_stack_on_top(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
    world: &World,
    ground_z: i32,
    ignore_body: Option<BodyId>,
) -> Option<(IVec3, Option<BodyId>)> {
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    let ray_dir: Vec3 = ray.direction.into();

    let mut closest: Option<(f32, IVec3, BodyId)> = None;

    for body in world.bodies() {
        if Some(body.id) == ignore_body {
            continue;
        }
        for world_cell in body.world_cells() {
            // Bevy coordinate conversion: (world_cell.x, world_cell.z, -world_cell.y)
            let center = Vec3::new(
                world_cell.x as f32,
                world_cell.z as f32,
                -world_cell.y as f32,
            );
            let half = Vec3::splat(0.5);
            let min = center - half;
            let max = center + half;

            if let Some(t) = ray_intersect_aabb(ray.origin, ray_dir, min, max) {
                if closest.is_none() || t < closest.unwrap().0 {
                    closest = Some((t, world_cell, body.id));
                }
            }
        }
    }

    if let Some((_t, hit_cell, hit_body_id)) = closest {
        // Find highest occupied Z coordinate among all bodies at column (hit_cell.x, hit_cell.y)
        let mut max_z = hit_cell.z;
        for body in world.bodies() {
            if Some(body.id) == ignore_body {
                continue;
            }
            for cell in body.world_cells() {
                if cell.x == hit_cell.x && cell.y == hit_cell.y && cell.z > max_z {
                    max_z = cell.z;
                }
            }
        }
        let target_cell = IVec3::new(hit_cell.x, hit_cell.y, max_z + 1);
        Some((target_cell, Some(hit_body_id)))
    } else {
        // No block hit directly -> raycast ground plane at ground_z and check if column has blocks
        let ground_cell = raycast_plane_at_z(camera, camera_transform, cursor_pos, ground_z)?;
        let mut max_z = ground_z - 1;
        let mut found_body = None;
        for body in world.bodies() {
            if Some(body.id) == ignore_body {
                continue;
            }
            for cell in body.world_cells() {
                if cell.x == ground_cell.x && cell.y == ground_cell.y && cell.z > max_z {
                    max_z = cell.z;
                    found_body = Some(body.id);
                }
            }
        }
        if max_z >= ground_z {
            Some((IVec3::new(ground_cell.x, ground_cell.y, max_z + 1), found_body))
        } else {
            Some((IVec3::new(ground_cell.x, ground_cell.y, ground_z), None))
        }
    }
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

/// Fill a rectangular floor of `width` x `height` blocks at layer `z`.
pub fn fill_floorplan(world: &mut World, width: i32, height: i32, z: i32) {
    use crate::sim::{TagKind, TagValue};
    // Remove existing Floor blocks on this layer
    let to_remove: Vec<BodyId> = world
        .bodies()
        .iter()
        .filter(|b| b.kind == BlockKind::Floor && b.anchor.z == z)
        .map(|b| b.id)
        .collect();
    for id in to_remove {
        world.despawn(id);
    }
    // Spawn floor tiles across 0..width, 0..height
    for x in 0..width {
        for y in 0..height {
            let id = world.spawn(BlockKind::Floor, IVec3::new(x, y, z), vec![IVec3::ZERO]);
            if let Some(b) = world.body_mut(id) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
        }
    }
    world.sync_grid();
}

fn editor_grid_interaction_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<camera::MainCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // 1. Multi-block transform shortcuts for all selected blocks
    if !editor.selected_body_ids.is_empty() {
        let ids = editor.selected_body_ids.clone();
        let mut modified = false;

        if keyboard.just_pressed(KeyCode::KeyR) {
            for &id in &ids {
                if let Some(b) = game.engine.world.body_mut(id) {
                    b.orientation = b.orientation.rot_world_z_cw();
                    modified = true;
                }
            }
            if modified {
                editor.toast("Rotated selected block(s) CW around Z axis [R].");
            }
        } else if keyboard.just_pressed(KeyCode::KeyT) {
            for &id in &ids {
                if let Some(b) = game.engine.world.body_mut(id) {
                    b.orientation = b.orientation.rot_world_x_pos();
                    modified = true;
                }
            }
            if modified {
                editor.toast("Pitched selected block(s) +90° around X axis [T].");
            }
        } else if keyboard.just_pressed(KeyCode::KeyG) {
            for &id in &ids {
                if let Some(b) = game.engine.world.body_mut(id) {
                    b.orientation = b.orientation.rot_world_y_pos();
                    modified = true;
                }
            }
            if modified {
                editor.toast("Rolled selected block(s) +90° around Y axis [G].");
            }
        } else if keyboard.just_pressed(KeyCode::KeyX) {
            for &id in &ids {
                if let Some(b) = game.engine.world.body_mut(id) {
                    b.orientation = b.orientation.reflect_x();
                    modified = true;
                }
            }
            if modified {
                editor.toast("Flipped selected block(s) across X axis [X].");
            }
        } else if keyboard.just_pressed(KeyCode::KeyY) {
            for &id in &ids {
                if let Some(b) = game.engine.world.body_mut(id) {
                    b.orientation = b.orientation.reflect_y();
                    modified = true;
                }
            }
            if modified {
                editor.toast("Flipped selected block(s) across Y axis [Y].");
            }
        } else if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
            for &id in &ids {
                game.engine.world.despawn(id);
            }
            editor.clear_selection();
            modified = true;
            editor.toast("Deleted selected block(s).");
        }

        if modified {
            game.engine.world.sync_grid();
            let new_world = game.engine.world.clone();
            game.engine.update_authoring_world(new_world);
            editor.cached_solution = None;
        }
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        editor.hovered_cell = None;
        return;
    };

    // Ignore clicks if cursor is over sidebars, top bar, or floorplan modal
    if cursor_pos.x < 260.0 && cursor_pos.y > 55.0 {
        editor.hovered_cell = None;
        return;
    }
    if cursor_pos.x > (window.width() - 260.0) && cursor_pos.y > 55.0 && !editor.selected_body_ids.is_empty() {
        editor.hovered_cell = None;
        return;
    }
    if cursor_pos.y < 55.0 {
        editor.hovered_cell = None;
        return;
    }
    if editor.floorplan_open && cursor_pos.x >= 260.0 && cursor_pos.x <= 550.0 && cursor_pos.y >= 55.0 && cursor_pos.y <= 360.0 {
        editor.hovered_cell = None;
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    let ignore_id = editor.dragging_body_id;
    let (target_cell, raw_hit_body_id) = match editor.z_mode {
        ZPlacementMode::StackOnTop => {
            let Some((cell, body_id)) = raycast_stack_on_top(camera, camera_transform, cursor_pos, &game.engine.world, editor.stage_ground_z(), ignore_id) else {
                editor.hovered_cell = None;
                return;
            };
            (cell, body_id)
        }
        ZPlacementMode::FixedLayer => {
            let Some(cell) = raycast_plane_at_z(camera, camera_transform, cursor_pos, editor.current_z) else {
                editor.hovered_cell = None;
                return;
            };
            let body_id = game.engine.world.body_at(cell).map(|b| b.id);
            (cell, body_id)
        }
    };

    // Filter locked Z layers from selection
    let clicked_body_id = raw_hit_body_id.filter(|&id| {
        if let Some(b) = game.engine.world.body(id) {
            !editor.is_layer_locked(b.anchor.z)
        } else {
            false
        }
    });

    editor.hovered_cell = Some(target_cell);
    let cell_pos = target_cell;

    // 2. Left Click: Selection / Placement / Dragging
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(kind) = editor.selected_kind {
            // ===============================================================
            // PLACEMENT MODE: Always place / stack block, NEVER drag
            // ===============================================================
            let place_target = if editor.z_mode == ZPlacementMode::FixedLayer {
                IVec3::new(cell_pos.x, cell_pos.y, editor.current_z)
            } else {
                target_cell
            };

            if !editor.is_layer_locked(place_target.z) {
                let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
                let is_fixed = if !can_moveable { true } else if !can_fixed { false } else { editor.is_fixed };

                if kind == BlockKind::Player {
                    if let Some(player_id) = game.engine.world.player_id() {
                        if let Some(p) = game.engine.world.body_mut(player_id) {
                            p.anchor = place_target;
                            game.engine.world.sync_grid();
                            let new_world = game.engine.world.clone();
                            game.engine.update_authoring_world(new_world);
                            editor.select_single(player_id);
                            editor.dragging_body_id = None;
                            editor.toast(format!("Relocated player character to ({}, {}, {}).", place_target.x, place_target.y, place_target.z));
                            editor.cached_solution = None;
                            return;
                        }
                    }
                }

                let new_id = game.engine.world.spawn(kind, place_target, vec![IVec3::ZERO]);
                if let Some(b) = game.engine.world.body_mut(new_id) {
                    if is_fixed {
                        b.tags.set(TagKind::Fixed, TagValue::Unit);
                    }
                }
                game.engine.world.sync_grid();
                let new_world = game.engine.world.clone();
                game.engine.update_authoring_world(new_world);
                editor.select_single(new_id);
                editor.dragging_body_id = None;
                editor.cached_solution = None;
                if editor.z_mode == ZPlacementMode::StackOnTop && place_target.z > editor.stage_ground_z() {
                    editor.toast(format!("Stacked {:?} on top at ({}, {}, {})", kind, place_target.x, place_target.y, place_target.z));
                } else {
                    editor.toast(format!("Placed {:?} at ({}, {}, {})", kind, place_target.x, place_target.y, place_target.z));
                }
            }
        } else {
            // ===============================================================
            // SELECT MODE: Click/Drag to select/move existing blocks
            // ===============================================================
            if editor.z_mode == ZPlacementMode::FixedLayer {
                // Fixed Layer Mode
                editor.box_select_start = Some(cursor_pos);
                editor.box_select_active = false;

                if let Some(body_id) = clicked_body_id {
                    if shift_held {
                        editor.toggle_selection(body_id);
                        let total = editor.selected_body_ids.len();
                        editor.toast(format!("Toggled selection (Total: {}).", total));
                    } else {
                        editor.select_single(body_id);
                        editor.toast(format!("Selected block (ID {}).", body_id.0));
                    }
                } else if !shift_held {
                    editor.clear_selection();
                }
            } else {
                // StackOnTop Select Mode
                if let Some(body_id) = clicked_body_id {
                    if shift_held {
                        editor.toggle_selection(body_id);
                        let total = editor.selected_body_ids.len();
                        editor.toast(format!("Toggled selection (Total: {}).", total));
                    } else {
                        editor.select_single(body_id);
                        editor.dragging_body_id = Some(body_id);
                        editor.toast(format!("Selected block (ID {}).", body_id.0));
                    }
                } else if !shift_held {
                    editor.clear_selection();
                    editor.dragging_body_id = None;
                }
            }
        }
    } else if mouse_button.pressed(MouseButton::Left) {
        if editor.selected_kind.is_none() {
            if editor.z_mode == ZPlacementMode::FixedLayer {
                if let Some(start) = editor.box_select_start {
                    if cursor_pos.distance(start) > 6.0 {
                        editor.box_select_active = true;
                    }
                }
            } else if let Some(drag_id) = editor.dragging_body_id {
                // StackOnTop drag single block
                if let Some(body) = game.engine.world.body(drag_id) {
                    if body.anchor != cell_pos && can_move_body_to(&game.engine.world, drag_id, cell_pos) {
                        if let Some(b) = game.engine.world.body_mut(drag_id) {
                            b.anchor = cell_pos;
                        }
                        game.engine.world.sync_grid();
                        let new_world = game.engine.world.clone();
                        game.engine.update_authoring_world(new_world);
                        editor.cached_solution = None;
                    }
                }
            }
        }
    }

    // 3. Release Left Click
    if mouse_button.just_released(MouseButton::Left) {
        if editor.z_mode == ZPlacementMode::FixedLayer {
            if editor.box_select_active {
                if let (Some(start), Ok((camera, camera_transform))) = (editor.box_select_start.take(), camera_query.single()) {
                    if let Some(start_cell) = raycast_plane_at_z(camera, camera_transform, start, editor.current_z) {
                        let min_x = start_cell.x.min(cell_pos.x);
                        let max_x = start_cell.x.max(cell_pos.x);
                        let min_y = start_cell.y.min(cell_pos.y);
                        let max_y = start_cell.y.max(cell_pos.y);

                        let mut boxed_ids = Vec::new();
                        for body in game.engine.world.bodies() {
                            if body.anchor.z == editor.current_z && !editor.is_layer_locked(body.anchor.z) {
                                if body.anchor.x >= min_x && body.anchor.x <= max_x && body.anchor.y >= min_y && body.anchor.y <= max_y {
                                    boxed_ids.push(body.id);
                                }
                            }
                        }

                        if shift_held {
                            for id in boxed_ids {
                                if !editor.is_selected(id) {
                                    editor.selected_body_ids.push(id);
                                }
                            }
                        } else {
                            editor.selected_body_ids = boxed_ids;
                        }
                        let total = editor.selected_body_ids.len();
                        editor.toast(format!("Box selected {} block(s).", total));
                    }
                }
            }
            editor.box_select_start = None;
            editor.box_select_active = false;
        } else if let Some(drag_id) = editor.dragging_body_id.take() {
            if let Some(body) = game.engine.world.body(drag_id) {
                editor.toast(format!("Moved {:?} to ({}, {}, {})", body.kind, body.anchor.x, body.anchor.y, body.anchor.z));
            }
        }
    }

    // 4. Right Click: Delete Block
    if mouse_button.just_pressed(MouseButton::Right) {
        let to_delete = if editor.z_mode == ZPlacementMode::FixedLayer {
            game.engine.world.body_at(cell_pos).map(|b| b.id)
        } else {
            clicked_body_id.or_else(|| game.engine.world.body_at(cell_pos).map(|b| b.id))
        };

        if let Some(id) = to_delete {
            if let Some(body) = game.engine.world.body(id) {
                if !editor.is_layer_locked(body.anchor.z) {
                    game.engine.world.despawn(id);
                    editor.selected_body_ids.retain(|&x| x != id);
                    if editor.dragging_body_id == Some(id) {
                        editor.dragging_body_id = None;
                    }
                    game.engine.world.sync_grid();
                    let new_world = game.engine.world.clone();
                    game.engine.update_authoring_world(new_world);
                    editor.cached_solution = None;
                    editor.toast(format!("Deleted block at ({}, {}, {})", cell_pos.x, cell_pos.y, cell_pos.z));
                }
            }
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
            (
                Option<&ui::RotateCwButton>,
                Option<&ui::RotateCcwButton>,
                Option<&ui::RotateXPosButton>,
                Option<&ui::RotateXNegButton>,
                Option<&ui::RotateYPosButton>,
                Option<&ui::RotateYNegButton>,
            ),
            (
                Option<&ui::ReflectXButton>,
                Option<&ui::ReflectYButton>,
                Option<&ui::ToggleFixedButton>,
                Option<&ui::CombineButton>,
                Option<&ui::UncombineButton>,
                Option<&ui::DeleteBlockButton>,
            ),
            (
                Option<&ui::ZModeToggleButton>,
                Option<&ui::ZLayerIncButton>,
                Option<&ui::ZLayerDecButton>,
            ),
            (
                Option<&ui::FloorplanWidthDecBtn>,
                Option<&ui::FloorplanWidthIncBtn>,
                Option<&ui::FloorplanHeightDecBtn>,
                Option<&ui::FloorplanHeightIncBtn>,
                Option<&ui::FloorplanZDecBtn>,
                Option<&ui::FloorplanZIncBtn>,
            ),
            (
                Option<&ui::FloorplanFillBtn>,
                Option<&ui::FloorplanLockToggleBtn>,
                Option<&ui::FloorplanCloseBtn>,
            ),
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut camera_query: Query<&mut camera::CameraController, With<camera::MainCamera>>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<AppMode>>,
    mut playback: ResMut<crate::PlaybackState>,
) {
    for (
        interaction,
        palette_btn,
        prop_btn,
        action_btn,
        (rot_cw, rot_ccw, rot_x_pos, rot_x_neg, rot_y_pos, rot_y_neg),
        (ref_x, ref_y, toggle_fixed, combine_btn, uncombine_btn, del_btn),
        (z_mode_btn, z_inc_btn, z_dec_btn),
        (fp_w_dec, fp_w_inc, fp_h_dec, fp_h_inc, fp_z_dec, fp_z_inc),
        (fp_fill, fp_lock_toggle, fp_close),
    ) in &mut interaction_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 1. Palette block selection
        if let Some(btn) = palette_btn {
            if let Some(kind) = btn.0 {
                if editor.selected_kind == Some(kind) {
                    editor.selected_kind = None;
                    editor.toast("Switched to Select Mode [S / Esc].");
                } else {
                    editor.selected_kind = Some(kind);
                    let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
                    if !can_moveable {
                        editor.is_fixed = true;
                    } else if !can_fixed {
                        editor.is_fixed = false;
                    }
                    let label = format!("Selected tool: {:?} (Click in viewport to place/stack)", kind);
                    editor.toast(label);
                }
            } else {
                // [S] Select button clicked
                editor.selected_kind = None;
                editor.toast("Select Mode [S] active. Click or drag blocks to select.");
            }
        }

        // 2. Property toggle (Moveable vs Stationary)
        if let Some(btn) = prop_btn {
            if let Some(kind) = editor.selected_kind {
                let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
                if btn.0 && can_fixed {
                    editor.is_fixed = true;
                    editor.toast("Tool property: Stationary (Fixed)");
                } else if !btn.0 && can_moveable {
                    editor.is_fixed = false;
                    editor.toast("Tool property: Moveable");
                }
            } else {
                editor.toast("Select a block tool from the palette first.");
            }
        }

        // 3. Z placement mode toggle
        if let Some(btn) = z_mode_btn {
            editor.z_mode = btn.0;
            let current_z = editor.current_z;
            match btn.0 {
                ZPlacementMode::StackOnTop => editor.toast("Z Mode: Stack on Top."),
                ZPlacementMode::FixedLayer => editor.toast(format!("Z Mode: Fixed Layer (Z={}).", current_z)),
            }
        }

        // 4. Z layer increment / decrement
        if z_inc_btn.is_some() {
            let next_z = (editor.current_z + 1).min(20);
            editor.current_z = next_z;
            editor.toast(format!("Z Layer: {}", next_z));
        }
        if z_dec_btn.is_some() {
            let next_z = (editor.current_z - 1).max(-5);
            editor.current_z = next_z;
            editor.toast(format!("Z Layer: {}", next_z));
        }

        // 5. Floorplan modal controls
        if fp_w_dec.is_some() {
            editor.floorplan_width = (editor.floorplan_width - 1).max(1);
        }
        if fp_w_inc.is_some() {
            editor.floorplan_width = (editor.floorplan_width + 1).min(50);
        }
        if fp_h_dec.is_some() {
            editor.floorplan_height = (editor.floorplan_height - 1).max(1);
        }
        if fp_h_inc.is_some() {
            editor.floorplan_height = (editor.floorplan_height + 1).min(50);
        }
        if fp_z_dec.is_some() {
            editor.floorplan_z = (editor.floorplan_z - 1).max(-10);
        }
        if fp_z_inc.is_some() {
            editor.floorplan_z = (editor.floorplan_z + 1).min(20);
        }
        if fp_fill.is_some() {
            let w = editor.floorplan_width;
            let h = editor.floorplan_height;
            let z = editor.floorplan_z;
            fill_floorplan(&mut game.engine.world, w, h, z);
            let new_world = game.engine.world.clone();
            game.engine.update_authoring_world(new_world);
            editor.cached_solution = None;
            editor.toast(format!("Filled {}x{} Floor blocks at Layer Z={}.", w, h, z));
        }
        if fp_lock_toggle.is_some() {
            let z = editor.floorplan_z;
            if editor.locked_z_layers.contains(&z) {
                editor.locked_z_layers.remove(&z);
                editor.toast(format!("Unlocked Layer Z={}.", z));
            } else {
                editor.locked_z_layers.insert(z);
                editor.toast(format!("Locked Layer Z={} (blocks cannot be selected/edited).", z));
            }
        }
        if fp_close.is_some() {
            editor.floorplan_open = false;
        }

        // 6. Top Action buttons
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
                    editor.clear_selection();
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
                            editor.clear_selection();
                            editor.cached_solution = None;
                            editor.solver_status = "Idle".into();
                            editor.toast(format!("Loaded: {}", target_path));
                        }
                    }
                }
                EditorAction::ToggleFloorplanModal => {
                    editor.floorplan_open = !editor.floorplan_open;
                }
                EditorAction::ToggleFramePreview => {
                    let next_show = !editor.show_frame1_preview;
                    editor.show_frame1_preview = next_show;
                    game.engine.refresh_preview();
                    editor.toast(format!("Frame 1 preview: {}", if next_show { "ON" } else { "OFF" }));
                }
                EditorAction::RotateViewCcw => {
                    for mut controller in &mut camera_query {
                        controller.target_yaw += std::f32::consts::FRAC_PI_2;
                    }
                    editor.toast("Rotated level view 90° CCW (Key: Q).");
                }
                EditorAction::RotateViewCw => {
                    for mut controller in &mut camera_query {
                        controller.target_yaw -= std::f32::consts::FRAC_PI_2;
                    }
                    editor.toast("Rotated level view 90° CW (Key: E).");
                }
                EditorAction::AttemptSolve => {
                    if !game.engine.is_valid() {
                        editor.solver_status = "Invalid Level".into();
                        editor.toast("Cannot solve: Level has spontaneous movement at Frame 1.");
                        return;
                    }
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
                    match game.engine.start_playtest() {
                        Ok(()) => {
                            editor.backup_world = Some(game.engine.frame_zero_star.clone());
                            next_mode.set(AppMode::Playtest);
                        }
                        Err(err) => {
                            editor.toast(format!("Cannot playtest: {}", err));
                        }
                    }
                }
                EditorAction::TestWithSolution => {
                    if !game.engine.is_valid() {
                        editor.toast("Cannot play solution: level has spontaneous movement.");
                        return;
                    }
                    let current_hash = compute_level_hash(&game.engine.world);
                    let cached_actions = editor
                        .cached_solution
                        .as_ref()
                        .filter(|(h, acts)| *h == current_hash && !acts.is_empty())
                        .map(|(_, acts)| acts.clone());

                    if let Some(actions) = cached_actions {
                        if game.engine.start_playtest().is_ok() {
                            editor.backup_world = Some(game.engine.frame_zero_star.clone());
                            playback.is_playback = true;
                            playback.actions = actions;
                            playback.current_index = 0;
                            playback.auto_playing = true;
                            next_mode.set(AppMode::Playback);
                        }
                    } else {
                        editor.toast("No valid solution cached for current level. Click 'Solve Level'.");
                    }
                }
            }
        }

        // 7. Inspector transforms for all selected bodies
        if !editor.selected_body_ids.is_empty() {
            let ids = editor.selected_body_ids.clone();
            let mut modified = false;

            if rot_x_pos.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_x_pos();
                        modified = true;
                    }
                }
                if modified { editor.toast("Pitched selected block(s) +90° around X axis."); }
            }

            if rot_x_neg.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_x_neg();
                        modified = true;
                    }
                }
                if modified { editor.toast("Pitched selected block(s) -90° around X axis."); }
            }

            if rot_y_pos.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_y_pos();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rolled selected block(s) +90° around Y axis."); }
            }

            if rot_y_neg.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_y_neg();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rolled selected block(s) -90° around Y axis."); }
            }

            if rot_cw.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_z_cw();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rotated selected block(s) CW around Z axis."); }
            }

            if rot_ccw.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_z_ccw();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rotated selected block(s) CCW around Z axis."); }
            }

            if ref_x.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.reflect_x();
                        modified = true;
                    }
                }
                if modified { editor.toast("Reflected selected block(s) across X axis."); }
            }

            if ref_y.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.reflect_y();
                        modified = true;
                    }
                }
                if modified { editor.toast("Reflected selected block(s) across Y axis."); }
            }

            if toggle_fixed.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        let (can_moveable, can_fixed) = editor.allowed_fixed_state(body.kind);
                        if can_moveable && can_fixed {
                            if body.is_fixed() {
                                body.tags.remove(TagKind::Fixed);
                            } else {
                                body.tags.set(TagKind::Fixed, TagValue::Unit);
                            }
                            modified = true;
                        }
                    }
                }
                if modified { editor.toast("Toggled Fixed / Moveable on selected block(s)."); }
            }

            if combine_btn.is_some() {
                let all_moveable = ids.iter().all(|&id| {
                    game.engine.world.body(id).map(|b| b.is_pushable()).unwrap_or(false)
                });
                if ids.len() >= 2 && all_moveable {
                    let group_id = game.engine.world.next_combined_group_id();
                    for &id in &ids {
                        if let Some(body) = game.engine.world.body_mut(id) {
                            body.combined_group = Some(group_id);
                            modified = true;
                        }
                    }
                    editor.toast(format!("Combined {} blocks into Mega Block #{}", ids.len(), group_id));
                } else {
                    editor.toast("Cannot combine: all blocks must be moveable (select 2+ moveable blocks).");
                }
            }

            if uncombine_btn.is_some() {
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        if body.combined_group.is_some() {
                            body.combined_group = None;
                            modified = true;
                        }
                    }
                }
                if modified { editor.toast("Uncombined selected blocks from groups."); }
            }

            if del_btn.is_some() {
                for &id in &ids {
                    game.engine.world.despawn(id);
                }
                editor.clear_selection();
                modified = true;
                editor.toast("Deleted selected block(s).");
            }

            if modified {
                game.engine.world.sync_grid();
                let new_world = game.engine.world.clone();
                game.engine.update_authoring_world(new_world);
                editor.cached_solution = None;
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
    camera_query: Query<(&Camera, &GlobalTransform), With<camera::MainCamera>>,
    editor: Res<EditorState>,
    game: Res<GameState>,
    mut gizmos: Gizmos,
) {
    if *app_mode.get() != AppMode::Editor {
        return;
    }

    // 1. Draw hovered cell highlight outline (full 3D box)
    if let Some(cell) = editor.hovered_cell {
        let is_occupied = game.engine.world.body_at(cell).is_some();
        let show_gizmo = if editor.selected_kind.is_some() {
            true
        } else {
            is_occupied
        };

        if show_gizmo {
            let x = cell.x as f32;
            let y = cell.z as f32;
            let z = -cell.y as f32;
            let y0 = y - 0.48;
            let y1 = y + 0.48;
            let d = 0.48;
            let col = if editor.selected_kind.is_some() {
                Color::srgba(1.0, 0.9, 0.2, 0.85)
            } else {
                Color::srgba(0.8, 0.9, 1.0, 0.65)
            };

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

    // 2. Draw selected bodies bounding boxes
    let col = Color::srgba(0.3, 0.8, 1.0, 0.95);
    for &id in &editor.selected_body_ids {
        if let Some(body) = game.engine.world.body(id) {
            for &cell in &body.world_cells() {
                let x = cell.x as f32;
                let y = cell.z as f32;
                let z = -cell.y as f32;
                let y0 = y - 0.45;
                let y1 = y + 0.45;
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

    // 3. Draw active box selection rectangle in FixedLayer mode
    if editor.box_select_active {
        if let (Some(start), Ok((camera, camera_transform))) = (editor.box_select_start, camera_query.single()) {
            if let Some(hovered) = editor.hovered_cell {
                if let Some(start_cell) = raycast_plane_at_z(camera, camera_transform, start, editor.current_z) {
                    let min_x = (start_cell.x.min(hovered.x) as f32) - 0.5;
                    let max_x = (start_cell.x.max(hovered.x) as f32) + 0.5;
                    let min_y = (start_cell.y.min(hovered.y) as f32) - 0.5;
                    let max_y = (start_cell.y.max(hovered.y) as f32) + 0.5;
                    let z_plane = editor.current_z as f32;
                    let box_col = Color::srgba(1.0, 0.85, 0.2, 0.9);

                    let p1 = Vec3::new(min_x, z_plane + 0.05, -min_y);
                    let p2 = Vec3::new(max_x, z_plane + 0.05, -min_y);
                    let p3 = Vec3::new(max_x, z_plane + 0.05, -max_y);
                    let p4 = Vec3::new(min_x, z_plane + 0.05, -max_y);

                    gizmos.line(p1, p2, box_col);
                    gizmos.line(p2, p3, box_col);
                    gizmos.line(p3, p4, box_col);
                    gizmos.line(p4, p1, box_col);
                }
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
    // Return to Editor from Playtest or Playback mode with Escape, or clear selection / tool
    if keys.just_pressed(KeyCode::Escape) {
        if *app_mode.get() != AppMode::Editor {
            game.engine.end_playtest();
            playback.is_playback = false;
            next_mode.set(AppMode::Editor);
            editor.toast("Returned to Level Editor (Frame 0*).");
            return;
        } else if editor.selected_kind.is_some() {
            editor.selected_kind = None;
            editor.toast("Select-Only mode (Palette tool deselected) [Esc].");
        } else if !editor.selected_body_ids.is_empty() {
            editor.clear_selection();
            editor.toast("Deselected all blocks.");
        }
    }

    if *app_mode.get() != AppMode::Editor {
        return;
    }

    // Palette selection hotkeys (when not transforming selected blocks)
    if editor.selected_body_ids.is_empty() {
        if keys.just_pressed(KeyCode::KeyS) {
            editor.selected_kind = None;
            editor.toast("Switched to Select Mode [Key: S]");
        } else if keys.just_pressed(KeyCode::KeyP) {
            editor.selected_kind = Some(BlockKind::Player);
            editor.toast("Selected tool: Player [Key: P]");
        } else if keys.just_pressed(KeyCode::KeyM) {
            editor.selected_kind = Some(BlockKind::Mirror);
            editor.toast("Selected tool: Mirror [Key: M]");
        } else if keys.just_pressed(KeyCode::KeyL) {
            editor.selected_kind = Some(BlockKind::LaserSource);
            editor.toast("Selected tool: Laser Source [Key: L]");
        } else if keys.just_pressed(KeyCode::KeyC) {
            editor.selected_kind = Some(BlockKind::Pushable);
            editor.toast("Selected tool: Pushable Crate [Key: C]");
        } else if keys.just_pressed(KeyCode::KeyW) {
            editor.selected_kind = Some(BlockKind::Wall);
            editor.toast("Selected tool: Wall [Key: W]");
        } else if keys.just_pressed(KeyCode::KeyF) {
            editor.selected_kind = Some(BlockKind::Floor);
            editor.toast("Selected tool: Floor [Key: F]");
        } else if keys.just_pressed(KeyCode::KeyK) {
            editor.selected_kind = Some(BlockKind::Glass);
            editor.toast("Selected tool: Glass Block [Key: K]");
        } else if keys.just_pressed(KeyCode::KeyG) {
            editor.selected_kind = Some(BlockKind::Goal);
            editor.toast("Selected tool: Goal Pyramid [Key: G]");
        }
    }

    // Toggle Z placement mode with Tab
    if keys.just_pressed(KeyCode::Tab) {
        let (new_mode, msg) = match editor.z_mode {
            ZPlacementMode::StackOnTop => {
                let z = editor.current_z;
                (ZPlacementMode::FixedLayer, format!("Z Mode: Fixed Layer (Z={}) [Tab]", z))
            }
            ZPlacementMode::FixedLayer => {
                (ZPlacementMode::StackOnTop, "Z Mode: Stack on Top [Tab]".to_string())
            }
        };
        editor.z_mode = new_mode;
        editor.toast(msg);
    }

    // Change Z layer with PageUp/PageDown or BracketRight/BracketLeft
    if keys.just_pressed(KeyCode::PageUp) || keys.just_pressed(KeyCode::BracketRight) {
        let next_z = (editor.current_z + 1).min(20);
        editor.current_z = next_z;
        editor.toast(format!("Z Layer: {}", next_z));
    }
    if keys.just_pressed(KeyCode::PageDown) || keys.just_pressed(KeyCode::BracketLeft) {
        let next_z = (editor.current_z - 1).max(-5);
        editor.current_z = next_z;
        editor.toast(format!("Z Layer: {}", next_z));
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

    #[test]
    fn ray_intersect_aabb_test() {
        let origin = Vec3::new(0.0, 10.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let min = Vec3::new(-0.5, -0.5, -0.5);
        let max = Vec3::new(0.5, 0.5, 0.5);

        let hit = ray_intersect_aabb(origin, dir, min, max);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 9.5).abs() < 1e-4);

        // Ray missing AABB
        let miss_dir = Vec3::new(1.0, 0.0, 0.0);
        assert!(ray_intersect_aabb(origin, miss_dir, min, max).is_none());
    }

    #[test]
    fn stack_on_top_target_z_calculation() {
        let mut world = World::new();
        let _w1 = world.spawn(BlockKind::Wall, IVec3::new(3, 4, 0), vec![IVec3::ZERO]);
        let w2 = world.spawn(BlockKind::Wall, IVec3::new(3, 4, 1), vec![IVec3::ZERO]);

        // Column (3, 4) has blocks at Z=0 and Z=1.
        let mut max_z_ignore_w2 = -1;
        for body in world.bodies() {
            if body.id == w2 {
                continue;
            }
            for cell in body.world_cells() {
                if cell.x == 3 && cell.y == 4 && cell.z > max_z_ignore_w2 {
                    max_z_ignore_w2 = cell.z;
                }
            }
        }
        assert_eq!(max_z_ignore_w2 + 1, 1);
    }

    #[test]
    fn select_only_mode_state_test() {
        let mut editor = EditorState::default();
        // Starts in Select-Only mode [S]
        assert_eq!(editor.selected_kind, None);
        assert_eq!(editor.stage_ground_z(), 0);

        // Selecting a tool
        editor.selected_kind = Some(BlockKind::Mirror);
        assert_eq!(editor.selected_kind, Some(BlockKind::Mirror));

        // Deselecting enters select-only mode
        editor.selected_kind = None;
        assert!(editor.selected_kind.is_none());

        let allowed = editor.allowed_fixed_state(BlockKind::Mirror);
        assert_eq!(allowed, (true, true));
    }

    #[test]
    fn multi_select_and_locked_layers_test() {
        let mut editor = EditorState::default();
        let id1 = BodyId(1);
        let id2 = BodyId(2);

        editor.select_single(id1);
        assert!(editor.is_selected(id1));
        assert_eq!(editor.selected_body_ids, vec![id1]);

        editor.toggle_selection(id2);
        assert!(editor.is_selected(id1));
        assert!(editor.is_selected(id2));
        assert_eq!(editor.selected_body_ids.len(), 2);

        editor.toggle_selection(id1);
        assert!(!editor.is_selected(id1));
        assert!(editor.is_selected(id2));

        // Lock layer test
        editor.locked_z_layers.insert(-1);
        assert!(editor.is_layer_locked(-1));
        assert!(!editor.is_layer_locked(0));
    }

    #[test]
    fn fill_floorplan_test() {
        let mut world = World::new();
        fill_floorplan(&mut world, 4, 4, -1);
        assert_eq!(world.bodies().len(), 16);
        for b in world.bodies() {
            assert_eq!(b.kind, BlockKind::Floor);
            assert_eq!(b.anchor.z, -1);
            assert!(b.is_fixed());
        }
    }
}
