//! Macro-Move representations, generator, and extensible archetype registry.

use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::block_types::{BlockKind, PlayerMovementMode};
use crate::sim::{BodyId, World};
use crate::solver::reachability::{ReachabilityMap, CARDINAL_DIRS};
use crate::turn::{collect_push_chain, PlayerAction};

/// Extensible classification of macro-move archetypes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroArchetype {
    /// Standard physical translation of a block from `from` to `to`.
    SpatialPush {
        body_id: BodyId,
        kind: BlockKind,
        from: IVec3,
        to: IVec3,
    },
    /// Spatial Exchange / Nook Parking: Storing an object temporarily to swap ordering.
    SpatialExchange {
        swapped_bodies: [BodyId; 2],
        parking_spot: IVec3,
    },
    /// Optical Topology Switch: Rerouting laser beams by altering mirror / laser positions.
    OpticalSwitch {
        source_id: BodyId,
        affected_blocks: Vec<BodyId>,
    },
    /// Irreversible Phase Shift: One-way state transition (dropping block, gate lock).
    PhaseShift {
        description: String,
    },
    /// Extension Slot: Sensor / Pressure Plate Trigger (Future mechanic)
    SensorTrigger {
        sensor_id: BodyId,
        activated: bool,
    },
    /// Extension Slot: Portal / Teleport Hop (Future mechanic)
    PortalHop {
        entry_portal: BodyId,
        exit_portal: BodyId,
    },
    /// Extension Slot: Laser Filter / Polarizer Match (Future mechanic)
    FilterMatch {
        filter_id: BodyId,
        wavelength_match: bool,
    },
    /// Custom / Extension hook for game mechanics
    Custom(String),
}

/// A high-level macro move representing an intentional world-altering operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroMove {
    /// ID of the primary body being manipulated.
    pub target_body: BodyId,
    /// Push direction in world space.
    pub direction: IVec3,
    /// Position where the player must stand before executing the push.
    pub player_stand_pos: IVec3,
    /// Facing direction required to initiate the push.
    pub player_push_facing: IVec3,
    /// Macro archetype category.
    pub archetype: MacroArchetype,
    /// Micro-step walk actions from player's starting position to player_stand_pos.
    pub walk_actions: Vec<PlayerAction>,
    /// The primary push action.
    pub push_action: PlayerAction,
}

impl MacroMove {
    /// Returns the full atomic sequence of [`PlayerAction`]s (walking + push).
    pub fn all_actions(&self) -> Vec<PlayerAction> {
        let mut actions = self.walk_actions.clone();
        actions.push(self.push_action);
        actions
    }
}

/// Generate all legal, productive macro moves available from the current world state.
pub fn generate_macro_moves(world: &World, reachability: &ReachabilityMap) -> Vec<MacroMove> {
    let Some(player_id) = world.player_id() else {
        return Vec::new();
    };
    let movement_mode = world
        .body(player_id)
        .map(|b| b.properties().player_movement_mode)
        .unwrap_or_default();

    let mut macro_moves = Vec::new();

    // Iterate through all pushable bodies in the world
    for body in world.bodies() {
        if !body.is_pushable() || body.id == player_id {
            continue;
        }

        let body_id = body.id;
        let body_kind = body.kind;
        let body_anchor = body.anchor;

        // Try pushing in all 4 cardinal directions
        for &dir in &CARDINAL_DIRS {
            let required_stand_pos = body_anchor - dir;

            // Player must be able to reach the required stand position
            if !reachability.is_reachable(required_stand_pos) {
                continue;
            }

            // Verify that pushing in `dir` is legally allowed by the physics chain
            let mut sim_world = world.clone();
            if let Some(p) = sim_world.body_mut(player_id) {
                p.anchor = required_stand_pos;
            }
            sim_world.sync_grid();
            if collect_push_chain(&sim_world, player_id, dir).is_none() {
                continue;
            }

            // Pathfind walking actions to required_stand_pos
            let walk_actions = if required_stand_pos == reachability.start_pos {
                Vec::new()
            } else {
                match reachability.find_walk_path(
                    required_stand_pos,
                    None,
                    movement_mode,
                ) {
                    Some(acts) => acts,
                    None => continue,
                }
            };

            let push_action = match movement_mode {
                PlayerMovementMode::Tank => PlayerAction::Forward,
                PlayerMovementMode::Strafe
                | PlayerMovementMode::TurnAndMove
                | PlayerMovementMode::TurnAndMoveBackstep => match (dir.x, dir.y) {
                    (0, 1) => PlayerAction::MoveNorth,
                    (0, -1) => PlayerAction::MoveSouth,
                    (1, 0) => PlayerAction::MoveEast,
                    (-1, 0) => PlayerAction::MoveWest,
                    _ => PlayerAction::Forward,
                },
            };

            // Classify archetype
            let archetype = if body_kind == BlockKind::Mirror || body_kind == BlockKind::LaserSource {
                MacroArchetype::OpticalSwitch {
                    source_id: body_id,
                    affected_blocks: vec![body_id],
                }
            } else {
                MacroArchetype::SpatialPush {
                    body_id,
                    kind: body_kind,
                    from: body_anchor,
                    to: body_anchor + dir,
                }
            };

            macro_moves.push(MacroMove {
                target_body: body_id,
                direction: dir,
                player_stand_pos: required_stand_pos,
                player_push_facing: dir,
                archetype,
                walk_actions,
                push_action,
            });
        }
    }

    macro_moves
}
