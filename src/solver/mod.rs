//! Automated puzzle solver and quality analysis engine for *Laser Potato*.
//!
//! Provides Macro-Move Quotient Graph search, reachability flood-fill,
//! composable heuristics, and puzzle quality profiling (Epiphany Score,
//! Bottleneck detection, Load-Bearing redundancy checking).

pub mod analysis;
pub mod heuristic;
pub mod macro_move;
pub mod reachability;
pub mod result;
pub mod search;
pub mod state;

pub use analysis::{analyze_puzzle, PuzzleProfile};
pub use heuristic::{
    CompositeHeuristic, GoalLaserTargetHeuristic, HeuristicKind, PlayerProximityHeuristic,
    PuzzleHeuristic,
};
pub use macro_move::{generate_macro_moves, MacroArchetype, MacroMove};
pub use reachability::ReachabilityMap;
pub use result::{load_actions_from_file, SolveResult, SolveStatus};
pub use search::{search, Algorithm, SolverConfig};
pub use state::{CompactBodyState, MacroState};

use crate::sim::World;

/// Solve the puzzle using default solver configuration (A* with Composite Heuristic).
pub fn solve(world: World) -> SolveResult {
    search(world, &SolverConfig::default())
}

/// Solve the puzzle using a specific [`SolverConfig`].
pub fn solve_with_config(world: World, config: &SolverConfig) -> SolveResult {
    search(world, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec3;
    use crate::block_types::BlockKind;

    #[test]
    fn solver_solves_trivial_single_push_puzzle() {
        let mut world = World::new();
        // Player at (3, 1) facing +X
        let player_id = world.spawn(BlockKind::Player, IVec3::new(3, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(player_id).unwrap().orientation = crate::sim::CubeRot::ROT_Z_270;

        // Fixed laser at (5, 0) firing +Y
        let laser_id = world.spawn(BlockKind::LaserSource, IVec3::new(5, 0, 0), vec![IVec3::ZERO]);
        world.body_mut(laser_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        // Moveable mirror at (4, 1) — identity orientation reflects +Y -> +X
        world.spawn(BlockKind::Mirror, IVec3::new(4, 1, 0), vec![IVec3::ZERO]);

        // Goal pyramid at (7, 1)
        let goal_id = world.spawn(BlockKind::Goal, IVec3::new(7, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(goal_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        let result = solve(world.clone());
        assert!(result.is_solved());
        assert_eq!(result.macro_count(), 1);
        assert!(crate::turn::validate_solution(&world, &result.actions));
    }

    #[test]
    fn solver_bfs_solves_and_finds_shortest_macro_path() {
        let mut world = World::new();
        let player_id = world.spawn(BlockKind::Player, IVec3::new(3, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(player_id).unwrap().orientation = crate::sim::CubeRot::ROT_Z_270;

        let laser_id = world.spawn(BlockKind::LaserSource, IVec3::new(5, 0, 0), vec![IVec3::ZERO]);
        world.body_mut(laser_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        world.spawn(BlockKind::Mirror, IVec3::new(4, 1, 0), vec![IVec3::ZERO]);
        let goal_id = world.spawn(BlockKind::Goal, IVec3::new(7, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(goal_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        let config = SolverConfig {
            algorithm: Algorithm::Bfs,
            ..Default::default()
        };
        let result = solve_with_config(world, &config);
        assert!(result.is_solved());
        assert_eq!(result.macro_count(), 1);
    }

    #[test]
    fn solver_detects_unsolvable_puzzle() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        // Fixed Laser firing into fixed wall
        let laser_id = world.spawn(BlockKind::LaserSource, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        world.body_mut(laser_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        let w1 = world.spawn(BlockKind::Wall, IVec3::new(2, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(w1).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        // Goal unreachable
        let goal_id = world.spawn(BlockKind::Goal, IVec3::new(5, 5, 0), vec![IVec3::ZERO]);
        world.body_mut(goal_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        // Bounded area
        for x in -2..=6 {
            let w = world.spawn(BlockKind::Wall, IVec3::new(x, -2, 0), vec![IVec3::ZERO]);
            world.body_mut(w).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);
            let w = world.spawn(BlockKind::Wall, IVec3::new(x, 6, 0), vec![IVec3::ZERO]);
            world.body_mut(w).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);
        }

        let config = SolverConfig {
            max_depth: Some(5),
            ..Default::default()
        };
        let result = solve_with_config(world, &config);
        assert!(!result.is_solved());
    }

    #[test]
    fn solver_puzzle_quality_profiler_test() {
        let mut world = World::new();
        // Player at (3, 1) facing +X
        let player_id = world.spawn(BlockKind::Player, IVec3::new(3, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(player_id).unwrap().orientation = crate::sim::CubeRot::ROT_Z_270;

        // Fixed laser at (5, 0) firing +Y
        let laser_id = world.spawn(BlockKind::LaserSource, IVec3::new(5, 0, 0), vec![IVec3::ZERO]);
        world.body_mut(laser_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        // Moveable mirror at (4, 1)
        let m1 = world.spawn(BlockKind::Mirror, IVec3::new(4, 1, 0), vec![IVec3::ZERO]);

        // Redundant moveable crate at (0, 0)
        let c1 = world.spawn(BlockKind::Pushable, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);

        // Goal pyramid at (7, 1)
        let goal_id = world.spawn(BlockKind::Goal, IVec3::new(7, 1, 0), vec![IVec3::ZERO]);
        world.body_mut(goal_id).unwrap().tags.set(crate::sim::TagKind::Fixed, crate::sim::TagValue::Unit);

        let profile = analyze_puzzle(&world);
        assert!(profile.is_solvable);
        assert_eq!(profile.macro_steps, 1);
        // Crate c1 should be detected as redundant
        assert!(profile.redundant_bodies.contains(&c1));
        // Mirror m1 should NOT be redundant (it is load-bearing)
        assert!(!profile.redundant_bodies.contains(&m1));
    }

    #[test]
    fn solve_levels_directory_test() {
        for file in crate::level::list_level_files() {
            println!("==================================================");
            println!("Testing level file: {}", file);
            if let Ok(data) = crate::level::load_level_from_file(&file) {
                let world = data.to_world();
                println!("  Bodies: {}", world.bodies().len());
                let player_id = world.player_id();
                println!("  Player ID: {:?}, anchor={:?}", player_id, player_id.and_then(|id| world.body(id)).map(|b| b.anchor));
                
                // Print all bodies around player
                if let Some(pid) = player_id {
                    let player = world.body(pid).unwrap();
                    println!("  Player pos: {:?}", player.anchor);
                    for &dir in &crate::solver::reachability::CARDINAL_DIRS {
                        let adj = player.anchor + dir;
                        let occ = world.body_at(adj);
                        println!("    Adj {:?} (at {:?}): {:?}", dir, adj, occ.map(|b| (b.id, b.kind, b.is_fixed())));
                    }
                }

                let reachability = ReachabilityMap::compute(&world);
                println!("  Reachability: {:?}", reachability.as_ref().map(|r| (r.start_pos, r.reachable_cells.len())));
                if let Some(rm) = &reachability {
                    println!("  Reachable cells: {:?}", rm.reachable_cells.keys().collect::<Vec<_>>());
                    println!("  Hazard cells: {:?}", rm.hazard_cells.iter().collect::<Vec<_>>());
                    let moves = generate_macro_moves(&world, rm);
                    println!("  Generated Macro Moves: {}", moves.len());
                    for (i, m) in moves.iter().enumerate() {
                        println!("    {}. body={:?} dir={:?} stand={:?} actions={:?}", i + 1, m.target_body, m.direction, m.player_stand_pos, m.walk_actions);
                    }
                }
                let res = solve(world.clone());
                println!("  Solve result: status={:?} macro_moves={} turns={} duration={:?}", res.status, res.macro_moves.len(), res.actions.len(), res.duration);
                assert!(res.is_solved(), "Level {} should be solvable", file);
                assert!(crate::turn::validate_solution(&world, &res.actions), "Solution for {} must be valid in turn engine", file);
            }
        }
    }
}
