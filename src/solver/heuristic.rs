//! Heuristic evaluation functions for state scoring and informed search (A*, Greedy Best-First).

use glam::IVec3;

use crate::block_types::BlockKind;
use crate::laser::LaserSegment;
use crate::sim::World;

/// Type of heuristic evaluation function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HeuristicKind {
    /// Always returns 0 (uninformed search, Dijkstra / BFS equivalence in A*).
    Zero,
    /// Minimum Manhattan distance from any active laser beam termination point to the Goal.
    LaserToGoal,
    /// Weighted combination of laser proximity to goal and player proximity to interactive blocks.
    #[default]
    Composite,
}

/// Manhattan distance (L1 norm) between two 3D integer coordinates.
#[inline]
pub fn manhattan_distance(a: IVec3, b: IVec3) -> u32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()) as u32
}

/// Evaluates the heuristic estimate $h(s)$ for the given world and laser state.
/// Lower values indicate states that are closer to completing the level.
pub fn evaluate(world: &World, laser_segments: &[LaserSegment], kind: HeuristicKind) -> u32 {
    if kind == HeuristicKind::Zero {
        return 0;
    }

    // 1. Locate Goal block(s)
    let goal_positions: Vec<IVec3> = world
        .bodies()
        .iter()
        .filter(|b| b.kind == BlockKind::Goal)
        .map(|b| b.anchor)
        .collect();

    if goal_positions.is_empty() {
        return 0;
    }

    // 2. Check if laser is already striking the goal
    for segment in laser_segments {
        if let Some(hit) = &segment.hit {
            if let Some(body) = world.body(hit.body_id) {
                if body.kind == BlockKind::Goal {
                    return 0;
                }
            }
        }
    }

    // 3. Laser-to-Goal distance: find minimum distance from any ray tip to any goal
    let mut min_laser_dist = u32::MAX;
    for segment in laser_segments {
        let endpoint = if let Some(hit) = &segment.hit {
            hit.cell
        } else if let Some(&last_cell) = segment.cells.last() {
            last_cell
        } else {
            segment.origin
        };

        for &goal_pos in &goal_positions {
            let d = manhattan_distance(endpoint, goal_pos);
            if d < min_laser_dist {
                min_laser_dist = d;
            }
        }
    }

    if min_laser_dist == u32::MAX {
        min_laser_dist = 50; // Fallback if no lasers active
    }

    if kind == HeuristicKind::LaserToGoal {
        return min_laser_dist * 10;
    }

    // 4. Composite: include player distance to nearest moveable / interactive block
    let mut player_to_block_dist = 0;
    if let Some(player_id) = world.player_id() {
        if let Some(player) = world.body(player_id) {
            let mut min_p_dist = u32::MAX;
            for body in world.bodies() {
                if body.id != player_id && body.is_pushable() {
                    let d = manhattan_distance(player.anchor, body.anchor);
                    if d < min_p_dist {
                        min_p_dist = d;
                    }
                }
            }
            if min_p_dist != u32::MAX {
                player_to_block_dist = min_p_dist;
            }
        }
    }

    min_laser_dist * 10 + player_to_block_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;
    use crate::laser;
    use crate::sim::World;

    #[test]
    fn goal_reached_evaluates_to_zero() {
        let mut world = World::new();
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Goal, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);

        let lasers = laser::cast_all_lasers(&world);
        assert_eq!(evaluate(&world, &lasers, HeuristicKind::LaserToGoal), 0);
        assert_eq!(evaluate(&world, &lasers, HeuristicKind::Composite), 0);
    }

    #[test]
    fn closer_laser_has_lower_heuristic_score() {
        let mut w1 = World::new();
        w1.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        w1.spawn(BlockKind::Wall, IVec3::new(0, 2, 0), vec![IVec3::ZERO]);
        w1.spawn(BlockKind::Goal, IVec3::new(5, 5, 0), vec![IVec3::ZERO]);

        let mut w2 = World::new();
        w2.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        w2.spawn(BlockKind::Wall, IVec3::new(0, 4, 0), vec![IVec3::ZERO]);
        w2.spawn(BlockKind::Goal, IVec3::new(5, 5, 0), vec![IVec3::ZERO]);

        let l1 = laser::cast_all_lasers(&w1);
        let l2 = laser::cast_all_lasers(&w2);

        let h1 = evaluate(&w1, &l1, HeuristicKind::LaserToGoal);
        let h2 = evaluate(&w2, &l2, HeuristicKind::LaserToGoal);

        // Ray in w2 travels further towards (5,5) than ray in w1
        assert!(h2 < h1);
    }
}
