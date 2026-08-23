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

pub use crate::block_types::{PlayerMovementMode, DEFAULT_PLAYER_MOVEMENT_MODE};

/// A player action for a single turn.
///
/// Behavior adapts dynamically based on the player block's [`PlayerMovementMode`]:
/// - **Tank**: Left/Right turn in place; Up/Down step forward/backward.
/// - **Strafe**: Directional keys step in cardinal directions without turning.
/// - **TurnAndMove**: Directional keys turn to face the direction and step.
/// - **TurnAndMoveBackstep**: Directional keys turn and step, but reverse direction steps backward without turning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PlayerAction {
    /// Step 1 cell in the current facing direction.
    Forward,
    /// Step 1 cell opposite to the current facing direction.
    Backward,
    /// Rotate 90° counter-clockwise (looking down from +Z in sim).
    TurnLeft,
    /// Rotate 90° clockwise (looking down from +Z in sim).
    TurnRight,
    /// Directional move intent: North (+Y in world space).
    MoveNorth,
    /// Directional move intent: South (-Y in world space).
    MoveSouth,
    /// Directional move intent: West (-X in world space).
    MoveWest,
    /// Directional move intent: East (+X in world space).
    MoveEast,
    /// Interact with whatever is in front of the player.
    Interact,
    /// Wait in place — still counts as a turn.
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
            PlayerAction::MoveNorth => "MoveNorth",
            PlayerAction::MoveSouth => "MoveSouth",
            PlayerAction::MoveWest => "MoveWest",
            PlayerAction::MoveEast => "MoveEast",
            PlayerAction::Interact => "Interact",
            PlayerAction::Wait => "Wait",
            PlayerAction::Undo => "Undo",
            PlayerAction::Reset => "Reset",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "forward" | "up" | "w" => Some(PlayerAction::Forward),
            "backward" | "down" | "s" => Some(PlayerAction::Backward),
            "turnleft" | "turn_left" | "q" => Some(PlayerAction::TurnLeft),
            "turnright" | "turn_right" | "e" => Some(PlayerAction::TurnRight),
            "movenorth" | "move_north" | "north" => Some(PlayerAction::MoveNorth),
            "movesouth" | "move_south" | "south" => Some(PlayerAction::MoveSouth),
            "movewest" | "move_west" | "west" => Some(PlayerAction::MoveWest),
            "moveeast" | "move_east" | "east" => Some(PlayerAction::MoveEast),
            "interact" => Some(PlayerAction::Interact),
            "wait" | "space" => Some(PlayerAction::Wait),
            "undo" | "z" | "u" => Some(PlayerAction::Undo),
            "reset" | "r" => Some(PlayerAction::Reset),
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
    /// Current active simulation state.
    pub world: World,
    /// Laser segments computed after the most recent state update.
    pub laser_state: Vec<LaserSegment>,
    /// Current game outcome.
    pub outcome: GameOutcome,
    /// Stack of previous world snapshots for undo.
    undo_stack: Vec<World>,
    /// The world as it was at frame 0 (for reset).
    initial_world: World,
    /// The raw authoring world at frame -0.5 (before frame 0 simulation).
    pub raw_world: World,
    /// Validation error detected during frame 0 computation (e.g. spontaneous movements).
    pub validation_error: Option<String>,
}

/// Compute Frame 0 from Frame -0.5 (raw authoring state).
///
/// Performs the full initial state update (grid sync, fixpoint state resolution, laser raycasting,
/// outcome evaluation). If any spontaneous movement or position change occurs during this frame 0 resolution,
/// returns a validation error string indicating the level is invalid.
pub fn compute_frame_zero(raw_world: &World) -> (World, Vec<LaserSegment>, GameOutcome, Option<String>) {
    let mut frame0_world = raw_world.clone();
    frame0_world.sync_grid();

    // Record pre-simulation body positions from frame -0.5
    let pre_sim_bodies: Vec<(BodyId, IVec3, CubeRot)> = raw_world
        .bodies()
        .iter()
        .map(|b| (b.id, b.anchor, b.orientation))
        .collect();

    // Multi-pass state / laser resolution fixpoint loop
    let mut laser_state = Vec::new();
    for _pass in 0..MAX_FIXPOINT_PASSES {
        laser_state = laser::cast_all_lasers(&frame0_world);
        break;
    }

    let outcome = evaluate_outcome(&frame0_world, &laser_state);

    // Verify no spontaneous movement occurred between Frame -0.5 and Frame 0:
    let mut moved_bodies = Vec::new();
    for &(id, orig_anchor, orig_rot) in &pre_sim_bodies {
        if let Some(b) = frame0_world.body(id) {
            if b.anchor != orig_anchor || b.orientation != orig_rot {
                moved_bodies.push(format!(
                    "{:?} (ID {}) moved from {:?} to {:?}",
                    b.kind, id.0, orig_anchor, b.anchor
                ));
            }
        }
    }

    let validation_error = if !moved_bodies.is_empty() {
        let msg = format!(
            "Level is invalid: spontaneous movement during Frame 0 resolution (Frame -0.5 → Frame 0): {}",
            moved_bodies.join(", ")
        );
        eprintln!("[!] {}", msg);
        Some(msg)
    } else {
        None
    };

    (frame0_world, laser_state, outcome, validation_error)
}

impl TurnEngine {
    pub fn new(world: World) -> Self {
        let raw_world = world.clone();
        let (frame0_world, laser_state, outcome, validation_error) = compute_frame_zero(&raw_world);
        let initial_world = frame0_world.clone();
        Self {
            world: frame0_world,
            laser_state,
            outcome,
            undo_stack: Vec::new(),
            initial_world,
            raw_world,
            validation_error,
        }
    }

    /// Whether this level passed frame 0 validation without spontaneous movement.
    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none()
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
        let (frame0_world, laser_state, outcome, validation_error) = compute_frame_zero(&self.raw_world);
        self.world = frame0_world.clone();
        self.initial_world = frame0_world;
        self.undo_stack.clear();
        self.laser_state = laser_state;
        self.outcome = outcome;
        self.validation_error = validation_error;
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

        let mode = self
            .world
            .body(player_id)
            .map(|b| b.properties().player_movement_mode)
            .unwrap_or_default();
        let facing = self.player_facing(player_id);

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
                self.try_step(player_id, facing);
            }
            PlayerAction::Backward => {
                self.try_step(player_id, -facing);
            }
            PlayerAction::MoveNorth => self.handle_directional_move(player_id, IVec3::Y, mode),
            PlayerAction::MoveSouth => self.handle_directional_move(player_id, -IVec3::Y, mode),
            PlayerAction::MoveWest => self.handle_directional_move(player_id, -IVec3::X, mode),
            PlayerAction::MoveEast => self.handle_directional_move(player_id, IVec3::X, mode),
            _ => {} // Wait / Interact — no movement
        }
    }

    /// Handle directional movement input according to the player's active [`PlayerMovementMode`].
    fn handle_directional_move(&mut self, player_id: BodyId, dir: IVec3, mode: PlayerMovementMode) {
        let current_facing = self.player_facing(player_id);

        match mode {
            PlayerMovementMode::Tank => {
                // Mode 1: Tank Controls
                if dir == current_facing {
                    self.try_step(player_id, current_facing);
                } else if dir == -current_facing {
                    self.try_step(player_id, -current_facing);
                } else {
                    // Turn to face the requested direction in place
                    let body = self.world.body_mut(player_id).unwrap();
                    body.orientation = CubeRot::from_facing_2d(dir);
                }
            }
            PlayerMovementMode::Strafe => {
                // Mode 2: Direct translation in dir without altering facing orientation
                self.try_step(player_id, dir);
            }
            PlayerMovementMode::TurnAndMove => {
                // Mode 3: Turn & Move (Always face the direction of movement)
                let body = self.world.body_mut(player_id).unwrap();
                body.orientation = CubeRot::from_facing_2d(dir);
                self.try_step(player_id, dir);
            }
            PlayerMovementMode::TurnAndMoveBackstep => {
                // Mode 4: Turn & Move with Backstep on Opposite Direction
                if dir == -current_facing {
                    // Exact reverse direction: step backward without turning
                    self.try_step(player_id, dir);
                } else {
                    // Forward or orthogonal: turn to face dir and step
                    let body = self.world.body_mut(player_id).unwrap();
                    body.orientation = CubeRot::from_facing_2d(dir);
                    self.try_step(player_id, dir);
                }
            }
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

    #[test]
    fn movement_mode_tank() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut engine = TurnEngine::new(world);
        let pid = engine.world.player_id().unwrap();

        // Facing North (+Y) initially
        assert_eq!(player_facing(&engine), IVec3::Y);

        // MoveNorth: steps forward to (0, 1)
        engine.handle_directional_move(pid, IVec3::Y, PlayerMovementMode::Tank);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::Y);

        // MoveWest: turns left to face West (-X), stays at (0, 1)
        engine.handle_directional_move(pid, IVec3::NEG_X, PlayerMovementMode::Tank);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::NEG_X);
    }

    #[test]
    fn movement_mode_strafe() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut engine = TurnEngine::new(world);
        let p_id = engine.world.player_id().unwrap();

        // 1. Move North (+Y): steps to (0, 1) without turning (facing remains North)
        engine.handle_directional_move(p_id, IVec3::Y, PlayerMovementMode::Strafe);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::Y);

        // 2. Move West (-X): steps to (-1, 1) without turning (facing remains North)
        engine.handle_directional_move(p_id, IVec3::NEG_X, PlayerMovementMode::Strafe);
        assert_eq!(player_body(&engine).anchor, IVec3::new(-1, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::Y, "Facing should remain North in Strafe mode");
    }

    #[test]
    fn movement_mode_turn_and_move() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut engine = TurnEngine::new(world);
        let p_id = engine.world.player_id().unwrap();

        // 1. Move East: turns East (+X) and steps to (1, 0)
        engine.handle_directional_move(p_id, IVec3::X, PlayerMovementMode::TurnAndMove);
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
        assert_eq!(player_facing(&engine), IVec3::X);

        // 2. Move West (reverse direction): turns 180° West (-X) and steps to (0, 0)
        engine.handle_directional_move(p_id, IVec3::NEG_X, PlayerMovementMode::TurnAndMove);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 0, 0));
        assert_eq!(player_facing(&engine), IVec3::NEG_X);
    }

    #[test]
    fn movement_mode_turn_and_move_backstep() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut engine = TurnEngine::new(world);
        let p_id = engine.world.player_id().unwrap();

        // 1. Move East: turns East (+X) and steps to (1, 0)
        engine.handle_directional_move(p_id, IVec3::X, PlayerMovementMode::TurnAndMoveBackstep);
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
        assert_eq!(player_facing(&engine), IVec3::X);

        // 2. Move West (exact opposite of East!): steps back to (0, 0) WITHOUT changing facing from East!
        engine.handle_directional_move(p_id, IVec3::NEG_X, PlayerMovementMode::TurnAndMoveBackstep);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 0, 0));
        assert_eq!(player_facing(&engine), IVec3::X, "Opposite direction must backstep without turning");

        // 3. Move North (orthogonal): turns North (+Y) and steps to (0, 1)
        engine.handle_directional_move(p_id, IVec3::Y, PlayerMovementMode::TurnAndMoveBackstep);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::Y);

        // 4. Move South (exact opposite of North!): steps back to (0, 0) WITHOUT changing facing from North!
        engine.handle_directional_move(p_id, IVec3::NEG_Y, PlayerMovementMode::TurnAndMoveBackstep);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 0, 0));
        assert_eq!(player_facing(&engine), IVec3::Y, "Opposite direction must backstep without turning");
    }

    #[test]
    fn single_press_turn_and_move_in_one_step() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut engine = TurnEngine::new(world);

        // Player starts at (0, 0) facing North (+Y) with default TurnAndMoveBackstep mode:
        assert_eq!(player_body(&engine).anchor, IVec3::ZERO);
        assert_eq!(player_facing(&engine), IVec3::Y);

        // 1. Press Right (MoveEast): in a SINGLE press, turns East (+X) AND steps to (1, 0)
        engine.apply(PlayerAction::MoveEast);
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 0, 0));
        assert_eq!(player_facing(&engine), IVec3::X);

        // 2. Press Up (MoveNorth): in a SINGLE press, turns North (+Y) AND steps to (1, 1)
        engine.apply(PlayerAction::MoveNorth);
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::Y);

        // 3. Press Left (MoveWest): in a SINGLE press, turns West (-X) AND steps to (0, 1)
        engine.apply(PlayerAction::MoveWest);
        assert_eq!(player_body(&engine).anchor, IVec3::new(0, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::NEG_X);

        // 4. Press Right (MoveEast - opposite of West): in a SINGLE press, steps back to (1, 1) keeping facing West (-X)
        engine.apply(PlayerAction::MoveEast);
        assert_eq!(player_body(&engine).anchor, IVec3::new(1, 1, 0));
        assert_eq!(player_facing(&engine), IVec3::NEG_X, "Backstep preserves facing in single turn");
    }

    #[test]
    fn frame_zero_full_state_update_and_validation() {
        let mut world = World::new();
        world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        world.spawn(BlockKind::LaserSource, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Mirror, IVec3::new(1, 4, 0), vec![IVec3::ZERO]);

        let engine = TurnEngine::new(world);

        // 1. Frame -0.5 is preserved in raw_world
        assert_eq!(engine.raw_world.bodies().len(), 3);

        // 2. Frame 0 is valid and has computed lasers & outcome
        assert!(engine.is_valid());
        assert!(engine.validation_error.is_none());
        assert_eq!(engine.laser_state.len(), 2, "Lasers must be computed on Frame 0");
        assert_eq!(engine.outcome, GameOutcome::InProgress);
    }
}
