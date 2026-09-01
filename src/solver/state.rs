//! Canonical state fingerprinting for the Macro Quotient Graph.
//!
//! Compresses all equivalent player walking permutations into a single canonical macro state.

use serde::{Deserialize, Serialize};

use crate::block_types::BlockKind;
use crate::laser;
use crate::sim::{CubeRot, World};
use crate::solver::reachability::ReachabilityMap;

/// Compact representation of a single body for hashing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CompactBodyState {
    pub kind: BlockKind,
    pub anchor: [i32; 3],
    pub orientation: CubeRot,
    pub combined_group: Option<u32>,
    pub is_fixed: bool,
}

/// Canonical fingerprint of a macro state in the quotient graph.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MacroState {
    /// Canonical list of all moveable / interactive bodies (sorted).
    pub bodies: Vec<CompactBodyState>,
    /// Canonical representative cell of the player's reachable zone.
    pub reachability_rep: [i32; 3],
    /// Mask or count of goals currently struck by lasers.
    pub goals_hit_mask: u64,
}

impl MacroState {
    /// Extract the canonical macro state from the current world and reachability map.
    pub fn from_world(world: &World, reachability: &ReachabilityMap) -> Self {
        let mut bodies = Vec::new();
        let player_id = world.player_id();

        for body in world.bodies() {
            // Ignore player body in the body list (player position is captured via reachability_rep)
            if Some(body.id) == player_id {
                continue;
            }

            // Exclude non-interactive static floors from canonical state if fixed
            if body.kind == BlockKind::Floor && body.is_fixed() {
                continue;
            }

            bodies.push(CompactBodyState {
                kind: body.kind,
                anchor: [body.anchor.x, body.anchor.y, body.anchor.z],
                orientation: body.orientation,
                combined_group: body.combined_group,
                is_fixed: body.is_fixed(),
            });
        }

        bodies.sort_unstable();

        let rep = reachability.canonical_representative();

        // Calculate goals struck by lasers
        let laser_segments = laser::cast_all_lasers(world);
        let mut goals_hit_mask = 0u64;
        let mut goal_index = 0;

        for body in world.bodies() {
            if body.kind == BlockKind::Goal {
                let is_hit = laser_segments
                    .iter()
                    .any(|seg| seg.hit.as_ref().map(|h| h.body_id == body.id).unwrap_or(false));
                if is_hit {
                    goals_hit_mask |= 1 << (goal_index % 64);
                }
                goal_index += 1;
            }
        }

        Self {
            bodies,
            reachability_rep: [rep.x, rep.y, rep.z],
            goals_hit_mask,
        }
    }
}
