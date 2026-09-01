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
    OpenLevel,
    Undo,
    Redo,
    ToggleFloorplanModal,
    ToggleFramePreview,
    RotateViewCcw,
    RotateViewCw,
    AttemptSolve,
    AnalyzeQuality,
    TestPlay,
    TestWithSolution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UnsavedAction {
    #[default]
    NewLevel,
    OpenLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZPlacementMode {
    #[default]
    StackOnTop,
    FixedLayer,
}

/// A single snapshot in the level editor's undo / redo history.
#[derive(Clone, Debug)]
pub struct EditorSnapshot {
    /// Deep copy of the authoring world (Frame 0*).
    pub world: World,
    /// Selected body IDs at the time of this state.
    pub selected_body_ids: Vec<BodyId>,
    /// Human-readable action description.
    pub description: String,
}

// ---------------------------------------------------------------------------
// Editor State Resource
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct EditorState {
    /// Currently selected base block kind from palette (None = Select-Only mode).
    pub selected_kind: Option<BlockKind>,
    /// Orientation to assign to newly placed blocks.
    pub placement_orientation: crate::sim::CubeRot,
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
    /// Screen start coordinate for dragging a block (to enforce minimum drag threshold).
    pub drag_start_cursor: Option<Vec2>,
    /// Grid origin cell when starting a drag operation.
    pub drag_origin_cell: Option<IVec3>,
    /// Whether block dragging is currently active (drag distance exceeded threshold).
    pub drag_active: bool,
    /// Snapshot of world at the beginning of an active drag gesture.
    pub drag_start_world: Option<World>,
    /// Screen start coordinate for drag box selection in FixedLayer mode.
    pub box_select_start: Option<Vec2>,
    /// Whether box selection is currently active.
    pub box_select_active: bool,
    /// Set of locked Z layers (blocks on these layers are not selectable/editable).
    pub locked_z_layers: std::collections::HashSet<i32>,
    /// Whether the floorplan dialog is currently open.
    pub floorplan_open: bool,
    /// Whether the Save As dialog is currently open.
    pub save_as_open: bool,
    /// Filename text buffer for Save As dialog.
    pub save_as_filename: String,
    /// Whether the unsaved changes confirmation dialog is currently open.
    pub unsaved_confirm_open: bool,
    /// Pending action that triggered the unsaved changes prompt.
    pub unsaved_action: UnsavedAction,
    /// Whether the file picker dialog is currently open.
    pub file_picker_open: bool,
    /// Directory path currently being browsed in the file picker.
    pub file_picker_dir: String,
    /// Flag indicating that the file picker directory contents need refreshing in the UI.
    pub file_picker_dirty: bool,
    /// World state hash when level was last saved/loaded (for detecting unsaved changes).
    pub last_saved_hash: u64,
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
    /// Saved / recorded solutions for the current level.
    pub solutions: Vec<crate::level::LevelSolution>,
    /// Whether the solution picker modal is currently open.
    pub solution_picker_open: bool,
    /// Flag indicating that the solution picker list needs refreshing in the UI.
    pub solution_picker_dirty: bool,
    /// Background puzzle quality analyzer worker receiver.
    pub analyzer_rx: Option<Arc<Mutex<Receiver<(u64, crate::solver::PuzzleProfile)>>>>,
    /// Level hash at time quality analyzer was launched.
    pub analyzing_hash: Option<u64>,
    /// Latest computed puzzle quality & epiphany profile.
    pub puzzle_profile: Option<crate::solver::PuzzleProfile>,
    /// Whether the quality analysis modal is currently open.
    pub quality_modal_open: bool,
    /// Flag indicating that the quality analysis modal contents need refreshing in the UI.
    pub quality_modal_dirty: bool,
    /// Whether the current playtest win has already been recorded.
    pub playtest_win_recorded: bool,
    /// Playback speed multiplier (default 1.0 = 400ms per step).
    pub playback_speed: f32,
    /// World state snapshot when entering Playtest / Playback mode.
    pub backup_world: Option<World>,
    /// Toast notification banner message & decay timer.
    pub status_message: Option<(String, Timer)>,
    /// Undo history stack.
    pub undo_stack: Vec<EditorSnapshot>,
    /// Redo history stack.
    pub redo_stack: Vec<EditorSnapshot>,
}

pub const MIN_BLOCK_DRAG_PIXELS: f32 = 18.0;
pub const MIN_BOX_SELECT_PIXELS: f32 = 10.0;

impl Default for EditorState {
    fn default() -> Self {
        let mut locked_z_layers = std::collections::HashSet::new();
        locked_z_layers.insert(-1); // Floor layer locked by default

        Self {
            selected_kind: None, // Default to Select Mode [S]
            placement_orientation: crate::sim::CubeRot::IDENTITY,
            is_fixed: false,
            z_mode: ZPlacementMode::StackOnTop,
            current_z: 0,
            selected_body_ids: Vec::new(),
            dragging_body_id: None,
            drag_start_cursor: None,
            drag_origin_cell: None,
            drag_active: false,
            drag_start_world: None,
            box_select_start: None,
            box_select_active: false,
            locked_z_layers,
            floorplan_open: false,
            save_as_open: false,
            save_as_filename: String::from("custom_puzzle.json"),
            unsaved_confirm_open: false,
            unsaved_action: UnsavedAction::NewLevel,
            file_picker_open: false,
            file_picker_dir: String::from("levels"),
            file_picker_dirty: true,
            solutions: Vec::new(),
            solution_picker_open: false,
            solution_picker_dirty: true,
            analyzer_rx: None,
            analyzing_hash: None,
            puzzle_profile: None,
            quality_modal_open: false,
            quality_modal_dirty: true,
            playtest_win_recorded: false,
            playback_speed: 1.0,
            last_saved_hash: 0,
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
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl EditorState {
    /// Copy the block kind, orientation, and fixed property of an existing block,
    /// entering placement mode ready to place matching blocks.
    pub fn copy_and_place(&mut self, body_id: BodyId, world: &World) -> bool {
        if let Some(body) = world.body(body_id) {
            self.selected_kind = Some(body.kind);
            self.placement_orientation = body.orientation;
            let (can_moveable, can_fixed) = self.allowed_fixed_state(body.kind);
            if !can_moveable {
                self.is_fixed = true;
            } else if !can_fixed {
                self.is_fixed = false;
            } else {
                self.is_fixed = body.is_fixed();
            }
            self.clear_selection();
            true
        } else {
            false
        }
    }

    /// Push an undo snapshot before mutating the world. Clears the redo stack.
    pub fn push_undo_snapshot(&mut self, world: &World, description: impl Into<String>) {
        let snapshot = EditorSnapshot {
            world: world.clone(),
            selected_body_ids: self.selected_body_ids.clone(),
            description: description.into(),
        };
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Perform an Undo: restore the world and selection from the top of undo_stack,
    /// and push the current world to redo_stack.
    pub fn perform_undo(&mut self, engine: &mut TurnEngine) -> Option<String> {
        let prev = self.undo_stack.pop()?;
        let current = EditorSnapshot {
            world: engine.world.clone(),
            selected_body_ids: self.selected_body_ids.clone(),
            description: prev.description.clone(),
        };
        self.redo_stack.push(current);

        let desc = prev.description;
        engine.update_authoring_world(prev.world);
        self.selected_body_ids = prev.selected_body_ids
            .into_iter()
            .filter(|&id| engine.world.body(id).is_some())
            .collect();
        self.cached_solution = None;
        self.toast(format!("Undo: {} ({} left in history)", desc, self.undo_stack.len()));
        Some(desc)
    }

    /// Perform a Redo: restore the world and selection from the top of redo_stack,
    /// and push the current world to undo_stack.
    pub fn perform_redo(&mut self, engine: &mut TurnEngine) -> Option<String> {
        let next = self.redo_stack.pop()?;
        let current = EditorSnapshot {
            world: engine.world.clone(),
            selected_body_ids: self.selected_body_ids.clone(),
            description: next.description.clone(),
        };
        self.undo_stack.push(current);

        let desc = next.description;
        engine.update_authoring_world(next.world);
        self.selected_body_ids = next.selected_body_ids
            .into_iter()
            .filter(|&id| engine.world.body(id).is_some())
            .collect();
        self.cached_solution = None;
        self.toast(format!("Redo: {} ({} available)", desc, self.redo_stack.len()));
        Some(desc)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.drag_start_world = None;
    }

    /// Return the stage ground level Z (where items sit on top of the floor).
    pub fn stage_ground_z(&self) -> i32 {
        self.floorplan_z + 1
    }

    /// Check if the simulation world has unsaved modifications.
    pub fn has_unsaved_changes(&self, world: &World) -> bool {
        compute_level_hash(world) != self.last_saved_hash
    }

    /// Reset level and create a new blank 10x10 puzzle room with perimeter walls and player.
    pub fn create_new_blank_room(&mut self, engine: &mut TurnEngine) {
        self.push_undo_snapshot(&engine.world, "New Blank Room");
        let mut world = World::new();
        fill_floorplan(&mut world, 10, 10, -1);
        world.spawn(BlockKind::Player, IVec3::new(3, 8, 0), vec![IVec3::ZERO]);
        let gid = world.spawn(BlockKind::Goal, IVec3::new(8, 8, 0), vec![IVec3::ZERO]);
        if let Some(b) = world.body_mut(gid) {
            b.tags.set(TagKind::Fixed, TagValue::Unit);
        }
        for x in 0..10 {
            let w1 = world.spawn(BlockKind::Wall, IVec3::new(x, 0, 0), vec![IVec3::ZERO]);
            if let Some(b) = world.body_mut(w1) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
            let w2 = world.spawn(BlockKind::Wall, IVec3::new(x, 9, 0), vec![IVec3::ZERO]);
            if let Some(b) = world.body_mut(w2) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
        }
        for y in 1..9 {
            let w1 = world.spawn(BlockKind::Wall, IVec3::new(0, y, 0), vec![IVec3::ZERO]);
            if let Some(b) = world.body_mut(w1) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
            let w2 = world.spawn(BlockKind::Wall, IVec3::new(9, y, 0), vec![IVec3::ZERO]);
            if let Some(b) = world.body_mut(w2) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
        }
        world.sync_grid();
        *engine = TurnEngine::new(world);
        self.locked_z_layers.clear();
        self.locked_z_layers.insert(-1);
        self.clear_selection();
        self.cached_solution = None;
        self.solutions.clear();
        self.solver_status = "Idle".into();
        self.last_saved_hash = compute_level_hash(&engine.world);
        self.current_level_path = "levels/custom_puzzle.json".into();
        self.toast("Created new blank 10x10 puzzle room with locked floor.");
    }

    /// Open the file picker modal starting in the `levels` directory.
    pub fn open_file_picker(&mut self) {
        self.file_picker_open = true;
        self.file_picker_dirty = true;
        if self.file_picker_dir.is_empty() {
            self.file_picker_dir = "levels".to_string();
        }
        self.toast("Browsing level files...");
    }

    /// Close any open modals. Returns true if a modal was closed.
    pub fn close_modals(&mut self) -> bool {
        let mut closed = false;
        if self.floorplan_open {
            self.floorplan_open = false;
            closed = true;
        }
        if self.save_as_open {
            self.save_as_open = false;
            closed = true;
        }
        if self.unsaved_confirm_open {
            self.unsaved_confirm_open = false;
            closed = true;
        }
        if self.file_picker_open {
            self.file_picker_open = false;
            closed = true;
        }
        if self.solution_picker_open {
            self.solution_picker_open = false;
            closed = true;
        }
        closed
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
                    ui::update_file_picker_ui_system,
                    ui::update_solution_picker_ui_system,
                    ui::update_quality_modal_ui_system,
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
                    file_picker_button_clicks_system,
                    solution_picker_button_clicks_system,
                    ui::quality_modal_interaction_system,
                )
                    .run_if(in_state(AppMode::Editor)),
            );
    }
}

pub fn update_palette_3d_preview(
    _time: Res<Time>,
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

        // Apply base placement orientation
        transform.rotation = crate::render::cube_rot_to_quat(&editor.placement_orientation);

        let (can_moveable, can_fixed) = editor.allowed_fixed_state(selected_kind);
        let is_moveable = if !can_moveable {
            false
        } else if !can_fixed {
            true
        } else {
            !editor.is_fixed
        };

        let visual_spec = crate::render::BlockVisualSpec::from_kind_and_props(
            selected_kind,
            is_moveable,
            false,
            false,
            false,
            None,
            true,
        );
        let target_mesh = render_assets.resolve_mesh(&visual_spec.mesh);
        let target_mat = render_assets.resolve_material(&visual_spec.material);

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

/// Helper to check if an entire selection of bodies can be moved by `delta` without colliding
/// with any other non-selected bodies in the world.
pub fn can_move_selection_by(world: &World, selected_ids: &[BodyId], delta: IVec3) -> bool {
    if selected_ids.is_empty() || delta == IVec3::ZERO {
        return false;
    }
    let sel_set: std::collections::HashSet<BodyId> = selected_ids.iter().copied().collect();
    for &id in selected_ids {
        if let Some(body) = world.body(id) {
            let new_anchor = body.anchor + delta;
            for local in &body.shape {
                let world_cell = new_anchor + body.orientation.apply(*local);
                if let Some(occ) = world.body_at(world_cell) {
                    if !sel_set.contains(&occ.id) {
                        return false;
                    }
                }
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

/// Helper function to execute Save As with the filename in `editor.save_as_filename`.
pub fn execute_save_as(editor: &mut EditorState, world: &World) {
    let mut filename = editor.save_as_filename.trim().to_string();
    if filename.is_empty() {
        filename = "custom_puzzle.json".to_string();
    }
    if !filename.ends_with(".json") {
        filename.push_str(".json");
    }
    let save_path = if filename.starts_with("levels/") {
        filename
    } else {
        format!("levels/{}", filename)
    };

    // Prune invalid solutions against current world before saving
    editor.solutions.retain(|s| level::validate_solution(world, &s.actions));

    let level_data = LevelData::from_world_with_solutions("Custom Level", world, editor.solutions.clone());
    match level::save_level_to_file(&save_path, &level_data) {
        Ok(_) => {
            editor.current_level_path = save_path.clone();
            editor.last_saved_hash = compute_level_hash(world);
            editor.save_as_open = false;
            editor.toast(format!("Saved level with {} solution(s) to {}", editor.solutions.len(), save_path));
        }
        Err(e) => {
            editor.toast(format!("Save error: {}", e));
        }
    }
}

/// Map KeyCode to character for typing in text input fields.
fn key_code_to_char(key: KeyCode, shift: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Period => Some('.'),
        KeyCode::Slash => Some('/'),
        KeyCode::Space => Some('_'),
        _ => None,
    }
}

fn editor_grid_interaction_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<camera::MainCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
) {
    if editor.floorplan_open || editor.save_as_open || editor.unsaved_confirm_open || editor.file_picker_open || editor.solution_picker_open {
        return;
    }

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let modifier_held = keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight) || keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    // 1. Multi-block transform shortcuts for all selected blocks (when Cmd/Ctrl not held)
    if !modifier_held && !editor.selected_body_ids.is_empty() {
        let ids = editor.selected_body_ids.clone();
        let mut modified = false;

        if keyboard.just_pressed(KeyCode::KeyR) {
            editor.push_undo_snapshot(&game.engine.world, "Rotate Block(s) CW");
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
            editor.push_undo_snapshot(&game.engine.world, "Pitch Block(s) +90°");
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
            editor.push_undo_snapshot(&game.engine.world, "Roll Block(s) +90°");
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
            editor.push_undo_snapshot(&game.engine.world, "Flip Block(s) X");
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
            editor.push_undo_snapshot(&game.engine.world, "Flip Block(s) Y");
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
            editor.push_undo_snapshot(&game.engine.world, format!("Delete {} Block(s)", ids.len()));
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
    } else if !modifier_held && editor.selected_kind.is_some() {
        if keyboard.just_pressed(KeyCode::KeyR) {
            editor.placement_orientation = editor.placement_orientation.rot_world_z_cw();
            editor.toast("Placement orientation: Rotated CW [R].");
        } else if keyboard.just_pressed(KeyCode::KeyT) {
            editor.placement_orientation = editor.placement_orientation.rot_world_x_pos();
            editor.toast("Placement orientation: Pitched +90° around X axis [T].");
        } else if keyboard.just_pressed(KeyCode::KeyG) {
            editor.placement_orientation = editor.placement_orientation.rot_world_y_pos();
            editor.toast("Placement orientation: Rolled +90° around Y axis [G].");
        } else if keyboard.just_pressed(KeyCode::KeyX) {
            editor.placement_orientation = editor.placement_orientation.reflect_x();
            editor.toast("Placement orientation: Flipped across X axis [X].");
        } else if keyboard.just_pressed(KeyCode::KeyY) {
            editor.placement_orientation = editor.placement_orientation.reflect_y();
            editor.toast("Placement orientation: Flipped across Y axis [Y].");
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

    let ignore_id = if editor.drag_active { editor.dragging_body_id } else { None };
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

                editor.push_undo_snapshot(&game.engine.world, format!("Place {:?}", kind));

                if kind == BlockKind::Player {
                    if let Some(player_id) = game.engine.world.player_id() {
                        if let Some(p) = game.engine.world.body_mut(player_id) {
                            p.anchor = place_target;
                            game.engine.world.sync_grid();
                            let new_world = game.engine.world.clone();
                            game.engine.update_authoring_world(new_world);
                            editor.clear_selection();
                            editor.dragging_body_id = None;
                            editor.drag_start_cursor = None;
                            editor.drag_origin_cell = None;
                            editor.drag_active = false;
                            editor.drag_start_world = None;
                            editor.toast(format!("Relocated player character to ({}, {}, {}).", place_target.x, place_target.y, place_target.z));
                            editor.cached_solution = None;
                            return;
                        }
                    }
                }

                let new_id = game.engine.world.spawn(kind, place_target, vec![IVec3::ZERO]);
                if let Some(b) = game.engine.world.body_mut(new_id) {
                    b.orientation = editor.placement_orientation;
                    if is_fixed {
                        b.tags.set(TagKind::Fixed, TagValue::Unit);
                    }
                }
                game.engine.world.sync_grid();
                let new_world = game.engine.world.clone();
                game.engine.update_authoring_world(new_world);
                editor.clear_selection();
                editor.dragging_body_id = None;
                editor.drag_start_cursor = None;
                editor.drag_origin_cell = None;
                editor.drag_active = false;
                editor.drag_start_world = None;
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
                if let Some(body_id) = clicked_body_id {
                    if shift_held {
                        editor.toggle_selection(body_id);
                        let total = editor.selected_body_ids.len();
                        editor.toast(format!("Toggled selection (Total: {}).", total));
                        editor.box_select_start = None;
                        editor.box_select_active = false;
                        editor.drag_start_cursor = None;
                        editor.drag_origin_cell = None;
                        editor.drag_active = false;
                        editor.drag_start_world = None;
                    } else {
                        if !editor.is_selected(body_id) {
                            editor.select_single(body_id);
                            editor.toast(format!("Selected block (ID {}).", body_id.0));
                        }
                        // Start tracking drag for the entire selection
                        editor.drag_start_cursor = Some(cursor_pos);
                        editor.drag_origin_cell = Some(cell_pos);
                        editor.drag_active = false;
                        editor.drag_start_world = Some(game.engine.world.clone());
                        editor.box_select_start = None;
                        editor.box_select_active = false;
                    }
                } else if shift_held {
                    // Shift-click on empty space: start additive box select
                    editor.box_select_start = Some(cursor_pos);
                    editor.box_select_active = false;
                    editor.drag_start_cursor = None;
                    editor.drag_origin_cell = None;
                    editor.drag_active = false;
                    editor.drag_start_world = None;
                } else {
                    // Normal click on empty space: clear selection & start box select
                    editor.clear_selection();
                    editor.box_select_start = Some(cursor_pos);
                    editor.box_select_active = false;
                    editor.drag_start_cursor = None;
                    editor.drag_origin_cell = None;
                    editor.drag_active = false;
                    editor.drag_start_world = None;
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
                        editor.drag_start_cursor = Some(cursor_pos);
                        editor.drag_origin_cell = Some(cell_pos);
                        editor.drag_active = false;
                        editor.drag_start_world = Some(game.engine.world.clone());
                        editor.toast(format!("Selected block (ID {}).", body_id.0));
                    }
                } else if !shift_held {
                    editor.clear_selection();
                    editor.dragging_body_id = None;
                    editor.drag_start_cursor = None;
                    editor.drag_origin_cell = None;
                    editor.drag_active = false;
                    editor.drag_start_world = None;
                }
            }
        }
    } else if mouse_button.pressed(MouseButton::Left) {
        if editor.selected_kind.is_none() {
            if editor.z_mode == ZPlacementMode::FixedLayer {
                if let Some(start) = editor.box_select_start {
                    if cursor_pos.distance(start) >= MIN_BOX_SELECT_PIXELS {
                        editor.box_select_active = true;
                    }
                } else if let Some(origin_cell) = editor.drag_origin_cell {
                    if let Some(start_pos) = editor.drag_start_cursor {
                        if cursor_pos.distance(start_pos) >= MIN_BLOCK_DRAG_PIXELS {
                            editor.drag_active = true;
                        }
                    }

                    if editor.drag_active {
                        let delta = cell_pos - origin_cell;
                        if (delta.x != 0 || delta.y != 0) && can_move_selection_by(&game.engine.world, &editor.selected_body_ids, delta) {
                            for &id in &editor.selected_body_ids {
                                if let Some(b) = game.engine.world.body_mut(id) {
                                    b.anchor += delta;
                                }
                            }
                            game.engine.world.sync_grid();
                            let new_world = game.engine.world.clone();
                            game.engine.update_authoring_world(new_world);
                            editor.drag_origin_cell = Some(cell_pos);
                            editor.cached_solution = None;
                        }
                    }
                }
            } else if let Some(drag_id) = editor.dragging_body_id {
                // StackOnTop: Check minimum drag distance before moving block
                if let Some(start_pos) = editor.drag_start_cursor {
                    if cursor_pos.distance(start_pos) >= MIN_BLOCK_DRAG_PIXELS {
                        editor.drag_active = true;
                    }
                }

                // Only move block once the drag threshold is reached
                if editor.drag_active {
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
    }

    // 3. Release Left Click
    if mouse_button.just_released(MouseButton::Left) {
        if let Some(start_world) = editor.drag_start_world.take() {
            if editor.drag_active && compute_level_hash(&start_world) != compute_level_hash(&game.engine.world) {
                let sel = editor.selected_body_ids.clone();
                editor.undo_stack.push(EditorSnapshot {
                    world: start_world,
                    selected_body_ids: sel,
                    description: "Move Block(s)".into(),
                });
                if editor.undo_stack.len() > 100 {
                    editor.undo_stack.remove(0);
                }
                editor.redo_stack.clear();
            }
        }

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
            } else if editor.drag_active {
                let total = editor.selected_body_ids.len();
                editor.toast(format!("Moved {} selected block(s).", total));
            }
            editor.box_select_start = None;
            editor.box_select_active = false;
            editor.drag_start_cursor = None;
            editor.drag_origin_cell = None;
            editor.drag_active = false;
        } else {
            if editor.drag_active {
                if let Some(drag_id) = editor.dragging_body_id {
                    if let Some(body) = game.engine.world.body(drag_id) {
                        editor.toast(format!("Moved {:?} to ({}, {}, {}).", body.kind, body.anchor.x, body.anchor.y, body.anchor.z));
                    }
                }
            }
            editor.dragging_body_id = None;
            editor.drag_start_cursor = None;
            editor.drag_origin_cell = None;
            editor.drag_active = false;
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
                    editor.push_undo_snapshot(&game.engine.world, "Delete Block");
                    game.engine.world.despawn(id);
                    editor.selected_body_ids.retain(|&x| x != id);
                    if editor.dragging_body_id == Some(id) {
                        editor.dragging_body_id = None;
                        editor.drag_start_cursor = None;
                        editor.drag_active = false;
                        editor.drag_start_world = None;
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
            (
                Option<&ui::SaveAsConfirmBtn>,
                Option<&ui::SaveAsCancelBtn>,
                Option<&ui::DiscardConfirmBtn>,
                Option<&ui::DiscardCancelBtn>,
            ),
            (
                Option<&ui::CopyAndPlaceButton>,
                Option<&ui::ResetPlacementOrientationButton>,
            ),
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut camera_query: Query<&mut camera::CameraController, With<camera::MainCamera>>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<AppMode>>,
    _playback: ResMut<crate::PlaybackState>,
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
        (save_as_confirm, save_as_cancel, discard_confirm, discard_cancel),
        (copy_and_place_btn, reset_orientation_btn),
    ) in &mut interaction_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Copy and Place button
        if copy_and_place_btn.is_some() {
            if let Some(&first_id) = editor.selected_body_ids.first() {
                if editor.copy_and_place(first_id, &game.engine.world) {
                    if let Some(kind) = editor.selected_kind {
                        editor.toast(format!("Copied {:?} properties to Placement Mode.", kind));
                    }
                }
            }
            continue;
        }

        // Reset Placement Orientation button
        if reset_orientation_btn.is_some() {
            editor.placement_orientation = crate::sim::CubeRot::IDENTITY;
            editor.toast("Reset placement orientation.");
            continue;
        }

        // Modal Action buttons
        if save_as_confirm.is_some() {
            execute_save_as(&mut editor, &game.engine.world);
        }
        if save_as_cancel.is_some() {
            editor.save_as_open = false;
            editor.toast("Cancelled Save As.");
        }
        if discard_confirm.is_some() {
            editor.unsaved_confirm_open = false;
            match editor.unsaved_action {
                UnsavedAction::NewLevel => editor.create_new_blank_room(&mut game.engine),
                UnsavedAction::OpenLevel => editor.open_file_picker(),
            }
        }
        if discard_cancel.is_some() {
            editor.unsaved_confirm_open = false;
            editor.toast("Cancelled.");
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
            editor.floorplan_z = (editor.floorplan_z - 1).max(-5);
        }
        if fp_z_inc.is_some() {
            editor.floorplan_z = (editor.floorplan_z + 1).min(20);
        }
        if fp_fill.is_some() {
            let w = editor.floorplan_width;
            let h = editor.floorplan_height;
            let z = editor.floorplan_z;
            editor.push_undo_snapshot(&game.engine.world, format!("Fill Floorplan ({}x{} @ Z={})", w, h, z));
            fill_floorplan(&mut game.engine.world, w, h, z);
            let new_world = game.engine.world.clone();
            game.engine.update_authoring_world(new_world);
            editor.cached_solution = None;
            editor.toast(format!("Filled {}x{} floor at Z={}.", w, h, z));
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

        // 6. Action buttons (Top Bar)
        if let Some(act) = action_btn {
            match act.0 {
                EditorAction::NewLevel => {
                    let current_hash = compute_level_hash(&game.engine.world);
                    if current_hash != editor.last_saved_hash {
                        editor.unsaved_action = UnsavedAction::NewLevel;
                        editor.unsaved_confirm_open = true;
                    } else {
                        editor.create_new_blank_room(&mut game.engine);
                    }
                }
                EditorAction::Save => {
                    let path = editor.current_level_path.clone();
                    editor.solutions.retain(|s| level::validate_solution(&game.engine.world, &s.actions));
                    let sol_count = editor.solutions.len();
                    let level_data = LevelData::from_world_with_solutions("Custom Level", &game.engine.world, editor.solutions.clone());
                    match level::save_level_to_file(&path, &level_data) {
                        Ok(_) => {
                            editor.last_saved_hash = compute_level_hash(&game.engine.world);
                            editor.toast(format!("Saved level with {} solution(s) to {}", sol_count, path));
                        }
                        Err(err) => {
                            editor.toast(format!("Save failed: {}", err));
                        }
                    }
                }
                EditorAction::SaveAs => {
                    let base_name = std::path::Path::new(&editor.current_level_path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("custom_puzzle.json");
                    editor.save_as_filename = base_name.to_string();
                    editor.save_as_open = true;
                }
                EditorAction::OpenLevel => {
                    let current_hash = compute_level_hash(&game.engine.world);
                    if current_hash != editor.last_saved_hash {
                        editor.unsaved_action = UnsavedAction::OpenLevel;
                        editor.unsaved_confirm_open = true;
                    } else {
                        editor.open_file_picker();
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
                EditorAction::Undo => {
                    if editor.perform_undo(&mut game.engine).is_none() {
                        editor.toast("Nothing to undo.");
                    }
                }
                EditorAction::Redo => {
                    if editor.perform_redo(&mut game.engine).is_none() {
                        editor.toast("Nothing to redo.");
                    }
                }
                EditorAction::AttemptSolve => {
                    if !game.engine.is_valid() {
                        editor.solver_status = "Invalid Level".into();
                        editor.toast("Cannot solve: Level has spontaneous movement at Frame 1.");
                        return;
                    }
                    let current_hash = compute_level_hash(&game.engine.world);
                    let cached_opt = editor.cached_solution.clone();
                    if let Some((cached_hash, sol)) = cached_opt {
                        if cached_hash == current_hash {
                            editor.toast(format!("Instant cached solution available: {} step(s)!", sol.len()));
                            editor.solutions.retain(|s| level::validate_solution(&game.engine.world, &s.actions));
                            if !editor.solutions.iter().any(|s| s.actions == sol) {
                                editor.solutions.push(crate::level::LevelSolution {
                                    name: "Optimal Solver Solution".into(),
                                    actions: sol.clone(),
                                });
                            }
                            editor.solution_picker_open = true;
                            editor.solution_picker_dirty = true;
                            return;
                        }
                    }

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
                EditorAction::AnalyzeQuality => {
                    if let Some(err) = &game.engine.validation_error {
                        editor.toast(format!("Cannot analyze invalid level: {}", err));
                        return;
                    }
                    if editor.analyzer_rx.is_some() {
                        editor.toast("Quality analyzer is already running...");
                        return;
                    }
                    let current_hash = compute_level_hash(&game.engine.world);
                    let world_clone = game.engine.world.clone();
                    let (tx, rx) = mpsc::channel();
                    editor.analyzer_rx = Some(Arc::new(Mutex::new(rx)));
                    editor.analyzing_hash = Some(current_hash);
                    editor.toast("Analyzing puzzle quality in background...");

                    std::thread::spawn(move || {
                        let profile = solver::analyze_puzzle(&world_clone);
                        let _ = tx.send((current_hash, profile));
                    });
                }
                EditorAction::TestPlay => {
                    match game.engine.start_playtest() {
                        Ok(()) => {
                            editor.backup_world = Some(game.engine.frame_zero_star.clone());
                            editor.playtest_win_recorded = false;
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
                    // Validate and prune solutions against current world
                    editor.solutions.retain(|s| level::validate_solution(&game.engine.world, &s.actions));
                    if editor.solutions.is_empty() {
                        editor.toast("No valid solutions for current level. Run 'Solve Level' or play to solve!");
                    } else {
                        editor.solution_picker_open = true;
                        editor.solution_picker_dirty = true;
                    }
                }
            }
        }

        // 7. Inspector transforms for all selected bodies
        if !editor.selected_body_ids.is_empty() {
            let ids = editor.selected_body_ids.clone();
            let mut modified = false;

            if rot_x_pos.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Pitch +X");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_x_pos();
                        modified = true;
                    }
                }
                if modified { editor.toast("Pitched selected block(s) +90° around X axis."); }
            }

            if rot_x_neg.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Pitch -X");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_x_neg();
                        modified = true;
                    }
                }
                if modified { editor.toast("Pitched selected block(s) -90° around X axis."); }
            }

            if rot_y_pos.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Roll +Y");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_y_pos();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rolled selected block(s) +90° around Y axis."); }
            }

            if rot_y_neg.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Roll -Y");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_y_neg();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rolled selected block(s) -90° around Y axis."); }
            }

            if rot_cw.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Rotate CW");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_z_cw();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rotated selected block(s) CW around Z axis."); }
            }

            if rot_ccw.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Rotate CCW");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.rot_world_z_ccw();
                        modified = true;
                    }
                }
                if modified { editor.toast("Rotated selected block(s) CCW around Z axis."); }
            }

            if ref_x.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Reflect X");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.reflect_x();
                        modified = true;
                    }
                }
                if modified { editor.toast("Reflected selected block(s) across X axis."); }
            }

            if ref_y.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Reflect Y");
                for &id in &ids {
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = body.orientation.reflect_y();
                        modified = true;
                    }
                }
                if modified { editor.toast("Reflected selected block(s) across Y axis."); }
            }

            if toggle_fixed.is_some() {
                editor.push_undo_snapshot(&game.engine.world, "Toggle Fixed/Moveable");
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
                    editor.push_undo_snapshot(&game.engine.world, "Combine Mega Block");
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
                let has_group = ids.iter().any(|&id| {
                    game.engine.world.body(id).map(|b| b.combined_group.is_some()).unwrap_or(false)
                });
                if has_group {
                    editor.push_undo_snapshot(&game.engine.world, "Uncombine Group");
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
            }

            if del_btn.is_some() {
                editor.push_undo_snapshot(&game.engine.world, format!("Delete {} Block(s)", ids.len()));
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
        // 8. Transformations for placement orientation (when in placement mode)
        else if let Some(kind) = editor.selected_kind {
            if rot_x_pos.is_some() {
                editor.placement_orientation = editor.placement_orientation.rot_world_x_pos();
                editor.toast("Placement orientation: Pitched +90° around X axis.");
            }
            if rot_x_neg.is_some() {
                editor.placement_orientation = editor.placement_orientation.rot_world_x_neg();
                editor.toast("Placement orientation: Pitched -90° around X axis.");
            }
            if rot_y_pos.is_some() {
                editor.placement_orientation = editor.placement_orientation.rot_world_y_pos();
                editor.toast("Placement orientation: Rolled +90° around Y axis.");
            }
            if rot_y_neg.is_some() {
                editor.placement_orientation = editor.placement_orientation.rot_world_y_neg();
                editor.toast("Placement orientation: Rolled -90° around Y axis.");
            }
            if rot_cw.is_some() {
                editor.placement_orientation = editor.placement_orientation.rot_world_z_cw();
                editor.toast("Placement orientation: Rotated CW around Z axis.");
            }
            if rot_ccw.is_some() {
                editor.placement_orientation = editor.placement_orientation.rot_world_z_ccw();
                editor.toast("Placement orientation: Rotated CCW around Z axis.");
            }
            if ref_x.is_some() {
                editor.placement_orientation = editor.placement_orientation.reflect_x();
                editor.toast("Placement orientation: Reflected across X axis.");
            }
            if ref_y.is_some() {
                editor.placement_orientation = editor.placement_orientation.reflect_y();
                editor.toast("Placement orientation: Reflected across Y axis.");
            }
            if toggle_fixed.is_some() {
                let (can_moveable, can_fixed) = editor.allowed_fixed_state(kind);
                if can_moveable && can_fixed {
                    editor.is_fixed = !editor.is_fixed;
                    let prop = if editor.is_fixed { "Stationary" } else { "Moveable" };
                    editor.toast(format!("Placement property: {}", prop));
                }
            }
        }
    }
}

/// Handles button clicks inside the floating File Picker dialog.
fn file_picker_button_clicks_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ui::FilePickerUpBtn>,
            Option<&ui::FilePickerDirBtn>,
            Option<&ui::FilePickerFileBtn>,
            Option<&ui::FilePickerCancelBtn>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
) {
    if !editor.file_picker_open {
        return;
    }

    for (interaction, up_btn, dir_btn, file_btn, cancel_btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(up) = up_btn {
            editor.file_picker_dir = up.0.clone();
            editor.file_picker_dirty = true;
        } else if let Some(dir) = dir_btn {
            editor.file_picker_dir = dir.0.clone();
            editor.file_picker_dirty = true;
        } else if let Some(file) = file_btn {
            let target_path = file.0.clone();
            match level::load_level_from_file(&target_path) {
                Ok(lvl) => {
                    game.engine = TurnEngine::new(lvl.to_world());
                    editor.current_level_path = target_path.clone();
                    editor.last_saved_hash = compute_level_hash(&game.engine.world);
                    editor.solutions = lvl.solutions.clone();
                    editor.solutions.retain(|s| level::validate_solution(&game.engine.world, &s.actions));
                    let sol_count = editor.solutions.len();
                    editor.clear_selection();
                    editor.clear_history();
                    editor.cached_solution = None;
                    editor.solver_status = "Idle".into();
                    editor.file_picker_open = false;
                    editor.toast(format!("Loaded: {} ({} solution(s))", target_path, sol_count));
                }
                Err(e) => {
                    editor.toast(format!("Load error: {}", e));
                }
            }
        } else if cancel_btn.is_some() {
            editor.file_picker_open = false;
            editor.toast("Cancelled file picker.");
        }
    }
}

/// Handles button clicks inside the floating Solution Picker dialog.
fn solution_picker_button_clicks_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ui::SolutionPlayBtn>,
            Option<&ui::SolutionDeleteBtn>,
            Option<&ui::SolutionPickerCancelBtn>,
            Option<&ui::SolutionSpeedDecBtn>,
            Option<&ui::SolutionSpeedIncBtn>,
            Option<&ui::SolutionSpeedPresetBtn>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<AppMode>>,
    mut playback: ResMut<crate::PlaybackState>,
) {
    if !editor.solution_picker_open {
        return;
    }

    for (interaction, play_btn, del_btn, cancel_btn, speed_dec, speed_inc, speed_preset) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(play) = play_btn {
            let idx = play.0;
            if idx < editor.solutions.len() {
                let sol = editor.solutions[idx].clone();
                editor.solution_picker_open = false;
                if game.engine.start_playtest().is_ok() {
                    editor.backup_world = Some(game.engine.frame_zero_star.clone());
                    playback.is_playback = true;
                    playback.actions = sol.actions;
                    playback.current_index = 0;
                    playback.auto_playing = true;
                    playback.speed = editor.playback_speed;
                    playback.step_timer = Timer::from_seconds(0.40 / editor.playback_speed.max(0.1), TimerMode::Repeating);
                    let speed = editor.playback_speed;
                    next_mode.set(AppMode::Playback);
                    editor.toast(format!("Playing solution #{}: {} ({:.1}x speed)", idx + 1, sol.name, speed));
                }
            }
        } else if let Some(del) = del_btn {
            let idx = del.0;
            if idx < editor.solutions.len() {
                let removed = editor.solutions.remove(idx);
                editor.solution_picker_dirty = true;
                editor.toast(format!("Deleted solution: {}", removed.name));
            }
        } else if cancel_btn.is_some() {
            editor.solution_picker_open = false;
            editor.toast("Cancelled solution picker.");
        } else if speed_dec.is_some() {
            editor.playback_speed = (editor.playback_speed * 0.75).clamp(0.2, 10.0);
        } else if speed_inc.is_some() {
            editor.playback_speed = (editor.playback_speed * 1.5).clamp(0.2, 10.0);
        } else if let Some(preset) = speed_preset {
            editor.playback_speed = preset.0;
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
                    "✓ Solved: {} moves ({} turns) ({:.2?})",
                    result.macro_moves.len(),
                    result.actions.len(),
                    result.duration
                );
                editor.cached_solution = Some((current_hash, result.actions.clone()));
                let name = format!("Solver A* ({} moves, {} turns)", result.macro_moves.len(), result.actions.len());
                if !editor.solutions.iter().any(|s| s.actions == result.actions) {
                    editor.solutions.push(crate::level::LevelSolution {
                        name,
                        actions: result.actions.clone(),
                    });
                }
                let msg = format!(
                    "Solver: Found {}-move ({}-turn) solution in {:.2?} (added to solutions)!",
                    result.macro_moves.len(),
                    result.actions.len(),
                    result.duration
                );
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

    // 2. Poll background quality analyzer worker
    let analyzer_result_pair = if let Some(rx_arc) = &editor.analyzer_rx {
        if let Ok(rx) = rx_arc.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    } else {
        None
    };

    if let Some((analyzed_hash, profile)) = analyzer_result_pair {
        editor.analyzer_rx = None;
        let current_hash = compute_level_hash(&game.engine.world);

        if analyzed_hash == current_hash {
            editor.puzzle_profile = Some(profile);
            editor.quality_modal_open = true;
            editor.quality_modal_dirty = true;
            editor.toast("Puzzle Quality Analysis complete!");
        } else {
            editor.toast("Level modified during analysis. Invalidated.");
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
    if *app_mode.get() == AppMode::Editor {
        if editor.save_as_open {
            let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            if keys.just_pressed(KeyCode::Escape) {
                editor.save_as_open = false;
                editor.toast("Cancelled Save As [Esc].");
                return;
            }
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
                execute_save_as(&mut editor, &game.engine.world);
                return;
            }
            if keys.just_pressed(KeyCode::Backspace) {
                editor.save_as_filename.pop();
                return;
            }
            for key in keys.get_just_pressed() {
                if let Some(c) = key_code_to_char(*key, shift_held) {
                    if editor.save_as_filename.len() < 64 {
                        editor.save_as_filename.push(c);
                    }
                }
            }
            return;
        }

        if editor.file_picker_open {
            if keys.just_pressed(KeyCode::Escape) {
                editor.file_picker_open = false;
                editor.toast("Cancelled file picker [Esc].");
                return;
            }
            return;
        }

        if editor.solution_picker_open {
            if keys.just_pressed(KeyCode::Escape) {
                editor.solution_picker_open = false;
                editor.toast("Cancelled solution picker [Esc].");
                return;
            }
            return;
        }

        if editor.quality_modal_open {
            if keys.just_pressed(KeyCode::Escape) {
                editor.quality_modal_open = false;
                editor.toast("Closed quality analysis report [Esc].");
                return;
            }
            return;
        }

        if editor.unsaved_confirm_open {
            if keys.just_pressed(KeyCode::Escape) {
                editor.unsaved_confirm_open = false;
                editor.toast("Cancelled [Esc].");
                return;
            }
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
                editor.unsaved_confirm_open = false;
                match editor.unsaved_action {
                    UnsavedAction::NewLevel => editor.create_new_blank_room(&mut game.engine),
                    UnsavedAction::OpenLevel => editor.open_file_picker(),
                }
                return;
            }
            return;
        }
    }

    // Handle Escape key:
    // 1. Return to Editor from Playtest or Playback mode
    // 2. Close any open modal (e.g. Floorplan modal)
    // 3. Clear block selection if blocks are selected
    // 4. Revert to Select Mode [S] if a placement tool is active
    if keys.just_pressed(KeyCode::Escape) {
        if *app_mode.get() != AppMode::Editor {
            game.engine.end_playtest();
            playback.is_playback = false;
            next_mode.set(AppMode::Editor);
            editor.toast("Returned to Level Editor (Frame 0*).");
            return;
        } else if editor.close_modals() {
            editor.toast("Closed modal [Esc].");
            return;
        } else if !editor.selected_body_ids.is_empty() {
            editor.clear_selection();
            editor.toast("Cleared selection [Esc].");
        } else if editor.selected_kind.is_some() {
            editor.selected_kind = None;
            editor.toast("Reverted to Select Mode [Esc].");
        }
    }

    if *app_mode.get() != AppMode::Editor {
        return;
    }

    let cmd_held = keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight);
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let modifier_held = cmd_held || ctrl_held;
    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    // Undo / Redo & Save shortcuts:
    // Cmd+Z / Ctrl+Z (Undo)
    // Cmd+Shift+Z / Ctrl+Shift+Z / Cmd+Y / Ctrl+Y (Redo)
    // Cmd+S / Ctrl+S (Save)
    if modifier_held {
        if keys.just_pressed(KeyCode::KeyZ) {
            if shift_held {
                if editor.perform_redo(&mut game.engine).is_none() {
                    editor.toast("Nothing to redo.");
                }
            } else {
                if editor.perform_undo(&mut game.engine).is_none() {
                    editor.toast("Nothing to undo.");
                }
            }
            return;
        }
        if keys.just_pressed(KeyCode::KeyY) {
            if editor.perform_redo(&mut game.engine).is_none() {
                editor.toast("Nothing to redo.");
            }
            return;
        }
        if keys.just_pressed(KeyCode::KeyS) {
            let path = editor.current_level_path.clone();
            editor.solutions.retain(|s| level::validate_solution(&game.engine.world, &s.actions));
            let sol_count = editor.solutions.len();
            let level_data = LevelData::from_world_with_solutions("Custom Level", &game.engine.world, editor.solutions.clone());
            if let Ok(_) = level::save_level_to_file(&path, &level_data) {
                editor.last_saved_hash = compute_level_hash(&game.engine.world);
                editor.toast(format!("Saved level with {} solution(s) to {}", sol_count, path));
            }
            return;
        }
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

    #[test]
    fn modal_and_escape_reversion_test() {
        let mut editor = EditorState::default();
        // Floor layer Z=-1 is locked by default
        assert!(editor.is_layer_locked(-1));

        // Open modal and close it
        editor.floorplan_open = true;
        assert!(editor.close_modals());
        assert!(!editor.floorplan_open);
        assert!(!editor.close_modals());

        // In placement mode, clearing reverts to select mode
        editor.selected_kind = Some(BlockKind::Wall);
        editor.select_single(BodyId(5));
        assert_eq!(editor.selected_body_ids.len(), 1);

        // First escape clears selection
        editor.clear_selection();
        assert!(editor.selected_body_ids.is_empty());
        assert!(editor.selected_kind.is_some());

        // Next escape reverts tool to select mode
        editor.selected_kind = None;
        assert!(editor.selected_kind.is_none());
    }

    #[test]
    fn drag_threshold_test() {
        let mut editor = EditorState::default();
        let start_pos = Vec2::new(100.0, 100.0);
        editor.drag_start_cursor = Some(start_pos);
        editor.drag_active = false;

        // Micro-jitter during click (e.g. 5 pixels away)
        let micro_jitter = Vec2::new(103.0, 104.0);
        assert!(micro_jitter.distance(start_pos) < MIN_BLOCK_DRAG_PIXELS);

        // Real intentional drag (e.g. 25 pixels away)
        let intentional_drag = Vec2::new(120.0, 115.0);
        assert!(intentional_drag.distance(start_pos) >= MIN_BLOCK_DRAG_PIXELS);
    }

    #[test]
    fn can_move_selection_by_test() {
        let mut world = World::new();
        let b1 = world.spawn(BlockKind::Mirror, IVec3::new(1, 1, 0), vec![IVec3::ZERO]);
        let b2 = world.spawn(BlockKind::Pushable, IVec3::new(2, 1, 0), vec![IVec3::ZERO]);
        let _obstacle = world.spawn(BlockKind::Wall, IVec3::new(4, 1, 0), vec![IVec3::ZERO]);

        let sel = vec![b1, b2];

        // Move group by (+1, 0, 0) -> b1 moves to (2, 1), b2 moves to (3, 1) - no obstacle collision!
        assert!(can_move_selection_by(&world, &sel, IVec3::new(1, 0, 0)));

        // Move group by (+2, 0, 0) -> b2 would move to (4, 1), colliding with obstacle!
        assert!(!can_move_selection_by(&world, &sel, IVec3::new(2, 0, 0)));

        // Move group in Y direction (+0, +2, 0) -> valid
        assert!(can_move_selection_by(&world, &sel, IVec3::new(0, 2, 0)));
    }

    #[test]
    fn stack_mode_floor_z_display_test() {
        let mut editor = EditorState::default();
        assert_eq!(editor.z_mode, ZPlacementMode::StackOnTop);
        assert_eq!(editor.floorplan_z, -1);

        // Floor Z is view-only in stack mode and comes from floorplan_z
        editor.floorplan_z = -2;
        assert_eq!(editor.floorplan_z, -2);
    }

    #[test]
    fn save_as_dialog_and_unsaved_changes_test() {
        let mut editor = EditorState::default();
        let mut world = World::new();
        editor.last_saved_hash = compute_level_hash(&world);

        // Initially no unsaved changes
        assert!(!editor.has_unsaved_changes(&world));

        // Spawning a block causes unsaved changes
        world.spawn(BlockKind::Pushable, IVec3::new(1, 2, 0), vec![IVec3::ZERO]);
        assert!(editor.has_unsaved_changes(&world));

        // Open Save As dialog
        editor.save_as_open = true;
        editor.save_as_filename = "my_custom_level".to_string();

        // Modals close properly on close_modals
        assert!(editor.close_modals());
        assert!(!editor.save_as_open);

        // Open Unsaved Confirm dialog
        editor.unsaved_confirm_open = true;
        assert!(editor.close_modals());
        assert!(!editor.unsaved_confirm_open);

        // Test key code char mapping
        assert_eq!(key_code_to_char(KeyCode::KeyA, false), Some('a'));
        assert_eq!(key_code_to_char(KeyCode::KeyA, true), Some('A'));
        assert_eq!(key_code_to_char(KeyCode::Digit1, false), Some('1'));
        assert_eq!(key_code_to_char(KeyCode::Minus, true), Some('_'));
        assert_eq!(key_code_to_char(KeyCode::Period, false), Some('.'));
    }

    #[test]
    fn file_picker_and_unsaved_flow_test() {
        let mut editor = EditorState::default();
        assert!(!editor.file_picker_open);
        assert_eq!(editor.file_picker_dir, "levels");

        // Opening file picker
        editor.open_file_picker();
        assert!(editor.file_picker_open);
        assert!(editor.file_picker_dirty);

        // Closing modals
        assert!(editor.close_modals());
        assert!(!editor.file_picker_open);

        // Unsaved action routing
        editor.unsaved_action = UnsavedAction::OpenLevel;
        editor.unsaved_confirm_open = true;
        assert_eq!(editor.unsaved_action, UnsavedAction::OpenLevel);

        assert!(editor.close_modals());
        assert!(!editor.unsaved_confirm_open);
    }

    #[test]
    fn solution_picker_and_validation_test() {
        let mut editor = EditorState::default();
        let mut world = World::new();
        // Laser at (2, 0, 0) shooting +Y
        world.spawn(BlockKind::LaserSource, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        // Pushable mirror at (1, 2, 0) reflecting +Y -> +X
        world.spawn(BlockKind::Mirror, IVec3::new(1, 2, 0), vec![IVec3::ZERO]);
        // Goal at (5, 2, 0)
        let gid = world.spawn(BlockKind::Goal, IVec3::new(5, 2, 0), vec![IVec3::ZERO]);
        world.body_mut(gid).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);
        // Player at (0, 2, 0) facing East (+X)
        let pid = world.spawn(BlockKind::Player, IVec3::new(0, 2, 0), vec![IVec3::ZERO]);
        world.body_mut(pid).unwrap().orientation = crate::sim::CubeRot::from_facing_2d(IVec3::X);

        // Solution 1: valid 1-step push into laser beam
        editor.solutions.push(crate::level::LevelSolution {
            name: "Push Mirror Win".into(),
            actions: vec![PlayerAction::Forward],
        });
        // Solution 2: invalid sequence that never wins
        editor.solutions.push(crate::level::LevelSolution {
            name: "Invalid Turn".into(),
            actions: vec![PlayerAction::TurnLeft],
        });

        assert_eq!(editor.solutions.len(), 2);

        // Validation test: retain only valid solutions
        editor.solutions.retain(|s| level::validate_solution(&world, &s.actions));
        assert_eq!(editor.solutions.len(), 1);
        assert_eq!(editor.solutions[0].name, "Push Mirror Win");

        // Solution picker modal opening and closing
        editor.solution_picker_open = true;
        assert!(editor.close_modals());
        assert!(!editor.solution_picker_open);
    }

    #[test]
    fn editor_undo_redo_test() {
        let mut editor = EditorState::default();
        let mut engine = TurnEngine::new(World::new());

        assert!(!editor.can_undo());
        assert!(!editor.can_redo());

        // 1. Initial state: 0 bodies
        assert_eq!(engine.world.bodies().len(), 0);

        // 2. Action 1: Place a Wall at (1, 1, 0)
        editor.push_undo_snapshot(&engine.world, "Place Wall");
        let w1 = engine.world.spawn(BlockKind::Wall, IVec3::new(1, 1, 0), vec![IVec3::ZERO]);
        engine.world.sync_grid();
        engine.update_authoring_world(engine.world.clone());
        editor.select_single(w1);

        assert_eq!(engine.world.bodies().len(), 1);
        assert!(editor.can_undo());
        assert!(!editor.can_redo());
        assert_eq!(editor.undo_stack.len(), 1);

        // 3. Action 2: Rotate Wall CW
        editor.push_undo_snapshot(&engine.world, "Rotate Wall CW");
        if let Some(b) = engine.world.body_mut(w1) {
            b.orientation = b.orientation.rot_world_z_cw();
        }
        engine.world.sync_grid();
        engine.update_authoring_world(engine.world.clone());

        assert_eq!(editor.undo_stack.len(), 2);
        let rot_after_action2 = engine.world.body(w1).unwrap().orientation;

        // 4. Action 3: Place a Pushable Crate at (2, 2, 0)
        editor.push_undo_snapshot(&engine.world, "Place Pushable");
        let c1 = engine.world.spawn(BlockKind::Pushable, IVec3::new(2, 2, 0), vec![IVec3::ZERO]);
        engine.world.sync_grid();
        engine.update_authoring_world(engine.world.clone());
        editor.select_single(c1);

        assert_eq!(engine.world.bodies().len(), 2);
        assert_eq!(editor.undo_stack.len(), 3);

        // 5. Undo Action 3 (Placement of Pushable)
        let undo_desc = editor.perform_undo(&mut engine);
        assert_eq!(undo_desc.as_deref(), Some("Place Pushable"));
        assert_eq!(engine.world.bodies().len(), 1);
        assert!(engine.world.body(w1).is_some());
        assert!(engine.world.body(c1).is_none());
        assert_eq!(editor.selected_body_ids, vec![w1]);
        assert!(editor.can_undo());
        assert!(editor.can_redo());

        // 6. Undo Action 2 (Rotation of Wall)
        let undo_desc2 = editor.perform_undo(&mut engine);
        assert_eq!(undo_desc2.as_deref(), Some("Rotate Wall CW"));
        assert_eq!(engine.world.bodies().len(), 1);
        assert_ne!(engine.world.body(w1).unwrap().orientation, rot_after_action2);

        // 7. Undo Action 1 (Placement of Wall)
        let undo_desc3 = editor.perform_undo(&mut engine);
        assert_eq!(undo_desc3.as_deref(), Some("Place Wall"));
        assert_eq!(engine.world.bodies().len(), 0);
        assert!(!editor.can_undo());
        assert!(editor.can_redo());
        assert_eq!(editor.redo_stack.len(), 3);

        // 8. Redo Action 1 (Restores Wall)
        let redo_desc1 = editor.perform_redo(&mut engine);
        assert_eq!(redo_desc1.as_deref(), Some("Place Wall"));
        assert_eq!(engine.world.bodies().len(), 1);
        assert!(editor.can_undo());
        assert_eq!(editor.redo_stack.len(), 2);

        // 9. New Action while Redo stack has items clears Redo stack
        editor.push_undo_snapshot(&engine.world, "Place Mirror");
        engine.world.spawn(BlockKind::Mirror, IVec3::new(4, 4, 0), vec![IVec3::ZERO]);
        engine.world.sync_grid();
        engine.update_authoring_world(engine.world.clone());

        assert_eq!(engine.world.bodies().len(), 2);
        assert!(!editor.can_redo());
        assert_eq!(editor.redo_stack.len(), 0);
    }

    #[test]
    fn copy_and_place_test() {
        let mut world = World::new();
        let mut editor = EditorState::default();

        // 1. Spawn a custom rotated, stationary Mirror in the world
        let m1 = world.spawn(BlockKind::Mirror, IVec3::new(2, 3, 0), vec![IVec3::ZERO]);
        let custom_rot = crate::sim::CubeRot::ROT_Z_270.rot_world_x_pos();
        if let Some(b) = world.body_mut(m1) {
            b.orientation = custom_rot;
            b.tags.set(TagKind::Fixed, TagValue::Unit);
        }

        // 2. Select the block in Select Mode
        editor.select_single(m1);
        assert_eq!(editor.selected_body_ids, vec![m1]);
        assert_eq!(editor.selected_kind, None);

        // 3. Execute Copy & Place
        let copied = editor.copy_and_place(m1, &world);
        assert!(copied);

        // 4. Verify Editor enters Placement Mode with identical properties
        assert_eq!(editor.selected_kind, Some(BlockKind::Mirror));
        assert_eq!(editor.placement_orientation, custom_rot);
        assert!(editor.is_fixed);
        assert!(editor.selected_body_ids.is_empty()); // Selection cleared for placement

        // 5. Place a new block with these copied settings
        let place_pos = IVec3::new(5, 5, 0);
        let m2 = world.spawn(editor.selected_kind.unwrap(), place_pos, vec![IVec3::ZERO]);
        if let Some(b) = world.body_mut(m2) {
            b.orientation = editor.placement_orientation;
            if editor.is_fixed {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
        }

        // 6. Verify placed block has the exact copied orientation and stationary status
        let b2 = world.body(m2).unwrap();
        assert_eq!(b2.kind, BlockKind::Mirror);
        assert_eq!(b2.orientation, custom_rot);
        assert!(b2.is_fixed());
    }

    #[test]
    fn placement_properties_and_orientation_transforms_test() {
        let mut editor = EditorState::default();

        // 1. Pick Laser Source tool
        editor.selected_kind = Some(BlockKind::LaserSource);
        assert_eq!(editor.placement_orientation, crate::sim::CubeRot::IDENTITY);
        assert!(!editor.is_fixed);

        // 2. Adjust placement properties (Pitch, Roll, Rot CW, Flip X, Toggle Fixed)
        editor.placement_orientation = editor.placement_orientation.rot_world_x_pos();
        editor.placement_orientation = editor.placement_orientation.rot_world_z_cw();
        editor.placement_orientation = editor.placement_orientation.reflect_x();
        editor.is_fixed = true;

        assert_ne!(editor.placement_orientation, crate::sim::CubeRot::IDENTITY);
        assert!(editor.is_fixed);

        // 3. Reset orientation
        editor.placement_orientation = crate::sim::CubeRot::IDENTITY;
        assert_eq!(editor.placement_orientation, crate::sim::CubeRot::IDENTITY);
    }

    #[test]
    fn quality_analysis_flow_and_modal_test() {
        let mut editor = EditorState::default();
        let world = crate::level::test_level();

        // 1. Analyze puzzle quality directly
        let profile = crate::solver::analyze_puzzle(&world);
        assert!(profile.is_solvable);
        assert_eq!(profile.macro_steps, 5);

        // 2. Set profile into editor state and open modal
        editor.puzzle_profile = Some(profile.clone());
        editor.quality_modal_open = true;
        editor.quality_modal_dirty = true;

        assert!(editor.quality_modal_open);
        assert!(editor.puzzle_profile.is_some());

        // 3. Select redundant blocks action if any
        if !profile.redundant_bodies.is_empty() {
            editor.selected_body_ids = profile.redundant_bodies.clone();
            assert_eq!(editor.selected_body_ids, profile.redundant_bodies);
        }
    }
}
