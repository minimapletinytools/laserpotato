//! Turn resolution engine — processes player actions, movement cascades,
//! laser recalculation, and undo / reset.
//!
//! No Bevy dependency. The public entry point is
//! [`TurnEngine::apply`], which takes a [`PlayerAction`] and mutates the
//! contained [`World`](crate::sim::World).

use glam::IVec3;

use crate::block_types::BlockKind;
use crate::laser::{self, LaserSegment};
use crate::sim::{BodyId, CubeRot, World};

/// Maximum number of movement↔state resolution passes before declaring
/// a paradox. (See design doc § fixpoint loop.)
const MAX_FIXPOINT_PASSES: usize = 16;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A player action for a single turn.
///
/// Movement uses tank controls:
/// - **Left / Right** turn the player 90° in place (no translation).
/// - **Forward** steps 1 cell in the current facing direction.
/// - **Backward** steps 1 cell opposite to the current facing direction.
///
/// Every action (including blocked moves and turns in place) counts as a
/// turn and is pushed onto the undo stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    /// Step 1 cell in the current facing direction.
    Forward,
    /// Step 1 cell opposite to the current facing direction.
    Backward,
    /// Rotate 90° counter-clockwise (looking down from +Z in sim).
    TurnLeft,
    /// Rotate 90° clockwise (looking down from +Z in sim).
    TurnRight,
    /// Interact with whatever is in front of the player.
    Interact,
    /// Wait in place — still counts as a turn (design doc §player character).
    Wait,
    /// Undo the last turn.
    Undo,
    /// Reset the puzzle to its initial state.
    Reset,
}

/// Result of attempting to apply a [`PlayerAction`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnResult {
    /// Turn executed successfully.
    Ok,
    /// Successfully undid the last turn.
    Undone,
    /// Puzzle was reset to initial state.
    WasReset,
    /// No player body exists in the world.
    NoPlayer,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Owns the simulation state, the undo stack, the current laser state,
/// and level completion status.
pub struct TurnEngine {
    /// Current simulation state — the source of truth.
    pub world: World,
    /// Laser segments computed after the most recent turn.
    pub laser_state: Vec<LaserSegment>,
    /// Whether the level goal condition has been satisfied (laser hits Goal block).
    pub is_won: bool,
    /// Stack of previous world snapshots for undo.
    undo_stack: Vec<World>,
    /// The world as it was when the puzzle started (for reset).
    initial_world: World,
}

impl TurnEngine {
    pub fn new(world: World) -> Self {
        let laser_state = laser::cast_all_lasers(&world);
        let is_won = check_is_won(&world, &laser_state);
        let initial_world = world.clone();
        Self {
            world,
            laser_state,
            is_won,
            undo_stack: Vec::new(),
            initial_world,
        }
    }

    /// Apply a player action to the world.
    pub fn apply(&mut self, action: PlayerAction) -> TurnResult {
        match action {
            PlayerAction::Undo => self.undo(),
            PlayerAction::Reset => self.reset(),
            _ => self.resolve_turn(action),
        }
    }

    // -- undo / reset ------------------------------------------------------

    fn undo(&mut self) -> TurnResult {
        if let Some(prev) = self.undo_stack.pop() {
            self.world = prev;
            self.laser_state = laser::cast_all_lasers(&self.world);
            self.is_won = check_is_won(&self.world, &self.laser_state);
            TurnResult::Undone
        } else {
            // Nothing to undo — still a valid action, just a no-op.
            TurnResult::Ok
        }
    }

    fn reset(&mut self) -> TurnResult {
        self.world = self.initial_world.clone();
        self.undo_stack.clear();
        self.laser_state = laser::cast_all_lasers(&self.world);
        self.is_won = check_is_won(&self.world, &self.laser_state);
        TurnResult::WasReset
    }

    // -- turn resolution ---------------------------------------------------

    fn resolve_turn(&mut self, action: PlayerAction) -> TurnResult {
        if self.world.player_id().is_none() {
            return TurnResult::NoPlayer;
        }

        // Snapshot for the undo stack.
        let snapshot = self.world.clone();

        // --- movement phase -----------------------------------------------
        self.resolve_movement(&action);

        // --- state / laser phase (with fixpoint loop) ---------------------
        for _pass in 0..MAX_FIXPOINT_PASSES {
            self.laser_state = laser::cast_all_lasers(&self.world);
            break;
        }

        self.is_won = check_is_won(&self.world, &self.laser_state);

        // Every input is a valid action and goes on the undo stack.
        self.undo_stack.push(snapshot);
        TurnResult::Ok
    }

    /// Execute movement for the current action.
    ///
    /// - **TurnLeft / TurnRight**: rotate the player 90° in place.
    /// - **Forward**: step 1 cell in the player's facing direction (push chain).
    /// - **Backward**: step 1 cell opposite the player's facing direction.
    /// - **Wait / Interact**: no movement.
    fn resolve_movement(&mut self, action: &PlayerAction) {
        let player_id = match self.world.player_id() {
            Some(id) => id,
            None => return,
        };

        match action {
            PlayerAction::TurnLeft => {
                let body = self.world.body_mut(player_id).unwrap();
                body.orientation = body.orientation.then(CubeRot::ROT_Z_90);
            }
            PlayerAction::TurnRight => {
                let body = self.world.body_mut(player_id).unwrap();
                body.orientation = body.orientation.then(CubeRot::ROT_Z_270);
            }
            PlayerAction::Forward => {
                let facing = self.player_facing(player_id);
                self.try_step(player_id, facing);
            }
            PlayerAction::Backward => {
                let facing = self.player_facing(player_id);
                self.try_step(player_id, -facing);
            }
            _ => {} // Wait / Interact — no movement
        }
    }

    /// The player's current world-space facing direction (+Y local → world).
    fn player_facing(&self, player_id: BodyId) -> IVec3 {
        let body = self.world.body(player_id).unwrap();
        body.orientation.apply(IVec3::new(0, 1, 0))
    }

    /// Attempt to move `mover_id` one cell in `direction`, pushing blocks
    /// as needed. If blocked, nothing happens (the turn still counts).
    fn try_step(&mut self, mover_id: BodyId, direction: IVec3) {
        if let Some(chain) = collect_push_chain(&self.world, mover_id, direction) {
            for &body_id in &chain {
                self.world.body_mut(body_id).unwrap().anchor += direction;
            }
            self.world.sync_grid();
        }
    }
}

/// Checks if any active laser beam segment hits a [`BlockKind::Goal`] block.
fn check_is_won(world: &World, laser_state: &[LaserSegment]) -> bool {
    for segment in laser_state {
        if let Some(hit) = &segment.hit {
            if let Some(body) = world.body(hit.body_id) {
                if body.kind == BlockKind::Goal {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Push-chain resolution
// ---------------------------------------------------------------------------

/// Starting from `mover_id` wanting to step in `direction`, collect every
/// body that must also move (transitively pushed). Returns `None` if the
/// chain is blocked by an immovable body.
fn collect_push_chain(world: &World, mover_id: BodyId, direction: IVec3) -> Option<Vec<BodyId>> {
    let mut chain = vec![mover_id];
    let mut i = 0;

    while i < chain.len() {
        let body_id = chain[i];
        let body = world.body(body_id).unwrap();

        for cell in body.world_cells() {
            let target = cell + direction;
            if let Some(occupant_id) = world.grid().occupant_at(target) {
                if chain.contains(&occupant_id) {
                    continue;
                }
                let occupant = world.body(occupant_id).unwrap();
                if !occupant.is_pushable() {
                    return None;
                }
                chain.push(occupant_id);
            }
        }
        i += 1;
    }

    Some(chain)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;
    use crate::sim::{TagKind, TagValue, World};

    /// Player(facing +Y) — Pushable — ··· — Wall
    ///       (0,0)           (1,0)          (3,0)
    fn simple_level() -> World {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Pushable, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Wall, IVec3::new(3, 0, 0), vec![IVec3::ZERO]);
        world
    }

    fn player_body(engine: &TurnEngine) -> &crate::sim::Body {
        engine
            .world
            .body(engine.world.player_id().unwrap())
            .unwrap()
    }

    /// The player's facing direction as a world-space IVec3.
    fn player_facing(engine: &TurnEngine) -> IVec3 {
        let body = player_body(engine);
        body.orientation.apply(IVec3::new(0, 1, 0))
    }

    #[test]
    fn forward_moves_in_facing_direction() {
        let mut engine = TurnEngine::new(simple_level());
        assert_eq!(player_facing(&engine), IVec3::new(0, 1, 0));
        engine.apply(PlayerAction::Forward);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
    }

    #[test]
    fn backward_moves_opposite_facing() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::Backward);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, -1, 0));
    }

    #[test]
    fn turn_left_rotates_ccw() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::TurnLeft);
        assert_eq!(player_facing(&engine), IVec3::new(-1, 0, 0));
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn turn_right_rotates_cw() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::TurnRight);
        assert_eq!(player_facing(&engine), IVec3::new(1, 0, 0));
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn push_moveable_laser_source() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        world.spawn(BlockKind::LaserSource, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);

        let mut engine = TurnEngine::new(world);
        engine.apply(PlayerAction::TurnRight); // face +X
        engine.apply(PlayerAction::Forward);   // push laser

        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
        let laser = engine.world.body_at(IVec3::new(2, 0, 0));
        assert!(laser.is_some());
        assert_eq!(laser.unwrap().kind, BlockKind::LaserSource);
    }

    #[test]
    fn fixed_tag_prevents_pushing() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let mirror_id = world.spawn(BlockKind::Mirror, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        // Tag mirror as Fixed
        world.body_mut(mirror_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

        let mut engine = TurnEngine::new(world);
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward);

        // Blocked because mirror is fixed!
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn laser_hitting_goal_sets_is_won() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        // Laser firing +Y from (2, 0)
        world.spawn(BlockKind::LaserSource, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        // Goal at (2, 4)
        world.spawn(BlockKind::Goal, IVec3::new(2, 4, 0), vec![IVec3::ZERO]);

        let engine = TurnEngine::new(world);
        assert!(engine.is_won, "Level should be won when laser strikes the Goal pyramid!");
    }

    #[test]
    fn push_block_forward() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward);
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
        let pushable = engine.world.body_at(IVec3::new(2, 0, 0));
        assert!(pushable.is_some());
        assert_eq!(pushable.unwrap().kind, BlockKind::Pushable);
    }

    #[test]
    fn undo_restores_previous_state() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::Forward);
        engine.apply(PlayerAction::Undo);
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn reset_restores_initial_state() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::Forward);
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Reset);
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
        assert_eq!(player_facing(&engine), IVec3::new(0, 1, 0));
    }
}
