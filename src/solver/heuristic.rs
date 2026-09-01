//! Composable, weighted heuristic framework for puzzle state evaluation.
//!
//! Provides extensible trait-based heuristic evaluators for laser routing,
//! goal activation, spatial proximity, and upcoming puzzle mechanics.

use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::block_types::BlockKind;
use crate::laser;
use crate::sim::World;
use crate::solver::reachability::ReachabilityMap;

/// Built-in heuristic selector for solver configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HeuristicKind {
    /// Zero heuristic ($h = 0$, pure uniform-cost / BFS).
    None,
    /// Distance from laser beam endpoints to Goal pyramids.
    GoalLaserTarget,
    /// Weighted composite of all domain evaluators.
    #[default]
    Composite,
}

/// Trait defining an admissible or informative puzzle heuristic.
pub trait PuzzleHeuristic: Send + Sync {
    /// Evaluate estimated remaining effort (0 = goal reached).
    fn estimate(&self, world: &World, reachability: &ReachabilityMap) -> u32;

    /// Human-readable name for logging and performance profiling.
    fn name(&self) -> &'static str;
}

/// Heuristic evaluating how close laser beam paths are to striking all Goal pyramids.
#[derive(Clone, Copy, Debug, Default)]
pub struct GoalLaserTargetHeuristic;

impl PuzzleHeuristic for GoalLaserTargetHeuristic {
    fn estimate(&self, world: &World, _reachability: &ReachabilityMap) -> u32 {
        let goals: Vec<IVec3> = world
            .bodies()
            .iter()
            .filter(|b| b.kind == BlockKind::Goal)
            .map(|b| b.anchor)
            .collect();

        if goals.is_empty() {
            return 0;
        }

        let laser_segments = laser::cast_all_lasers(world);
        let mut total_h = 0u32;

        for &goal_pos in &goals {
            let mut is_hit = false;
            let mut min_dist_to_goal = u32::MAX;

            for seg in &laser_segments {
                if let Some(hit) = &seg.hit {
                    if let Some(target_body) = world.body(hit.body_id) {
                        if target_body.kind == BlockKind::Goal && target_body.anchor == goal_pos {
                            is_hit = true;
                            break;
                        }
                    }
                }

                let end_point = seg
                    .hit
                    .as_ref()
                    .map(|h| h.cell)
                    .unwrap_or_else(|| seg.cells.last().copied().unwrap_or(seg.origin));

                let seg_dist = (end_point.x - goal_pos.x).abs()
                    + (end_point.y - goal_pos.y).abs()
                    + (end_point.z - goal_pos.z).abs();
                min_dist_to_goal = min_dist_to_goal.min(seg_dist as u32);
            }

            if !is_hit {
                // Base penalty for unhit goal + distance from closest laser tip
                total_h += 10 + min_dist_to_goal.min(50);
            }
        }

        total_h
    }

    fn name(&self) -> &'static str {
        "GoalLaserTarget"
    }
}

/// Heuristic measuring player distance to the nearest moveable interaction.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerProximityHeuristic;

impl PuzzleHeuristic for PlayerProximityHeuristic {
    fn estimate(&self, world: &World, _reachability: &ReachabilityMap) -> u32 {
        let Some(player_id) = world.player_id() else {
            return 0;
        };
        let Some(player) = world.body(player_id) else {
            return 0;
        };

        let min_dist = world
            .bodies()
            .iter()
            .filter(|b| b.is_pushable() && b.id != player_id)
            .map(|b| {
                (b.anchor.x - player.anchor.x).abs()
                    + (b.anchor.y - player.anchor.y).abs()
                    + (b.anchor.z - player.anchor.z).abs()
            })
            .min()
            .unwrap_or(0);

        // Small tie-breaker cost (scaled down)
        (min_dist as u32).min(10)
    }

    fn name(&self) -> &'static str {
        "PlayerProximity"
    }
}

/// Composite heuristic aggregating multiple weighted sub-evaluators.
pub struct CompositeHeuristic {
    evaluators: Vec<(Box<dyn PuzzleHeuristic>, f32)>,
}

impl Default for CompositeHeuristic {
    fn default() -> Self {
        Self {
            evaluators: vec![
                (Box::new(GoalLaserTargetHeuristic), 1.0),
                (Box::new(PlayerProximityHeuristic), 0.2),
            ],
        }
    }
}

impl CompositeHeuristic {
    pub fn new() -> Self {
        Self {
            evaluators: Vec::new(),
        }
    }

    /// Add a domain evaluator with a relative weight.
    pub fn add_evaluator<H: PuzzleHeuristic + 'static>(&mut self, heuristic: H, weight: f32) {
        self.evaluators.push((Box::new(heuristic), weight));
    }
}

impl PuzzleHeuristic for CompositeHeuristic {
    fn estimate(&self, world: &World, reachability: &ReachabilityMap) -> u32 {
        let mut score = 0.0f32;
        for (evaluator, weight) in &self.evaluators {
            score += evaluator.estimate(world, reachability) as f32 * weight;
        }
        score.round() as u32
    }

    fn name(&self) -> &'static str {
        "Composite"
    }
}

/// Evaluate a heuristic by kind on the current world and reachability map.
pub fn evaluate_heuristic(
    kind: HeuristicKind,
    world: &World,
    reachability: &ReachabilityMap,
) -> u32 {
    match kind {
        HeuristicKind::None => 0,
        HeuristicKind::GoalLaserTarget => GoalLaserTargetHeuristic.estimate(world, reachability),
        HeuristicKind::Composite => CompositeHeuristic::default().estimate(world, reachability),
    }
}
