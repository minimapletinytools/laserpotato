//! 3D grid block placement, movement, raycast picking, floorplan manipulation, and selection gizmos.

use bevy::prelude::*;
use crate::block_types::BlockKind;
use crate::camera;
use crate::editor::raycast::{raycast_plane_at_z, raycast_stack_on_top};
use crate::editor::{AppMode, EditorState, Palette3dPreview, ZPlacementMode};
use crate::level::{self, compute_level_hash, LevelData};
use crate::sim::{BodyId, World};
use crate::GameState;

/// Helper to check if a body can be moved to a target anchor without colliding with other bodies.
pub fn can_move_body_to(world: &World, body_id: BodyId, target_anchor: IVec3) -> bool {
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

    let level_data = LevelData::from_world_with_solutions_and_profile(
        "Custom Level",
        world,
        editor.solutions.clone(),
        editor.puzzle_profile.clone(),
    );
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

/// System to update the 3D rotating preview block in the palette.
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

/// System to draw 3D selection outlines, hovered cell indicators, and box selection lines.
pub fn draw_editor_selection_gizmos(
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

/// System handling mouse interaction (click, drag, box select) in the 3D grid viewport.
pub fn editor_grid_interaction_system(
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
                            editor.toast(format!("Moved Player to ({}, {}, {})", place_target.x, place_target.y, place_target.z));
                        }
                    } else {
                        let id = game.engine.world.spawn(BlockKind::Player, place_target, vec![IVec3::ZERO]);
                        if let Some(p) = game.engine.world.body_mut(id) {
                            p.orientation = editor.placement_orientation;
                        }
                        game.engine.world.sync_grid();
                        let new_world = game.engine.world.clone();
                        game.engine.update_authoring_world(new_world);
                        editor.clear_selection();
                        editor.toast(format!("Placed Player at ({}, {}, {})", place_target.x, place_target.y, place_target.z));
                    }
                } else {
                    let mut shape = vec![IVec3::ZERO];
                    if kind == BlockKind::LaserSource {
                        shape.push(IVec3::Y);
                    }
                    let id = game.engine.world.spawn(kind, place_target, shape);
                    if let Some(body) = game.engine.world.body_mut(id) {
                        body.orientation = editor.placement_orientation;
                        if is_fixed {
                            body.tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);
                        }
                    }
                    game.engine.world.sync_grid();
                    let new_world = game.engine.world.clone();
                    game.engine.update_authoring_world(new_world);
                    editor.select_single(id);
                    editor.toast(format!("Placed {:?} at ({}, {}, {})", kind, place_target.x, place_target.y, place_target.z));
                }
                editor.cached_solution = None;
            }
        } else {
            // ===============================================================
            // SELECT-ONLY MODE: Click / Drag movement or Box select
            // ===============================================================
            if let Some(hit_id) = clicked_body_id {
                let is_already_selected = editor.is_selected(hit_id);
                if shift_held {
                    editor.toggle_selection(hit_id);
                } else if !is_already_selected {
                    editor.select_single(hit_id);
                }

                editor.dragging_body_id = Some(hit_id);
                editor.drag_start_cursor = Some(cursor_pos);
                editor.drag_origin_cell = Some(cell_pos);
                editor.drag_active = false;
                editor.drag_start_world = Some(game.engine.world.clone());
            } else {
                if !shift_held {
                    editor.clear_selection();
                }
                if editor.z_mode == ZPlacementMode::FixedLayer {
                    editor.box_select_start = Some(cursor_pos);
                    editor.box_select_active = true;
                }
            }
        }
    }

    // 3. Mouse Dragging / Movement
    if mouse_button.pressed(MouseButton::Left) && editor.selected_kind.is_none() {
        if let (Some(drag_id), Some(start_cursor), Some(origin_cell)) = (
            editor.dragging_body_id,
            editor.drag_start_cursor,
            editor.drag_origin_cell,
        ) {
            let cursor_dist = (cursor_pos - start_cursor).length();
            if cursor_dist > 5.0 || editor.drag_active {
                if !editor.drag_active {
                    editor.drag_active = true;
                    if let Some(initial_world) = editor.drag_start_world.clone() {
                        let count = editor.selected_body_ids.len();
                        if count > 1 {
                            editor.push_undo_snapshot(&initial_world, format!("Move {} Selected Blocks", count));
                        } else if let Some(b) = initial_world.body(drag_id) {
                            editor.push_undo_snapshot(&initial_world, format!("Move {:?}", b.kind));
                        }
                    }
                }

                let delta = cell_pos - origin_cell;
                if delta != IVec3::ZERO {
                    let has_multi_selection = editor.selected_body_ids.len() > 1 && editor.is_selected(drag_id);
                    let mut moved = false;

                    if has_multi_selection {
                        if let Some(initial_world) = &editor.drag_start_world {
                            if can_move_selection_by(initial_world, &editor.selected_body_ids, delta) {
                                let mut temp_world = initial_world.clone();
                                for &id in &editor.selected_body_ids {
                                    if let Some(body) = temp_world.body_mut(id) {
                                        body.anchor += delta;
                                    }
                                }
                                temp_world.sync_grid();
                                game.engine.update_authoring_world(temp_world);
                                moved = true;
                            }
                        }
                    } else {
                        if let Some(initial_world) = &editor.drag_start_world {
                            if let Some(body) = initial_world.body(drag_id) {
                                let target_anchor = body.anchor + delta;
                                if can_move_body_to(initial_world, drag_id, target_anchor) {
                                    let mut temp_world = initial_world.clone();
                                    if let Some(b) = temp_world.body_mut(drag_id) {
                                        b.anchor = target_anchor;
                                    }
                                    temp_world.sync_grid();
                                    game.engine.update_authoring_world(temp_world);
                                    moved = true;
                                }
                            }
                        }
                    }

                    if moved {
                        editor.cached_solution = None;
                    }
                }
            }
        }
    }

    // Release Left Click
    if mouse_button.just_released(MouseButton::Left) {
        if editor.selected_kind.is_none() {
            if editor.box_select_active {
                if let (Some(start_cursor), Ok((camera, camera_transform))) = (editor.box_select_start, camera_query.single()) {
                    if let (Some(start_cell), Some(end_cell)) = (
                        raycast_plane_at_z(camera, camera_transform, start_cursor, editor.current_z),
                        raycast_plane_at_z(camera, camera_transform, cursor_pos, editor.current_z),
                    ) {
                        let min_x = start_cell.x.min(end_cell.x);
                        let max_x = start_cell.x.max(end_cell.x);
                        let min_y = start_cell.y.min(end_cell.y);
                        let max_y = start_cell.y.max(end_cell.y);

                        let mut boxed_ids = Vec::new();
                        for body in game.engine.world.bodies() {
                            if !editor.is_layer_locked(body.anchor.z) {
                                for cell in body.world_cells() {
                                    if cell.z == editor.current_z && cell.x >= min_x && cell.x <= max_x && cell.y >= min_y && cell.y <= max_y {
                                        if !boxed_ids.contains(&body.id) {
                                            boxed_ids.push(body.id);
                                        }
                                    }
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
