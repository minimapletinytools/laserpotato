//! Level Editor module for *Laser Potato*.
//!
//! Provides palette selection, property toggles with validation, 3D grid
//! raycasting and placement, block inspector, folderized level management,
//! background thread solver integration, and seamless transitions to playtest
//! and solution replay modes.

pub mod camera;
pub mod history;
pub mod interactions;
pub mod placement;
pub mod raycast;
pub mod solver_poll;
pub mod ui;

pub use history::*;
pub use interactions::*;
pub use placement::*;
pub use raycast::*;
pub use solver_poll::*;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use bevy::prelude::*;
use glam::IVec3;

use crate::block_types::BlockKind;
use crate::level::compute_level_hash;
use crate::sim::{BodyId, TagKind, TagValue, World};
use crate::solver::SolveResult;
use crate::turn::{PlayerAction, TurnEngine};

pub const MIN_BLOCK_DRAG_PIXELS: f32 = 12.0;

// ---------------------------------------------------------------------------
// App Modes
// ---------------------------------------------------------------------------

/// High-level application mode.
#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AppMode {
    #[default]
    Editor,
    LevelTester,
    Playtest,
    Playback,
}

// ---------------------------------------------------------------------------
// Editor Actions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAction {
    NewLevel,
    SaveLevel,
    SaveAsLevel,
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
    EnterLevelTester,
    TesterOpenInEditor,
    TesterPlay,
    TesterPlaySolution,
    TesterDelete,
    TesterComment,
    TesterPromote,
    TesterExit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TesterSortColumn {
    #[default]
    Name,
    MacroMoves,
    AtomicTurns,
    Epiphany,
    Size,
    Blocks,
    LoadBearing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TesterSortDirection {
    #[default]
    Ascending,
    Descending,
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
    /// Top item scroll offset for the file picker list.
    pub file_picker_scroll_offset: usize,
    /// Whether user is actively dragging the scrollbar thumb/track.
    pub file_picker_drag_scrolling: bool,
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
    /// Flag indicating that the quality modal contents need refreshing in the UI.
    pub quality_modal_dirty: bool,
    /// Whether the most recent playtest run reached the goal (won).
    pub playtest_win_recorded: bool,
    /// Playback speed multiplier (1.0 = normal, 0.5 = half speed, 2.0 = double).
    pub playback_speed: f32,
    /// Saved frame-0* world state before starting playtest, restored on Return.
    pub backup_world: Option<World>,
    /// Destination mode to return to upon pressing Escape in Playtest/Playback.
    pub return_mode: AppMode,
    /// Level tester active directory.
    pub tester_dir: String,
    /// Level tester scanned level entries.
    pub tester_entries: Vec<crate::level::TesterLevelEntry>,
    /// Currently selected level path in tester mode.
    pub tester_selected_path: Option<String>,
    /// Set of multi-selected level paths for bulk actions.
    pub tester_bulk_selected: std::collections::HashSet<String>,
    /// Column currently used for sorting tester entries.
    pub tester_sort_col: TesterSortColumn,
    /// Current sort order direction.
    pub tester_sort_dir: TesterSortDirection,
    /// Whether the table is horizontally expanded with extra stats columns.
    pub tester_expanded: bool,
    /// Current vertical scroll row index offset.
    pub tester_scroll_offset: usize,
    /// Current horizontal scroll offset in pixels (for expanded columns).
    pub tester_h_scroll_offset: f32,
    /// Whether user is actively drag-scrolling the vertical scrollbar.
    pub tester_drag_scrolling: bool,
    /// Flag indicating that the tester table UI needs full refresh.
    pub tester_dirty: bool,
    /// Tester comment dialog open state.
    pub tester_comment_modal_open: bool,
    /// Tester comment text buffer.
    pub tester_comment_buffer: String,
    /// Tester promote dialog open state.
    pub tester_promote_modal_open: bool,
    /// Tester promote title buffer.
    pub tester_promote_title_buffer: String,
    /// Tester promote destination filename buffer.
    pub tester_promote_filename_buffer: String,
    /// Tester promote destination directory.
    pub tester_promote_dest_dir: String,
    /// Tester delete confirmation modal open state.
    pub tester_delete_modal_open: bool,
    /// Temporary toast message shown on the bottom status bar: (text, timer).
    pub status_message: Option<(String, Timer)>,
    /// Undo snapshot stack (max 100 entries).
    pub undo_stack: Vec<EditorSnapshot>,
    /// Redo snapshot stack.
    pub redo_stack: Vec<EditorSnapshot>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut locked_z_layers = std::collections::HashSet::new();
        locked_z_layers.insert(-1); // Default locked floor layer
        Self {
            selected_kind: None, // Starts in Select-Only mode
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
            file_picker_scroll_offset: 0,
            file_picker_drag_scrolling: false,
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
            return_mode: AppMode::Editor,
            tester_dir: String::from("levels/mined"),
            tester_entries: Vec::new(),
            tester_selected_path: None,
            tester_bulk_selected: std::collections::HashSet::new(),
            tester_sort_col: TesterSortColumn::Name,
            tester_sort_dir: TesterSortDirection::Ascending,
            tester_expanded: false,
            tester_scroll_offset: 0,
            tester_h_scroll_offset: 0.0,
            tester_drag_scrolling: false,
            tester_dirty: true,
            tester_comment_modal_open: false,
            tester_comment_buffer: String::new(),
            tester_promote_modal_open: false,
            tester_promote_title_buffer: String::new(),
            tester_promote_filename_buffer: String::new(),
            tester_promote_dest_dir: String::from("levels"),
            tester_delete_modal_open: false,
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
            self.selected_body_ids.clear();
            true
        } else {
            false
        }
    }

    /// Push an undo snapshot before mutating the world. Clears the redo stack.
    pub fn push_undo_snapshot(&mut self, world: &World, description: impl Into<String>) {
        let snapshot = EditorSnapshot::new(world, self.selected_body_ids.clone(), description);
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
        let current = EditorSnapshot::new(&engine.world, self.selected_body_ids.clone(), prev.description.clone());
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
        let current = EditorSnapshot::new(&engine.world, self.selected_body_ids.clone(), next.description.clone());
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
    }

    /// Returns the ground plane Z coordinate for raycasting in StackOnTop mode.
    pub fn stage_ground_z(&self) -> i32 {
        0
    }

    /// Check if the level has unsaved modifications since the last load / save.
    pub fn has_unsaved_changes(&self, world: &World) -> bool {
        compute_level_hash(world) != self.last_saved_hash
    }

    /// Reset world to a fresh blank room and reset state.
    pub fn create_new_blank_room(&mut self, engine: &mut TurnEngine) {
        let mut world = World::new();
        // Create 10x10 floor layer at Z = -1
        for x in 0..10 {
            for y in 0..10 {
                let id = world.spawn(BlockKind::Floor, IVec3::new(x, y, -1), vec![IVec3::ZERO]);
                if let Some(b) = world.body_mut(id) {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
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
        self.file_picker_scroll_offset = 0;
        self.file_picker_drag_scrolling = false;
        if self.file_picker_dir.is_empty() {
            self.file_picker_dir = "levels".to_string();
        }
        self.toast("Browsing level files...");
    }

    /// Whether any modal dialog is currently open in the editor.
    pub fn is_modal_open(&self) -> bool {
        self.floorplan_open
            || self.save_as_open
            || self.unsaved_confirm_open
            || self.file_picker_open
            || self.solution_picker_open
            || self.quality_modal_open
            || self.tester_comment_modal_open
            || self.tester_promote_modal_open
            || self.tester_delete_modal_open
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
        if self.quality_modal_open {
            self.quality_modal_open = false;
            closed = true;
        }
        if self.tester_comment_modal_open {
            self.tester_comment_modal_open = false;
            closed = true;
        }
        if self.tester_promote_modal_open {
            self.tester_promote_modal_open = false;
            closed = true;
        }
        if self.tester_delete_modal_open {
            self.tester_delete_modal_open = false;
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
                    ui::update_tester_ui_system,
                    ui::update_tester_table_ui_system,
                ),
            )
            .add_systems(
                Update,
                (
                    placement::update_palette_3d_preview,
                    solver_poll::background_solver_poll_system,
                    placement::draw_editor_selection_gizmos,
                    interactions::editor_keyboard_shortcuts_system,
                    solver_poll::toast_decay_system,
                    interactions::editor_button_clicks_system,
                ),
            )
            .add_systems(
                Update,
                (
                    placement::editor_grid_interaction_system,
                    interactions::file_picker_button_clicks_system,
                    interactions::file_picker_keyboard_and_wheel_system,
                    interactions::file_picker_scrollbar_drag_system,
                    interactions::solution_picker_button_clicks_system,
                    ui::quality_modal_interaction_system,
                )
                    .run_if(in_state(AppMode::Editor)),
            )
            .add_systems(
                Update,
                (
                    interactions::tester_table_interaction_system,
                    interactions::tester_scrollbar_drag_system,
                    interactions::tester_modal_interaction_system,
                    interactions::tester_keyboard_shortcuts_system,
                )
                    .run_if(in_state(AppMode::LevelTester)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level;

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
        assert_eq!(crate::editor::ui::widgets::key_code_to_char(KeyCode::KeyA, false), Some('a'));
        assert_eq!(crate::editor::ui::widgets::key_code_to_char(KeyCode::KeyA, true), Some('A'));
        assert_eq!(crate::editor::ui::widgets::key_code_to_char(KeyCode::Digit1, false), Some('1'));
        assert_eq!(crate::editor::ui::widgets::key_code_to_char(KeyCode::Minus, true), Some('_'));
        assert_eq!(crate::editor::ui::widgets::key_code_to_char(KeyCode::Period, false), Some('.'));
    }

    #[test]
    fn file_picker_and_unsaved_flow_test() {
        let mut editor = EditorState::default();
        assert!(!editor.file_picker_open);
        assert!(!editor.is_modal_open());
        assert_eq!(editor.file_picker_dir, "levels");
        assert_eq!(editor.file_picker_scroll_offset, 0);

        // Opening file picker
        editor.file_picker_scroll_offset = 15;
        editor.open_file_picker();
        assert!(editor.file_picker_open);
        assert!(editor.is_modal_open());
        assert!(editor.file_picker_dirty);
        assert_eq!(editor.file_picker_scroll_offset, 0); // Reset to top

        // Scrolling offset changes
        editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_add(5);
        assert_eq!(editor.file_picker_scroll_offset, 5);
        editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_sub(2);
        assert_eq!(editor.file_picker_scroll_offset, 3);

        // Closing modals
        assert!(editor.close_modals());
        assert!(!editor.file_picker_open);
        assert!(!editor.is_modal_open());

        // Unsaved action routing
        editor.unsaved_action = UnsavedAction::OpenLevel;
        editor.unsaved_confirm_open = true;
        assert_eq!(editor.unsaved_action, UnsavedAction::OpenLevel);
        assert!(editor.is_modal_open());

        assert!(editor.close_modals());
        assert!(!editor.unsaved_confirm_open);
        assert!(!editor.is_modal_open());
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
        editor.solutions.push(crate::level::LevelSolution::new(
            "Push Mirror Win",
            vec![PlayerAction::Forward],
        ));
        // Solution 2: invalid sequence that never wins
        editor.solutions.push(crate::level::LevelSolution::new(
            "Invalid Turn",
            vec![PlayerAction::TurnLeft],
        ));

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
        editor.is_fixed = false;
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

    #[test]
    fn level_tester_state_and_table_flow_test() {
        let mut editor = EditorState::default();
        assert_eq!(editor.tester_dir, "levels/mined");
        assert_eq!(editor.tester_sort_col, TesterSortColumn::Name);
        assert_eq!(editor.tester_sort_dir, TesterSortDirection::Ascending);
        assert!(!editor.tester_expanded);

        // 1. Create dummy entries with diverse stats
        let mut entries = vec![
            crate::level::TesterLevelEntry {
                path: "levels/mined/puzzle_c.json".into(),
                filename: "puzzle_c.json".into(),
                name: "Alpha Puzzle".into(),
                description: "Tricky reflection step".into(),
                macro_steps: 12,
                atomic_turns: 30,
                epiphany: 7.5,
                width: 6,
                height: 6,
                depth: 1,
                mirrors: 2,
                crates: 1,
                polyominos: 0,
                lasers: 1,
                goals: 1,
                total_blocks: 8,
                load_bearing_pct: 100.0,
                has_comment: true,
            },
            crate::level::TesterLevelEntry {
                path: "levels/mined/puzzle_a.json".into(),
                filename: "puzzle_a.json".into(),
                name: "Gamma Puzzle".into(),
                description: "".into(),
                macro_steps: 5,
                atomic_turns: 10,
                epiphany: 3.0,
                width: 8,
                height: 8,
                depth: 2,
                mirrors: 4,
                crates: 2,
                polyominos: 1,
                lasers: 1,
                goals: 2,
                total_blocks: 14,
                load_bearing_pct: 85.0,
                has_comment: false,
            },
            crate::level::TesterLevelEntry {
                path: "levels/mined/puzzle_b.json".into(),
                filename: "puzzle_b.json".into(),
                name: "Beta Puzzle".into(),
                description: "".into(),
                macro_steps: 22,
                atomic_turns: 54,
                epiphany: 15.0,
                width: 5,
                height: 5,
                depth: 1,
                mirrors: 2,
                crates: 0,
                polyominos: 0,
                lasers: 1,
                goals: 1,
                total_blocks: 6,
                load_bearing_pct: 100.0,
                has_comment: false,
            },
        ];

        // 2. Test sorting by MacroMoves Ascending & Descending
        crate::editor::ui::sort_tester_entries(&mut entries, TesterSortColumn::MacroMoves, TesterSortDirection::Ascending);
        assert_eq!(entries[0].macro_steps, 5);
        assert_eq!(entries[1].macro_steps, 12);
        assert_eq!(entries[2].macro_steps, 22);

        crate::editor::ui::sort_tester_entries(&mut entries, TesterSortColumn::MacroMoves, TesterSortDirection::Descending);
        assert_eq!(entries[0].macro_steps, 22);
        assert_eq!(entries[1].macro_steps, 12);
        assert_eq!(entries[2].macro_steps, 5);

        // 3. Test sorting by Epiphany Score
        crate::editor::ui::sort_tester_entries(&mut entries, TesterSortColumn::Epiphany, TesterSortDirection::Descending);
        assert_eq!(entries[0].name, "Beta Puzzle");

        // 4. Test sorting by Name Ascending
        crate::editor::ui::sort_tester_entries(&mut entries, TesterSortColumn::Name, TesterSortDirection::Ascending);
        assert_eq!(entries[0].name, "Alpha Puzzle");
        assert_eq!(entries[1].name, "Beta Puzzle");
        assert_eq!(entries[2].name, "Gamma Puzzle");

        // 5. Test Bulk Selection logic
        assert!(editor.tester_bulk_selected.is_empty());
        editor.tester_entries = entries.clone();

        // Select all
        for e in &editor.tester_entries {
            editor.tester_bulk_selected.insert(e.path.clone());
        }
        assert_eq!(editor.tester_bulk_selected.len(), 3);

        // Toggle / Deselect one
        editor.tester_bulk_selected.remove("levels/mined/puzzle_a.json");
        assert_eq!(editor.tester_bulk_selected.len(), 2);
        assert!(!editor.tester_bulk_selected.contains("levels/mined/puzzle_a.json"));

        // 6. Test Promote & Comment buffers
        editor.tester_selected_path = Some("levels/mined/puzzle_c.json".into());
        editor.tester_comment_buffer = "Needs higher laser obstacle".into();
        assert_eq!(editor.tester_comment_buffer, "Needs higher laser obstacle");

        editor.tester_promote_title_buffer = "Laser Gauntlet 1".into();
        editor.tester_promote_filename_buffer = "gauntlet_1.json".into();
        assert_eq!(editor.tester_promote_dest_dir, "levels");
        assert_eq!(editor.tester_promote_title_buffer, "Laser Gauntlet 1");
        assert_eq!(editor.tester_promote_filename_buffer, "gauntlet_1.json");
    }
}
