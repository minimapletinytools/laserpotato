//! Button click handlers, keyboard shortcuts, modal interactions, and table interactions.

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::Duration;
use bevy::prelude::*;
use crate::block_types::BlockKind;
use crate::camera;
use crate::editor::placement::execute_save_as;
use crate::editor::ui::widgets::key_code_to_char;
use crate::editor::{ui, AppMode, EditorAction, EditorState, TesterSortDirection, UnsavedAction, ZPlacementMode};
use crate::level::{self, compute_level_hash, LevelData};
use crate::sim::{TagKind, TagValue};
use crate::turn::TurnEngine;
use crate::solver::{self, SolverConfig};
use crate::GameState;

/// Main button click dispatcher for editor controls, top bar actions, palette, and inspector.
pub fn editor_button_clicks_system(
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
            editor.floorplan_width = (editor.floorplan_width - 1).max(3);
        }
        if fp_w_inc.is_some() {
            editor.floorplan_width = (editor.floorplan_width + 1).min(50);
        }
        if fp_h_dec.is_some() {
            editor.floorplan_height = (editor.floorplan_height - 1).max(3);
        }
        if fp_h_inc.is_some() {
            editor.floorplan_height = (editor.floorplan_height + 1).min(50);
        }
        if fp_z_dec.is_some() {
            editor.floorplan_z = (editor.floorplan_z - 1).max(-10);
        }
        if fp_z_inc.is_some() {
            editor.floorplan_z = (editor.floorplan_z + 1).min(10);
        }
        if fp_fill.is_some() {
            let (w, h, z) = (editor.floorplan_width, editor.floorplan_height, editor.floorplan_z);
            editor.push_undo_snapshot(&game.engine.world, format!("Fill {}x{} Floor (Z={})", w, h, z));
            crate::editor::placement::fill_floorplan(&mut game.engine.world, w, h, z);
            let new_world = game.engine.world.clone();
            game.engine.update_authoring_world(new_world);
            editor.cached_solution = None;
            editor.toast(format!("Filled {}x{} floor at Z={}", w, h, z));
        }
        if fp_lock_toggle.is_some() {
            let z = editor.floorplan_z;
            if editor.locked_z_layers.contains(&z) {
                editor.locked_z_layers.remove(&z);
                editor.toast(format!("Unlocked layer Z={}", z));
            } else {
                editor.locked_z_layers.insert(z);
                editor.toast(format!("Locked layer Z={}", z));
            }
        }
        if fp_close.is_some() {
            editor.floorplan_open = false;
        }

        // 6. Action Bar buttons
        if let Some(action_btn) = action_btn {
            match action_btn.0 {
                EditorAction::NewLevel => {
                    if editor.has_unsaved_changes(&game.engine.world) {
                        editor.unsaved_action = UnsavedAction::NewLevel;
                        editor.unsaved_confirm_open = true;
                    } else {
                        editor.create_new_blank_room(&mut game.engine);
                    }
                }
                EditorAction::OpenLevel => {
                    if editor.has_unsaved_changes(&game.engine.world) {
                        editor.unsaved_action = UnsavedAction::OpenLevel;
                        editor.unsaved_confirm_open = true;
                    } else {
                        editor.open_file_picker();
                    }
                }
                EditorAction::SaveLevel => {
                    let path = editor.current_level_path.clone();
                    editor.solutions.retain(|s| level::validate_solution(&game.engine.world, &s.actions));
                    let sol_count = editor.solutions.len();
                    let level_data = LevelData::from_world_with_solutions_and_profile(
                        "Custom Level",
                        &game.engine.world,
                        editor.solutions.clone(),
                        editor.puzzle_profile.clone(),
                    );
                    match level::save_level_to_file(&path, &level_data) {
                        Ok(_) => {
                            editor.last_saved_hash = compute_level_hash(&game.engine.world);
                            editor.toast(format!("Saved level with {} solution(s) to {}", sol_count, path));
                        }
                        Err(e) => {
                            editor.toast(format!("Save error: {}", e));
                        }
                    }
                }
                EditorAction::SaveAsLevel => {
                    let default_name = std::path::Path::new(&editor.current_level_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("custom_puzzle.json")
                        .to_string();
                    editor.save_as_filename = default_name;
                    editor.save_as_open = true;
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
                                editor.solutions.push(crate::level::LevelSolution::new(
                                    "Optimal Solver Solution",
                                    sol.clone(),
                                ));
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
                    editor.solver_status = "Analyzing quality in background...".into();
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
                            editor.return_mode = AppMode::Editor;
                            next_mode.set(AppMode::Playtest);
                        }
                        Err(err) => {
                            editor.toast(format!("Cannot playtest: {}", err));
                        }
                    }
                }
                EditorAction::TestWithSolution => {
                    if editor.solutions.is_empty() {
                        let current_hash = compute_level_hash(&game.engine.world);
                        if let Some((cached_hash, sol)) = editor.cached_solution.clone() {
                            if cached_hash == current_hash {
                                editor.solutions.push(crate::level::LevelSolution::new(
                                    "Recorded Solution",
                                    sol,
                                ));
                            }
                        }
                    }

                    if !editor.solutions.is_empty() {
                        editor.solution_picker_open = true;
                        editor.solution_picker_dirty = true;
                    } else {
                        editor.toast("No recorded solution available. Solve level or playtest to win first.");
                    }
                }
                EditorAction::EnterLevelTester => {
                    editor.return_mode = AppMode::LevelTester;
                    editor.tester_entries.clear();
                    editor.tester_dirty = true;
                    next_mode.set(AppMode::LevelTester);
                    editor.toast("Switched to Level Tester mode.");
                }
                EditorAction::TesterOpenInEditor => {
                    editor.return_mode = AppMode::Editor;
                    next_mode.set(AppMode::Editor);
                    editor.toast("Opened level in Level Editor.");
                }
                EditorAction::TesterPlay => {
                    if let Some(path) = &editor.tester_selected_path {
                        if let Ok(lvl) = crate::level::load_level_from_file(path) {
                            game.engine = TurnEngine::new(lvl.to_world());
                            editor.current_level_path = path.clone();
                            editor.solutions = lvl.solutions;
                            editor.last_saved_hash = compute_level_hash(&game.engine.world);
                        }
                    }
                    match game.engine.start_playtest() {
                        Ok(()) => {
                            editor.backup_world = Some(game.engine.frame_zero_star.clone());
                            editor.playtest_win_recorded = false;
                            editor.return_mode = AppMode::LevelTester;
                            next_mode.set(AppMode::Playtest);
                            editor.toast("Testing level (Press [Esc] to return to Level Tester).");
                        }
                        Err(err) => {
                            editor.toast(format!("Cannot play: {}", err));
                        }
                    }
                }
                EditorAction::TesterPlaySolution => {
                    if let Some(path) = &editor.tester_selected_path {
                        if let Ok(lvl) = crate::level::load_level_from_file(path) {
                            game.engine = TurnEngine::new(lvl.to_world());
                            editor.current_level_path = path.clone();
                            editor.solutions = lvl.solutions;
                            editor.last_saved_hash = compute_level_hash(&game.engine.world);
                        }
                    }
                    if let Some(sol) = editor.solutions.first() {
                        let actions = sol.actions.clone();
                        match game.engine.start_playtest() {
                            Ok(()) => {
                                editor.backup_world = Some(game.engine.frame_zero_star.clone());
                                editor.playtest_win_recorded = false;
                                playback.is_playback = true;
                                playback.actions = actions;
                                playback.current_index = 0;
                                playback.auto_playing = true;
                                playback.speed = editor.playback_speed;
                                playback.step_timer = Timer::from_seconds(
                                    0.40 / editor.playback_speed.max(0.1),
                                    TimerMode::Repeating,
                                );
                                editor.return_mode = AppMode::LevelTester;
                                next_mode.set(AppMode::Playback);
                                editor.toast("Replaying solution (Press [Esc] to return to Level Tester).");
                            }
                            Err(err) => {
                                editor.toast(format!("Cannot play solution: {}", err));
                            }
                        }
                    } else {
                        editor.toast("No solution recorded for this level.");
                    }
                }
                EditorAction::TesterDelete => {
                    editor.tester_delete_modal_open = true;
                }
                EditorAction::TesterComment => {
                    editor.tester_comment_modal_open = true;
                    let selected_opt = editor.tester_selected_path.clone();
                    if let Some(selected) = selected_opt {
                        if let Some(entry) = editor.tester_entries.iter().find(|e| e.path == selected) {
                            editor.tester_comment_buffer = entry.description.clone();
                        }
                    }
                }
                EditorAction::TesterPromote => {
                    editor.tester_promote_modal_open = true;
                    let selected_opt = editor.tester_selected_path.clone();
                    if let Some(selected) = selected_opt {
                        let entry_opt = editor.tester_entries.iter().find(|e| e.path == selected).cloned();
                        if let Some(entry) = entry_opt {
                            editor.tester_promote_title_buffer = entry.name;
                            editor.tester_promote_filename_buffer = entry.filename;
                        }
                    }
                }
                EditorAction::TesterExit => {
                    editor.return_mode = AppMode::Editor;
                    next_mode.set(AppMode::Editor);
                    editor.toast("Exited Level Tester mode.");
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
pub fn file_picker_button_clicks_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ui::FilePickerUpBtn>,
            Option<&ui::FilePickerDirBtn>,
            Option<&ui::FilePickerFileBtn>,
            Option<&ui::FilePickerScrollUpBtn>,
            Option<&ui::FilePickerScrollDownBtn>,
            Option<&ui::FilePickerScrollPageUpBtn>,
            Option<&ui::FilePickerScrollPageDownBtn>,
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

    for (interaction, up_btn, dir_btn, file_btn, scroll_up, scroll_down, scroll_page_up, scroll_page_down, cancel_btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(up) = up_btn {
            editor.file_picker_dir = up.0.clone();
            editor.file_picker_scroll_offset = 0;
            editor.file_picker_dirty = true;
        } else if let Some(dir) = dir_btn {
            editor.file_picker_dir = dir.0.clone();
            editor.file_picker_scroll_offset = 0;
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
                    editor.puzzle_profile = lvl.quality_profile.clone();
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
        } else if scroll_up.is_some() {
            editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_sub(1);
            editor.file_picker_dirty = true;
        } else if scroll_down.is_some() {
            editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_add(1);
            editor.file_picker_dirty = true;
        } else if scroll_page_up.is_some() {
            editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_sub(10);
            editor.file_picker_dirty = true;
        } else if scroll_page_down.is_some() {
            editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_add(10);
            editor.file_picker_dirty = true;
        } else if cancel_btn.is_some() {
            editor.file_picker_open = false;
            editor.toast("Cancelled file picker.");
        }
    }
}

/// Handles mouse wheel scrolling and keyboard navigation (Arrow Up/Down, Page Up/Down, Home/End) inside the File Picker modal.
pub fn file_picker_keyboard_and_wheel_system(
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorState>,
) {
    if !editor.file_picker_open {
        return;
    }

    let mut scroll_delta: i32 = 0;

    // 1. Mouse wheel scrolling
    for event in mouse_wheel.read() {
        if event.y > 0.0 {
            scroll_delta -= 3;
        } else if event.y < 0.0 {
            scroll_delta += 3;
        }
    }

    // 2. Keyboard scrolling shortcuts
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        scroll_delta -= 1;
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        scroll_delta += 1;
    }
    if keys.just_pressed(KeyCode::PageUp) {
        scroll_delta -= 10;
    }
    if keys.just_pressed(KeyCode::PageDown) {
        scroll_delta += 10;
    }
    if keys.just_pressed(KeyCode::Home) {
        editor.file_picker_scroll_offset = 0;
        editor.file_picker_dirty = true;
        return;
    }
    if keys.just_pressed(KeyCode::End) {
        editor.file_picker_scroll_offset = usize::MAX;
        editor.file_picker_dirty = true;
        return;
    }

    if scroll_delta != 0 {
        if scroll_delta < 0 {
            editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_sub((-scroll_delta) as usize);
        } else {
            editor.file_picker_scroll_offset = editor.file_picker_scroll_offset.saturating_add(scroll_delta as usize);
        }
        editor.file_picker_dirty = true;
    }
}

/// Handles clicking and dragging directly on the scrollbar track or thumb.
pub fn file_picker_scrollbar_drag_system(
    windows: Query<&Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    track_query: Query<(&Interaction, &GlobalTransform, &ComputedNode), With<ui::FilePickerScrollBarTrack>>,
    thumb_query: Query<&Interaction, With<ui::FilePickerScrollBarThumb>>,
    mut editor: ResMut<EditorState>,
) {
    if !editor.file_picker_open {
        editor.file_picker_drag_scrolling = false;
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    let Ok((track_interaction, track_gt, track_node)) = track_query.single() else { return };

    let thumb_pressed = thumb_query.iter().any(|i| *i == Interaction::Pressed);

    // 1. Mouse just pressed on track or thumb -> begin drag
    if *track_interaction == Interaction::Pressed || thumb_pressed || (mouse_button.just_pressed(MouseButton::Left) && *track_interaction == Interaction::Hovered) {
        editor.file_picker_drag_scrolling = true;
    }

    // 2. Mouse released -> stop drag
    if mouse_button.just_released(MouseButton::Left) {
        editor.file_picker_drag_scrolling = false;
    }

    // 3. Actively dragging or clicked on track
    if editor.file_picker_drag_scrolling && mouse_button.pressed(MouseButton::Left) {
        let track_center = track_gt.translation().xy();
        let track_height = track_node.size().y;
        if track_height > 1.0 {
            let track_top = track_center.y - track_height * 0.5;
            let relative_y = ((cursor_pos.y - track_top) / track_height).clamp(0.0, 1.0);

            let (parent_opt, entries) = crate::level::list_directory_entries(&editor.file_picker_dir);
            let total_count = (parent_opt.is_some() as usize) + entries.len();
            let visible_count = 14;
            let max_offset = total_count.saturating_sub(visible_count);

            let new_offset = (relative_y * max_offset as f32).round() as usize;
            if editor.file_picker_scroll_offset != new_offset {
                editor.file_picker_scroll_offset = new_offset;
                editor.file_picker_dirty = true;
            }
        }
    }
}

/// Handles button clicks inside the floating Solution Picker dialog.
pub fn solution_picker_button_clicks_system(
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

pub fn editor_keyboard_shortcuts_system(
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

    // Handle Escape key
    if keys.just_pressed(KeyCode::Escape) {
        if *app_mode.get() == AppMode::Playtest || *app_mode.get() == AppMode::Playback {
            game.engine.end_playtest();
            playback.is_playback = false;
            let target = editor.return_mode;
            next_mode.set(target);
            if target == AppMode::LevelTester {
                editor.tester_dirty = true;
                editor.toast("Returned to Level Tester.");
            } else {
                editor.toast("Returned to Level Editor (Frame 0*).");
            }
            return;
        } else if editor.close_modals() {
            editor.toast("Closed modal [Esc].");
            return;
        } else if *app_mode.get() == AppMode::LevelTester {
            if !editor.tester_bulk_selected.is_empty() {
                editor.tester_bulk_selected.clear();
                editor.tester_dirty = true;
                editor.toast("Cleared bulk selection [Esc].");
            }
            return;
        } else if !editor.selected_body_ids.is_empty() {
            editor.clear_selection();
            editor.toast("Cleared selection [Esc].");
        } else if editor.selected_kind.is_some() {
            editor.selected_kind = None;
            editor.toast("Reverted to Select Mode [Esc].");
        }
    }

    if keys.just_pressed(KeyCode::F2) {
        if *app_mode.get() == AppMode::Editor {
            editor.return_mode = AppMode::LevelTester;
            editor.tester_dirty = true;
            next_mode.set(AppMode::LevelTester);
            editor.toast("Switched to Level Tester mode [F2].");
            return;
        } else if *app_mode.get() == AppMode::LevelTester {
            editor.return_mode = AppMode::Editor;
            next_mode.set(AppMode::Editor);
            editor.toast("Switched to Level Editor [F2].");
            return;
        }
    }

    if *app_mode.get() != AppMode::Editor {
        return;
    }

    let cmd_held = keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight);
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let modifier_held = cmd_held || ctrl_held;
    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    // Undo / Redo & Save shortcuts
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
            let level_data = LevelData::from_world_with_solutions_and_profile(
                "Custom Level",
                &game.engine.world,
                editor.solutions.clone(),
                editor.puzzle_profile.clone(),
            );
            if let Ok(_) = level::save_level_to_file(&path, &level_data) {
                editor.last_saved_hash = compute_level_hash(&game.engine.world);
                editor.toast(format!("Saved level with {} solution(s) to {}", sol_count, path));
            }
            return;
        }
        return;
    }

    // Palette selection hotkeys
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

pub fn tester_table_interaction_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ui::TesterSortHeaderBtn>,
            Option<&ui::TesterRowSelectBtn>,
            Option<&ui::TesterRowCheckBtn>,
            Option<&ui::TesterSelectAllBtn>,
            Option<&ui::TesterTrashSelectedBtn>,
            Option<&ui::TesterExpandToggleBtn>,
            Option<&ui::TesterUpBtn>,
            Option<&ui::TesterRefreshBtn>,
            Option<&ui::TesterScrollUpBtn>,
            Option<&ui::TesterScrollDownBtn>,
            Option<&ui::TesterScrollPageUpBtn>,
            Option<&ui::TesterScrollPageDownBtn>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
) {
    for (
        interaction,
        sort_btn,
        select_row,
        check_row,
        select_all,
        trash_btn,
        expand_btn,
        up_btn,
        refresh_btn,
        scroll_up,
        scroll_down,
        scroll_page_up,
        scroll_page_down,
    ) in &mut interaction_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(sort) = sort_btn {
            if editor.tester_sort_col == sort.0 {
                editor.tester_sort_dir = match editor.tester_sort_dir {
                    TesterSortDirection::Ascending => TesterSortDirection::Descending,
                    TesterSortDirection::Descending => TesterSortDirection::Ascending,
                };
            } else {
                editor.tester_sort_col = sort.0;
                editor.tester_sort_dir = TesterSortDirection::Ascending;
            }
            editor.tester_dirty = true;
        } else if let Some(row) = select_row {
            let path = row.0.clone();
            if let Some(entry) = editor.tester_entries.iter().find(|e| e.path == path).cloned() {
                if entry.is_directory {
                    editor.tester_dir = path;
                    editor.tester_entries.clear();
                    editor.tester_scroll_offset = 0;
                    editor.tester_selected_path = None;
                    editor.tester_bulk_selected.clear();
                    editor.tester_dirty = true;
                    editor.toast(format!("Opened folder: {}", entry.filename));
                } else {
                    editor.tester_selected_path = Some(path.clone());
                    if let Ok(lvl) = crate::level::load_level_from_file(&path) {
                        game.engine = TurnEngine::new(lvl.to_world());
                        editor.current_level_path = path;
                        editor.solutions = lvl.solutions;
                        editor.last_saved_hash = compute_level_hash(&game.engine.world);
                    }
                    editor.tester_dirty = true;
                }
            }
        } else if let Some(check) = check_row {
            let path = check.0.clone();
            if editor.tester_bulk_selected.contains(&path) {
                editor.tester_bulk_selected.remove(&path);
            } else {
                editor.tester_bulk_selected.insert(path);
            }
            editor.tester_dirty = true;
        } else if select_all.is_some() {
            let file_paths: Vec<String> = editor
                .tester_entries
                .iter()
                .filter(|e| !e.is_directory)
                .map(|e| e.path.clone())
                .collect();
            let total = file_paths.len();
            if total > 0 && editor.tester_bulk_selected.len() == total {
                editor.tester_bulk_selected.clear();
            } else {
                editor.tester_bulk_selected.extend(file_paths);
            }
            editor.tester_dirty = true;
        } else if trash_btn.is_some() {
            if !editor.tester_bulk_selected.is_empty() || editor.tester_selected_path.is_some() {
                editor.tester_delete_modal_open = true;
            } else {
                editor.toast("No level selected to delete.");
            }
        } else if expand_btn.is_some() {
            editor.tester_expanded = !editor.tester_expanded;
            editor.tester_dirty = true;
        } else if let Some(_up) = up_btn {
            let current = std::path::PathBuf::from(&editor.tester_dir);
            if let Some(parent) = current.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                if !parent_str.is_empty() && parent_str != "." {
                    editor.tester_dir = parent_str;
                } else {
                    editor.tester_dir = "levels".into();
                }
            } else {
                editor.tester_dir = "levels".into();
            }
            editor.tester_entries.clear();
            editor.tester_scroll_offset = 0;
            editor.tester_selected_path = None;
            editor.tester_bulk_selected.clear();
            editor.tester_dirty = true;
        } else if refresh_btn.is_some() {
            editor.tester_entries.clear();
            editor.tester_dirty = true;
            editor.toast("Refreshed level list.");
        } else if scroll_up.is_some() {
            editor.tester_scroll_offset = editor.tester_scroll_offset.saturating_sub(1);
            editor.tester_dirty = true;
        } else if scroll_down.is_some() {
            editor.tester_scroll_offset = editor.tester_scroll_offset.saturating_add(1);
            editor.tester_dirty = true;
        } else if scroll_page_up.is_some() {
            editor.tester_scroll_offset = editor.tester_scroll_offset.saturating_sub(14);
            editor.tester_dirty = true;
        } else if scroll_page_down.is_some() {
            editor.tester_scroll_offset = editor.tester_scroll_offset.saturating_add(14);
            editor.tester_dirty = true;
        }
    }
}

pub fn tester_scrollbar_drag_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    track_query: Query<(&GlobalTransform, &Node), With<ui::TesterScrollBarTrack>>,
    thumb_query: Query<&Interaction, (With<ui::TesterScrollBarThumb>, Changed<Interaction>)>,
    mut editor: ResMut<EditorState>,
) {
    if editor.tester_entries.is_empty() {
        return;
    }

    for interaction in &thumb_query {
        if *interaction == Interaction::Pressed {
            editor.tester_drag_scrolling = true;
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        editor.tester_drag_scrolling = false;
    }

    let is_dragging = editor.tester_drag_scrolling;
    let track_clicked = mouse_buttons.just_pressed(MouseButton::Left);

    if is_dragging || track_clicked {
        let Ok(window) = windows.single() else { return };
        let Some(cursor_pos) = window.cursor_position() else { return };
        let Ok((track_gt, track_node)) = track_query.single() else { return };

        let track_translation = track_gt.translation();
        let track_top = track_translation.y;
        let track_height = if let Val::Px(h) = track_node.height {
            h
        } else {
            360.0
        };

        if track_clicked {
            let track_left = track_translation.x - 12.0;
            let track_right = track_translation.x + 12.0;
            let track_bottom = track_top + track_height;
            if cursor_pos.x < track_left
                || cursor_pos.x > track_right
                || cursor_pos.y < track_top
                || cursor_pos.y > track_bottom
            {
                if !is_dragging {
                    return;
                }
            }
        }

        let relative_y = (cursor_pos.y - track_top).clamp(0.0, track_height);
        let ratio = (relative_y / track_height).clamp(0.0, 1.0);

        let total_count = editor.tester_entries.len();
        let visible_count = 14;
        let max_offset = total_count.saturating_sub(visible_count);
        let new_offset = ((ratio * max_offset as f32).round() as usize).min(max_offset);

        if new_offset != editor.tester_scroll_offset {
            editor.tester_scroll_offset = new_offset;
            editor.tester_dirty = true;
        }
    }
}

pub fn tester_modal_interaction_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ui::TesterCommentSaveBtn>,
            Option<&ui::TesterCommentCancelBtn>,
            Option<&ui::TesterPromoteCopyBtn>,
            Option<&ui::TesterPromoteMoveBtn>,
            Option<&ui::TesterPromoteCancelBtn>,
            Option<&ui::TesterDeleteConfirmBtn>,
            Option<&ui::TesterDeleteCancelBtn>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
) {
    for (
        interaction,
        comment_save,
        comment_cancel,
        promote_copy,
        promote_move,
        promote_cancel,
        delete_confirm,
        delete_cancel,
    ) in &mut interaction_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if comment_save.is_some() {
            let selected_opt = editor.tester_selected_path.clone();
            let comment_buf = editor.tester_comment_buffer.clone();
            if let Some(selected) = selected_opt {
                if let Ok(mut lvl) = crate::level::load_level_from_file(&selected) {
                    lvl.description = Some(comment_buf.clone());
                    let _ = crate::level::save_level_to_file(&selected, &lvl);
                    if let Some(entry) = editor.tester_entries.iter_mut().find(|e| e.path == selected) {
                        entry.description = comment_buf;
                        entry.has_comment = !entry.description.trim().is_empty();
                    }
                    editor.toast("Saved comment to level file.");
                }
            }
            editor.tester_comment_modal_open = false;
            editor.tester_dirty = true;
        } else if comment_cancel.is_some() {
            editor.tester_comment_modal_open = false;
        } else if promote_copy.is_some() || promote_move.is_some() {
            let is_move = promote_move.is_some();
            let selected_opt = editor.tester_selected_path.clone();
            let title = editor.tester_promote_title_buffer.clone();
            let filename = if editor.tester_promote_filename_buffer.ends_with(".json") {
                editor.tester_promote_filename_buffer.clone()
            } else {
                format!("{}.json", editor.tester_promote_filename_buffer)
            };
            let dest_dir = editor.tester_promote_dest_dir.clone();
            if let Some(selected) = selected_opt {
                if let Ok(mut lvl) = crate::level::load_level_from_file(&selected) {
                    lvl.name = title;
                    let dest_path = format!("{}/{}", dest_dir, filename);
                    if let Ok(()) = crate::level::save_level_to_file(&dest_path, &lvl) {
                        if is_move {
                            let _ = std::fs::remove_file(&selected);
                            editor.tester_entries.retain(|e| e.path != selected);
                            if editor.tester_selected_path.as_deref() == Some(&selected) {
                                editor.tester_selected_path =
                                    editor.tester_entries.first().map(|e| e.path.clone());
                            }
                            editor.toast(format!("Promoted & moved level to '{}'.", dest_path));
                        } else {
                            editor.toast(format!("Promoted & copied level to '{}'.", dest_path));
                        }
                    }
                }
            }
            editor.tester_promote_modal_open = false;
            editor.tester_dirty = true;
        } else if promote_cancel.is_some() {
            editor.tester_promote_modal_open = false;
        } else if delete_confirm.is_some() {
            let bulk = editor.tester_bulk_selected.clone();
            if !bulk.is_empty() {
                let count = bulk.len();
                for path in &bulk {
                    let _ = std::fs::remove_file(path);
                }
                editor.tester_entries.retain(|e| !bulk.contains(&e.path));
                editor.tester_bulk_selected.clear();
                if let Some(selected) = &editor.tester_selected_path {
                    if bulk.contains(selected) {
                        editor.tester_selected_path =
                            editor.tester_entries.first().map(|e| e.path.clone());
                    }
                }
                editor.toast(format!("Permanently deleted {} level file(s).", count));
            } else if let Some(selected) = editor.tester_selected_path.clone() {
                let _ = std::fs::remove_file(&selected);
                editor.tester_entries.retain(|e| e.path != selected);
                editor.tester_selected_path =
                    editor.tester_entries.first().map(|e| e.path.clone());
                editor.toast(format!("Deleted '{}'.", selected));
            }
            editor.tester_delete_modal_open = false;
            editor.tester_dirty = true;
            if let Some(selected) = &editor.tester_selected_path {
                if let Ok(lvl) = crate::level::load_level_from_file(selected) {
                    game.engine = TurnEngine::new(lvl.to_world());
                }
            }
        } else if delete_cancel.is_some() {
            editor.tester_delete_modal_open = false;
        }
    }
}

pub fn tester_keyboard_shortcuts_system(
    app_mode: Res<State<AppMode>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut editor: ResMut<EditorState>,
    mut game: ResMut<GameState>,
    mut next_mode: ResMut<NextState<AppMode>>,
    mut playback: ResMut<crate::PlaybackState>,
) {
    if *app_mode.get() != AppMode::LevelTester {
        return;
    }

    let shift_held = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    // Modal text inputs
    if editor.tester_comment_modal_open {
        if keys.just_pressed(KeyCode::Escape) {
            editor.tester_comment_modal_open = false;
            return;
        }
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            let selected_opt = editor.tester_selected_path.clone();
            let comment_buf = editor.tester_comment_buffer.clone();
            if let Some(selected) = selected_opt {
                if let Ok(mut lvl) = crate::level::load_level_from_file(&selected) {
                    lvl.description = Some(comment_buf.clone());
                    let _ = crate::level::save_level_to_file(&selected, &lvl);
                    if let Some(entry) = editor.tester_entries.iter_mut().find(|e| e.path == selected) {
                        entry.description = comment_buf;
                        entry.has_comment = !entry.description.trim().is_empty();
                    }
                    editor.toast("Saved comment to level file.");
                }
            }
            editor.tester_comment_modal_open = false;
            editor.tester_dirty = true;
            return;
        }
        if keys.just_pressed(KeyCode::Backspace) {
            editor.tester_comment_buffer.pop();
            return;
        }
        for key in keys.get_just_pressed() {
            if let Some(c) = key_code_to_char(*key, shift_held) {
                if editor.tester_comment_buffer.len() < 256 {
                    editor.tester_comment_buffer.push(c);
                }
            }
        }
        return;
    }

    if editor.tester_promote_modal_open {
        if keys.just_pressed(KeyCode::Escape) {
            editor.tester_promote_modal_open = false;
            return;
        }
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            let selected_opt = editor.tester_selected_path.clone();
            let title = editor.tester_promote_title_buffer.clone();
            let filename = if editor.tester_promote_filename_buffer.ends_with(".json") {
                editor.tester_promote_filename_buffer.clone()
            } else {
                format!("{}.json", editor.tester_promote_filename_buffer)
            };
            let dest_dir = editor.tester_promote_dest_dir.clone();
            if let Some(selected) = selected_opt {
                if let Ok(mut lvl) = crate::level::load_level_from_file(&selected) {
                    lvl.name = title;
                    let dest_path = format!("{}/{}", dest_dir, filename);
                    let _ = crate::level::save_level_to_file(&dest_path, &lvl);
                    editor.toast(format!("Promoted & copied level to '{}'.", dest_path));
                }
            }
            editor.tester_promote_modal_open = false;
            editor.tester_dirty = true;
            return;
        }
        if keys.just_pressed(KeyCode::Backspace) {
            editor.tester_promote_title_buffer.pop();
            return;
        }
        for key in keys.get_just_pressed() {
            if let Some(c) = key_code_to_char(*key, shift_held) {
                if editor.tester_promote_title_buffer.len() < 64 {
                    editor.tester_promote_title_buffer.push(c);
                }
            }
        }
        return;
    }

    if editor.tester_delete_modal_open {
        if keys.just_pressed(KeyCode::Escape) {
            editor.tester_delete_modal_open = false;
            return;
        }
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            let bulk = editor.tester_bulk_selected.clone();
            if !bulk.is_empty() {
                let count = bulk.len();
                for path in &bulk {
                    let _ = std::fs::remove_file(path);
                }
                editor.tester_entries.retain(|e| !bulk.contains(&e.path));
                editor.tester_bulk_selected.clear();
                if let Some(selected) = &editor.tester_selected_path {
                    if bulk.contains(selected) {
                        editor.tester_selected_path =
                            editor.tester_entries.first().map(|e| e.path.clone());
                    }
                }
                editor.toast(format!("Permanently deleted {} level file(s).", count));
            } else if let Some(selected) = editor.tester_selected_path.clone() {
                let _ = std::fs::remove_file(&selected);
                editor.tester_entries.retain(|e| e.path != selected);
                editor.tester_selected_path =
                    editor.tester_entries.first().map(|e| e.path.clone());
                editor.toast(format!("Deleted '{}'.", selected));
            }
            editor.tester_delete_modal_open = false;
            editor.tester_dirty = true;
            if let Some(selected) = &editor.tester_selected_path {
                if let Ok(lvl) = crate::level::load_level_from_file(selected) {
                    game.engine = TurnEngine::new(lvl.to_world());
                }
            }
            return;
        }
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        if let Some(path) = &editor.tester_selected_path {
            if let Ok(lvl) = crate::level::load_level_from_file(path) {
                game.engine = TurnEngine::new(lvl.to_world());
                editor.current_level_path = path.clone();
                editor.solutions = lvl.solutions;
                editor.last_saved_hash = compute_level_hash(&game.engine.world);
            }
        }
        match game.engine.start_playtest() {
            Ok(()) => {
                editor.backup_world = Some(game.engine.frame_zero_star.clone());
                editor.playtest_win_recorded = false;
                editor.return_mode = AppMode::LevelTester;
                next_mode.set(AppMode::Playtest);
                editor.toast("Testing level (Press [Esc] to return to Level Tester).");
            }
            Err(err) => {
                editor.toast(format!("Cannot play: {}", err));
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyP) {
        if let Some(path) = &editor.tester_selected_path {
            if let Ok(lvl) = crate::level::load_level_from_file(path) {
                game.engine = TurnEngine::new(lvl.to_world());
                editor.current_level_path = path.clone();
                editor.solutions = lvl.solutions;
                editor.last_saved_hash = compute_level_hash(&game.engine.world);
            }
        }
        if let Some(sol) = editor.solutions.first() {
            let actions = sol.actions.clone();
            match game.engine.start_playtest() {
                Ok(()) => {
                    editor.backup_world = Some(game.engine.frame_zero_star.clone());
                    editor.playtest_win_recorded = false;
                    playback.is_playback = true;
                    playback.actions = actions;
                    playback.current_index = 0;
                    playback.auto_playing = true;
                    playback.speed = editor.playback_speed;
                    playback.step_timer = Timer::from_seconds(
                        0.40 / editor.playback_speed.max(0.1),
                        TimerMode::Repeating,
                    );
                    editor.return_mode = AppMode::LevelTester;
                    next_mode.set(AppMode::Playback);
                    editor.toast("Replaying solution (Press [Esc] to return to Level Tester).");
                }
                Err(err) => {
                    editor.toast(format!("Cannot play solution: {}", err));
                }
            }
        } else {
            editor.toast("No solution recorded for this level.");
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyE) && !keys.pressed(KeyCode::ControlLeft) && !keys.pressed(KeyCode::SuperLeft) {
        if let Some(path) = &editor.tester_selected_path {
            if let Ok(lvl) = crate::level::load_level_from_file(path) {
                game.engine = TurnEngine::new(lvl.to_world());
                editor.current_level_path = path.clone();
                editor.solutions = lvl.solutions;
                editor.last_saved_hash = compute_level_hash(&game.engine.world);
            }
        }
        editor.return_mode = AppMode::Editor;
        next_mode.set(AppMode::Editor);
        editor.toast("Opened level in Editor.");
        return;
    }

    if keys.just_pressed(KeyCode::KeyC) {
        editor.tester_comment_modal_open = true;
        let selected_opt = editor.tester_selected_path.clone();
        if let Some(selected) = selected_opt {
            if let Some(entry) = editor.tester_entries.iter().find(|e| e.path == selected) {
                editor.tester_comment_buffer = entry.description.clone();
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyR) {
        editor.tester_promote_modal_open = true;
        let selected_opt = editor.tester_selected_path.clone();
        if let Some(selected) = selected_opt {
            let entry_opt = editor.tester_entries.iter().find(|e| e.path == selected).cloned();
            if let Some(entry) = entry_opt {
                editor.tester_promote_title_buffer = entry.name;
                editor.tester_promote_filename_buffer = entry.filename;
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::Delete) {
        editor.tester_delete_modal_open = true;
        return;
    }

    // Row selection navigation with Up / Down arrow keys
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowDown) {
        let total = editor.tester_entries.len();
        if total > 0 {
            let current_idx = editor
                .tester_selected_path
                .as_ref()
                .and_then(|p| editor.tester_entries.iter().position(|e| &e.path == p))
                .unwrap_or(0);

            let new_idx = if keys.just_pressed(KeyCode::ArrowUp) {
                current_idx.saturating_sub(1)
            } else {
                (current_idx + 1).min(total - 1)
            };

            let path = editor.tester_entries[new_idx].path.clone();
            editor.tester_selected_path = Some(path.clone());
            if let Ok(lvl) = crate::level::load_level_from_file(&path) {
                game.engine = TurnEngine::new(lvl.to_world());
                editor.current_level_path = path;
                editor.solutions = lvl.solutions;
                editor.last_saved_hash = compute_level_hash(&game.engine.world);
            }
            if new_idx < editor.tester_scroll_offset {
                editor.tester_scroll_offset = new_idx;
            } else if new_idx >= editor.tester_scroll_offset + 14 {
                editor.tester_scroll_offset = new_idx - 13;
            }
            editor.tester_dirty = true;
        }
    }

    // Mouse wheel scrolling
    for event in mouse_wheel.read() {
        if event.y > 0.0 {
            editor.tester_scroll_offset = editor.tester_scroll_offset.saturating_sub(3);
            editor.tester_dirty = true;
        } else if event.y < 0.0 {
            editor.tester_scroll_offset = editor.tester_scroll_offset.saturating_add(3);
            editor.tester_dirty = true;
        }
    }
}
