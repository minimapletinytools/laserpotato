//! Turn resolution engine — processes player actions, movement cascades,
//! laser recalculation, and undo / reset.
//!
//! No Bevy dependency. The public entry point is
//! [`TurnEngine::apply`], which takes a [`PlayerAction`] and mutates the
//! contained [`World`](crate::sim::World).

use glam::IVec3;

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

/// Owns the simulation state, the undo stack, and the current laser state.
pub struct TurnEngine {
    /// Current simulation state — the source of truth.
    pub world: World,
    /// Laser segments computed after the most recent turn.
    pub laser_state: Vec<LaserSegment>,
    /// Stack of previous world snapshots for undo.
    undo_stack: Vec<World>,
    /// The world as it was when the puzzle started (for reset).
    initial_world: World,
}

impl TurnEngine {
    pub fn new(world: World) -> Self {
        let laser_state = laser::cast_all_lasers(&world);
        let initial_world = world.clone();
        Self {
            world,
            laser_state,
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
            // Future: inspect laser hits for state changes that create new
            // movement intentions, and loop back to resolve_movement.
            break;
        }

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
                // Turn left = CCW (+Z) in sim
                body.orientation = body.orientation.then(CubeRot::ROT_Z_90);
            }
            PlayerAction::TurnRight => {
                let body = self.world.body_mut(player_id).unwrap();
                // Turn right = CW (-Z) in sim
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
                if !occupant.kind.is_pushable() {
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
    use crate::sim::World;

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
        // Player starts facing +Y.
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
        // Facing +Y, turn left (CCW) → facing −X.
        engine.apply(PlayerAction::TurnLeft);
        assert_eq!(player_facing(&engine), IVec3::new(-1, 0, 0));
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO); // no movement
    }

    #[test]
    fn turn_right_rotates_cw() {
        let mut engine = TurnEngine::new(simple_level());
        // Facing +Y, turn right (CW) → facing +X.
        engine.apply(PlayerAction::TurnRight);
        assert_eq!(player_facing(&engine), IVec3::new(1, 0, 0));
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO); // no movement
    }

    #[test]
    fn four_left_turns_return_to_original_facing() {
        let mut engine = TurnEngine::new(simple_level());
        for _ in 0..4 {
            engine.apply(PlayerAction::TurnLeft);
        }
        assert_eq!(player_facing(&engine), IVec3::new(0, 1, 0));
    }

    #[test]
    fn turn_then_forward_moves_in_new_direction() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::TurnRight); // now facing +X (towards pushable)
        engine.apply(PlayerAction::Forward);   // step +X
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
    }

    #[test]
    fn push_block_forward() {
        let mut engine = TurnEngine::new(simple_level());
        // Turn right to face +X (toward the pushable block at (1,0,0)).
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward);
        // Player should be at (1,0), pushable at (2,0).
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
        let pushable = engine.world.body_at(IVec3::new(2, 0, 0));
        assert!(pushable.is_some());
        assert_eq!(pushable.unwrap().kind, BlockKind::Pushable);
    }

    #[test]
    fn blocked_forward_doesnt_move() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        world.spawn(BlockKind::Wall, IVec3::new(0, 1, 0), vec![IVec3::ZERO]);

        let mut engine = TurnEngine::new(world);
        engine.apply(PlayerAction::Forward);
        // Blocked by wall — player stays.
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn push_into_wall_doesnt_move() {
        let mut engine = TurnEngine::new(simple_level());
        // Face +X, push block twice toward wall at (3,0).
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward); // player→(1,0), block→(2,0)
        engine.apply(PlayerAction::Forward); // block would hit wall at (3,0) → blocked
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
    }

    #[test]
    fn undo_restores_previous_state() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::Forward);
        engine.apply(PlayerAction::Undo);
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn undo_restores_orientation() {
        let mut engine = TurnEngine::new(simple_level());
        engine.apply(PlayerAction::TurnRight);
        assert_eq!(player_facing(&engine), IVec3::new(1, 0, 0));
        engine.apply(PlayerAction::Undo);
        assert_eq!(player_facing(&engine), IVec3::new(0, 1, 0));
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

    #[test]
    fn chain_push_two_blocks() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Pushable, IVec3::new(0, 1, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Pushable, IVec3::new(0, 2, 0), vec![IVec3::ZERO]);

        let mut engine = TurnEngine::new(world);
        // Player faces +Y by default, blocks are ahead.
        engine.apply(PlayerAction::Forward);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
        assert!(engine.world.body_at(IVec3::new(0, 2, 0)).is_some());
        assert!(engine.world.body_at(IVec3::new(0, 3, 0)).is_some());
    }

    #[test]
    fn wait_is_a_valid_turn() {
        let mut engine = TurnEngine::new(simple_level());
        assert_eq!(engine.apply(PlayerAction::Wait), TurnResult::Ok);
        assert_eq!(engine.apply(PlayerAction::Undo), TurnResult::Undone);
    }
}
