//! Automated puzzle solver engine for *Laser Potato*.
//!
//! Provides graph search (BFS, DFS, A*, Greedy Best-First), state
//! canonicalization, cycle/loop detection, and heuristic evaluation.

pub mod heuristic;
pub mod result;
pub mod search;
pub mod state;

pub use heuristic::HeuristicKind;
pub use result::{load_actions_from_file, SolveResult, SolveStatus};
pub use search::{search, Algorithm, SolverConfig};
pub use state::{CanonicalState, CompactBodyState};

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
    use crate::turn::PlayerAction;

    #[test]
    fn solver_solves_trivial_single_step_puzzle() {
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

        let result = solve(world);
        assert!(result.is_solved());
        assert_eq!(result.actions.len(), 1);
        assert_eq!(result.actions[0], PlayerAction::Forward);
    }

    #[test]
    fn solver_bfs_solves_and_finds_shortest_path() {
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
        assert_eq!(result.actions.len(), 1);
        assert_eq!(result.actions[0], PlayerAction::Forward);
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
            max_depth: Some(10),
            ..Default::default()
        };
        let result = solve_with_config(world, &config);
        assert!(!result.is_solved());
    }
}
