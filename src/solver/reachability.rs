//! 3D grid flood-fill reachability and micro-walk pathfinding for the player.
//!
//! Computes all grid cells the player can navigate to without pushing or displacing blocks,
//! avoiding cells with lethal active laser hazards.

use std::collections::{HashMap, HashSet, VecDeque};
use glam::IVec3;

use crate::block_types::PlayerMovementMode;
use crate::laser;
use crate::sim::{CubeRot, World};
use crate::turn::PlayerAction;

/// Directional unit vectors in the XY plane.
pub const CARDINAL_DIRS: [IVec3; 4] = [
    IVec3::new(0, 1, 0),  // North (+Y)
    IVec3::new(0, -1, 0), // South (-Y)
    IVec3::new(1, 0, 0),  // East (+X)
    IVec3::new(-1, 0, 0), // West (-X)
];

/// A reachability map representing all tiles the player can safely access.
#[derive(Clone, Debug)]
pub struct ReachabilityMap {
    /// Start position of the player when reachability was computed.
    pub start_pos: IVec3,
    /// Start facing orientation of the player.
    pub start_facing: IVec3,
    /// Map of reachable cell -> minimum step count from start_pos.
    pub reachable_cells: HashMap<IVec3, u32>,
    /// Set of cells that currently contain lethal active laser beams.
    pub hazard_cells: HashSet<IVec3>,
}

impl ReachabilityMap {
    /// Compute all reachable tiles for the player in the current world state.
    pub fn compute(world: &World) -> Option<Self> {
        let player_id = world.player_id()?;
        let player = world.body(player_id)?;
        let start_pos = player.anchor;
        let start_facing = player.orientation.apply(IVec3::Y);

        // Compute search bounding box from world bodies + safety margin
        let mut min_x = start_pos.x;
        let mut max_x = start_pos.x;
        let mut min_y = start_pos.y;
        let mut max_y = start_pos.y;
        let mut min_z = start_pos.z;
        let mut max_z = start_pos.z;

        for body in world.bodies() {
            for cell in body.world_cells() {
                min_x = min_x.min(cell.x);
                max_x = max_x.max(cell.x);
                min_y = min_y.min(cell.y);
                max_y = max_y.max(cell.y);
                min_z = min_z.min(cell.z);
                max_z = max_z.max(cell.z);
            }
        }

        let bounds_min = IVec3::new(min_x - 2, min_y - 2, min_z);
        let bounds_max = IVec3::new(max_x + 2, max_y + 2, max_z);

        // Identify hazardous laser beam cells (where stepping would burn the player)
        let laser_segments = laser::cast_all_lasers(world);
        let mut hazard_cells = HashSet::new();
        for seg in &laser_segments {
            for &cell in &seg.cells {
                hazard_cells.insert(cell);
            }
            if let Some(hit) = &seg.hit {
                hazard_cells.insert(hit.cell);
            }
        }
        hazard_cells.remove(&start_pos);

        let mut reachable_cells = HashMap::new();
        let mut queue = VecDeque::new();

        reachable_cells.insert(start_pos, 0);
        queue.push_back(start_pos);

        while let Some(current) = queue.pop_front() {
            let current_dist = reachable_cells[&current];

            for &dir in &CARDINAL_DIRS {
                let next = current + dir;

                // Bounds check
                if next.x < bounds_min.x
                    || next.x > bounds_max.x
                    || next.y < bounds_min.y
                    || next.y > bounds_max.y
                    || next.z < bounds_min.z
                    || next.z > bounds_max.z
                {
                    continue;
                }

                // Check if next cell is already visited
                if reachable_cells.contains_key(&next) {
                    continue;
                }

                // Check if next cell is occupied by any body (other than player)
                if let Some(occ) = world.body_at(next) {
                    if occ.id != player_id {
                        continue;
                    }
                }

                // Check if next cell is hazardous (active laser beam)
                if hazard_cells.contains(&next) {
                    continue;
                }

                // Cell is safe and vacant -> player can walk into it
                reachable_cells.insert(next, current_dist + 1);
                queue.push_back(next);
            }
        }

        Some(Self {
            start_pos,
            start_facing,
            reachable_cells,
            hazard_cells,
        })
    }

    /// Check if a given cell is reachable.
    pub fn is_reachable(&self, cell: IVec3) -> bool {
        self.reachable_cells.contains_key(&cell)
    }

    /// Lexicographically smallest cell in the reachable set (canonical partition representative).
    pub fn canonical_representative(&self) -> IVec3 {
        *self
            .reachable_cells
            .keys()
            .min_by(|a, b| a.z.cmp(&b.z).then(a.y.cmp(&b.y)).then(a.x.cmp(&b.x)))
            .unwrap_or(&self.start_pos)
    }

    /// Find the shortest atomic [`PlayerAction`] path from start_pos to target_pos,
    /// ending with the player facing `desired_facing`.
    pub fn find_walk_path(
        &self,
        target_pos: IVec3,
        desired_facing: Option<IVec3>,
        movement_mode: PlayerMovementMode,
    ) -> Option<Vec<PlayerAction>> {
        if !self.is_reachable(target_pos) {
            return None;
        }

        // BFS over (position, facing) state space
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        struct WalkState {
            pos: IVec3,
            facing: IVec3,
        }

        let start_state = WalkState {
            pos: self.start_pos,
            facing: self.start_facing,
        };

        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<WalkState, (Option<WalkState>, PlayerAction)> = HashMap::new();

        queue.push_back(start_state);
        parent_map.insert(start_state, (None, PlayerAction::Wait));

        let mut end_state: Option<WalkState> = None;

        while let Some(curr) = queue.pop_front() {
            if curr.pos == target_pos {
                if let Some(req_facing) = desired_facing {
                    if curr.facing == req_facing {
                        end_state = Some(curr);
                        break;
                    }
                } else {
                    end_state = Some(curr);
                    break;
                }
            }

            // Candidate walking / turning actions
            let candidate_actions = match movement_mode {
                PlayerMovementMode::Tank => vec![
                    (PlayerAction::Forward, curr.pos + curr.facing, curr.facing),
                    (PlayerAction::Backward, curr.pos - curr.facing, curr.facing),
                    (
                        PlayerAction::TurnLeft,
                        curr.pos,
                        CubeRot::ROT_Z_90.apply(curr.facing),
                    ),
                    (
                        PlayerAction::TurnRight,
                        curr.pos,
                        CubeRot::ROT_Z_270.apply(curr.facing),
                    ),
                ],
                PlayerMovementMode::Strafe => {
                    let mut acts = Vec::new();
                    for &dir in &CARDINAL_DIRS {
                        let action = match (dir.x, dir.y) {
                            (0, 1) => PlayerAction::MoveNorth,
                            (0, -1) => PlayerAction::MoveSouth,
                            (1, 0) => PlayerAction::MoveEast,
                            (-1, 0) => PlayerAction::MoveWest,
                            _ => continue,
                        };
                        acts.push((action, curr.pos + dir, curr.facing));
                    }
                    acts
                }
                PlayerMovementMode::TurnAndMove | PlayerMovementMode::TurnAndMoveBackstep => {
                    let mut acts = Vec::new();
                    for &dir in &CARDINAL_DIRS {
                        let action = match (dir.x, dir.y) {
                            (0, 1) => PlayerAction::MoveNorth,
                            (0, -1) => PlayerAction::MoveSouth,
                            (1, 0) => PlayerAction::MoveEast,
                            (-1, 0) => PlayerAction::MoveWest,
                            _ => continue,
                        };
                        let next_facing = if movement_mode == PlayerMovementMode::TurnAndMoveBackstep && dir == -curr.facing {
                            curr.facing
                        } else {
                            dir
                        };
                        acts.push((action, curr.pos + dir, next_facing));
                    }
                    acts
                }
            };

            for (action, next_pos, next_facing) in candidate_actions {
                // Pos must be reachable
                if !self.is_reachable(next_pos) {
                    continue;
                }

                let next_state = WalkState {
                    pos: next_pos,
                    facing: next_facing,
                };

                if !parent_map.contains_key(&next_state) {
                    parent_map.insert(next_state, (Some(curr), action));
                    queue.push_back(next_state);
                }
            }
        }

        let found_end = end_state?;
        let mut path = Vec::new();
        let mut curr = found_end;

        while let Some((prev_opt, act)) = parent_map.get(&curr) {
            if let Some(prev) = prev_opt {
                path.push(*act);
                curr = *prev;
            } else {
                break;
            }
        }

        path.reverse();
        Some(path)
    }
}
