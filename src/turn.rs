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

impl PlayerAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlayerAction::Forward => "Forward",
            PlayerAction::Backward => "Backward",
            PlayerAction::TurnLeft => "TurnLeft",
            PlayerAction::TurnRight => "TurnRight",
            PlayerAction::Interact => "Interact",
            PlayerAction::Wait => "Wait",
            PlayerAction::Undo => "Undo",
            PlayerAction::Reset => "Reset",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "forward" => Some(PlayerAction::Forward),
            "backward" => Some(PlayerAction::Backward),
            "turnleft" | "turn_left" | "left" => Some(PlayerAction::TurnLeft),
            "turnright" | "turn_right" | "right" => Some(PlayerAction::TurnRight),
            "interact" => Some(PlayerAction::Interact),
            "wait" => Some(PlayerAction::Wait),
            "undo" => Some(PlayerAction::Undo),
            "reset" => Some(PlayerAction::Reset),
            _ => None,
        }
    }
}

/// Overall outcome / state of the game session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GameOutcome {
    #[default]
    InProgress,
    /// Player won (laser struck Goal pyramid).
    Won,
    /// Player lost (laser struck the Player character).
    Lost,
}

impl GameOutcome {
    pub fn is_game_over(self) -> bool {
        self != Self::InProgress
    }
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
    /// Action was rejected because the game is already in a Win/Loss state (Undo & Reset are still allowed).
    GameOver,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Owns the simulation state, the undo stack, the current laser state,
/// and game outcome (Win / Lose / InProgress).
pub struct TurnEngine {
    /// Current simulation state — the source of truth.
    pub world: World,
    /// Laser segments computed after the most recent turn.
    pub laser_state: Vec<LaserSegment>,
    /// Current game outcome.
    pub outcome: GameOutcome,
    /// Stack of previous world snapshots for undo.
    undo_stack: Vec<World>,
    /// The world as it was when the puzzle started (for reset).
    initial_world: World,
}

impl TurnEngine {
    pub fn new(world: World) -> Self {
        let laser_state = laser::cast_all_lasers(&world);
        let outcome = evaluate_outcome(&world, &laser_state);
        let initial_world = world.clone();
        Self {
            world,
            laser_state,
            outcome,
            undo_stack: Vec::new(),
            initial_world,
        }
    }

    /// Convenience helper for checking if the player has won.
    pub fn is_won(&self) -> bool {
        self.outcome == GameOutcome::Won
    }

    /// Convenience helper for checking if the player has lost.
    pub fn is_lost(&self) -> bool {
        self.outcome == GameOutcome::Lost
    }

    /// Apply a player action to the world.
    pub fn apply(&mut self, action: PlayerAction) -> TurnResult {
        match action {
            PlayerAction::Undo => self.undo(),
            PlayerAction::Reset => self.reset(),
            _ => {
                // When in a Win or Loss state, no further gameplay moves are accepted.
                if self.outcome.is_game_over() {
                    return TurnResult::GameOver;
                }
                self.resolve_turn(action)
            }
        }
    }

    // -- undo / reset ------------------------------------------------------

    fn undo(&mut self) -> TurnResult {
        if let Some(prev) = self.undo_stack.pop() {
            self.world = prev;
            self.laser_state = laser::cast_all_lasers(&self.world);
            self.outcome = evaluate_outcome(&self.world, &self.laser_state);
            TurnResult::Undone
        } else {
            TurnResult::Ok
        }
    }

    fn reset(&mut self) -> TurnResult {
        self.world = self.initial_world.clone();
        self.undo_stack.clear();
        self.laser_state = laser::cast_all_lasers(&self.world);
        self.outcome = evaluate_outcome(&self.world, &self.laser_state);
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

        self.outcome = evaluate_outcome(&self.world, &self.laser_state);

        // Every input is a valid action and goes on the undo stack.
        self.undo_stack.push(snapshot);
        TurnResult::Ok
    }

    /// Execute movement for the current action.
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

/// Evaluates if the current state is a Win, Loss, or InProgress.
/// Striking the Player takes precedence as a Loss.
fn evaluate_outcome(world: &World, laser_state: &[LaserSegment]) -> GameOutcome {
    let mut hit_player = false;
    let mut hit_goal = false;

    for segment in laser_state {
        if let Some(hit) = &segment.hit {
            if let Some(body) = world.body(hit.body_id) {
                match body.kind {
                    BlockKind::Player => hit_player = true,
                    BlockKind::Goal => hit_goal = true,
                    _ => {}
                }
            }
        }
    }

    if hit_player {
        GameOutcome::Lost
    } else if hit_goal {
        GameOutcome::Won
    } else {
        GameOutcome::InProgress
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
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward);

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
        world.body_mut(mirror_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

        let mut engine = TurnEngine::new(world);
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward);

        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
    }

    #[test]
    fn laser_hitting_goal_sets_win() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::LaserSource, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Goal, IVec3::new(2, 4, 0), vec![IVec3::ZERO]);

        let mut engine = TurnEngine::new(world);
        assert_eq!(engine.outcome, GameOutcome::Won);
        assert!(engine.is_won());

        // In Win state, gameplay inputs are blocked
        assert_eq!(engine.apply(PlayerAction::Forward), TurnResult::GameOver);
        // But Undo is allowed
        assert_eq!(engine.apply(PlayerAction::Undo), TurnResult::Ok);
    }

    #[test]
    fn laser_hitting_player_sets_loss() {
        let mut world = World::new();
        // Laser at (2, 0) firing +Y
        world.spawn(BlockKind::LaserSource, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        // Player steps into laser at (2, 3)
        world.spawn(BlockKind::Player, IVec3::new(1, 3, 0), vec![IVec3::ZERO]);

        let mut engine = TurnEngine::new(world);
        assert_eq!(engine.outcome, GameOutcome::InProgress);

        // Turn right and step into (2, 3) where the laser is active!
        engine.apply(PlayerAction::TurnRight);
        engine.apply(PlayerAction::Forward);

        assert_eq!(engine.outcome, GameOutcome::Lost);
        assert!(engine.is_lost());

        // In Loss state, gameplay inputs are blocked
        assert_eq!(engine.apply(PlayerAction::Forward), TurnResult::GameOver);
        assert_eq!(engine.apply(PlayerAction::TurnLeft), TurnResult::GameOver);

        // Undo successfully takes the player out of the laser!
        assert_eq!(engine.apply(PlayerAction::Undo), TurnResult::Undone);
        assert_eq!(engine.outcome, GameOutcome::InProgress);
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
    fn test_level_initial_state_and_solve() {
        let mut engine = TurnEngine::new(crate::level::test_level());
        assert_eq!(engine.outcome, GameOutcome::InProgress);
        assert!(!engine.is_won());

        // Manually place mirrors in winning relay positions:
        // Move MM2 from (3, 6) to (5, 6)
        let mm2_id = engine.world.body_at(IVec3::new(3, 6, 0)).unwrap().id;
        engine.world.body_mut(mm2_id).unwrap().anchor = IVec3::new(5, 6, 0);

        // Move MM1 from (2, 5) to (1, 4)
        let mm1_id = engine.world.body_at(IVec3::new(2, 5, 0)).unwrap().id;
        engine.world.body_mut(mm1_id).unwrap().anchor = IVec3::new(1, 4, 0);

        // Push LaserSource from (1, 0) to (1, 1)
        let laser_id = engine.world.body_at(IVec3::new(1, 0, 0)).unwrap().id;
        engine.world.body_mut(laser_id).unwrap().anchor = IVec3::new(1, 1, 0);
        engine.world.sync_grid();

        // Take a turn (Wait) to trigger recalculation
        engine.apply(PlayerAction::Wait);

        // The 3-bounce laser path connects: (1,1) -> (1,4) -> (5,4) -> (5,6) -> (7,6) Goal!
        assert_eq!(engine.outcome, GameOutcome::Won);
        assert!(engine.is_won());
    }
}
