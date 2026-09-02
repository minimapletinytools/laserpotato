//! Background worker polling for solver search and quality analysis, plus toast notification decaying.

use bevy::prelude::*;
use crate::editor::EditorState;
use crate::level::compute_level_hash;
use crate::GameState;

/// System to poll background solver and quality analyzer thread receivers.
pub fn background_solver_poll_system(
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
                if let Some(existing) = editor.solutions.iter_mut().find(|s| s.actions == result.actions) {
                    existing.name = name;
                } else {
                    editor.solutions.push(crate::level::LevelSolution::new(
                        name,
                        result.actions.clone(),
                    ));
                }
                editor.solution_picker_dirty = true;
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
            editor.puzzle_profile = Some(profile.clone());
            editor.quality_modal_open = true;
            editor.quality_modal_dirty = true;

            if profile.is_solvable {
                editor.solver_status = format!(
                    "✓ Analyzed: {} moves (Epiphany {:.1}, {:.0}% Load-Bearing)",
                    profile.macro_steps,
                    profile.epiphany_score,
                    profile.load_bearing_factor * 100.0
                );

                if !profile.optimal_actions.is_empty() {
                    editor.cached_solution = Some((current_hash, profile.optimal_actions.clone()));
                    let sol_name = format!(
                        "Optimal Solution ({} moves, {} turns, Epiphany {:.1})",
                        profile.macro_steps,
                        profile.atomic_turns,
                        profile.epiphany_score
                    );
                    if let Some(existing) = editor.solutions.iter_mut().find(|s| s.actions == profile.optimal_actions) {
                        existing.name = sol_name;
                        existing.profile = Some(profile.clone());
                    } else {
                        editor.solutions.push(crate::level::LevelSolution::with_profile(
                            sol_name,
                            profile.optimal_actions.clone(),
                            Some(profile.clone()),
                        ));
                    }
                    editor.solution_picker_dirty = true;
                }
            } else {
                editor.solver_status = "✗ Unsolvable (Quality Analyzed)".into();
            }

            editor.toast("Puzzle Quality Analysis complete!");
        } else {
            editor.solver_status = "Level modified during analysis. Invalidated.".into();
            editor.toast("Level modified during analysis. Invalidated.");
        }
    }
}

/// System to decay transient toast notifications over time.
pub fn toast_decay_system(time: Res<Time>, mut editor: ResMut<EditorState>) {
    if let Some((_, timer)) = &mut editor.status_message {
        timer.tick(time.delta());
        if timer.is_finished() {
            editor.status_message = None;
        }
    }
}
