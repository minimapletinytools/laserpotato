//! Procedural candidate generator: builds a candidate [`World`] from a seed and recipe.

use glam::IVec3;
use crate::block_types::BlockKind;
use crate::generator::prng::FastRng;
use crate::generator::recipe::BlockRecipe;
use crate::sim::{unit_shape, CubeRot, TagKind, TagValue, World};
use crate::turn::TurnEngine;

/// Configuration defining the room dimensions and recipe for candidate synthesis.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateSpec {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub recipe: BlockRecipe,
}

impl Default for CandidateSpec {
    fn default() -> Self {
        Self {
            width: 7,
            height: 7,
            depth: 1,
            recipe: BlockRecipe::default(),
        }
    }
}

/// Generate a candidate puzzle [`World`] from a deterministic seed.
/// Returns `Some(world)` if the level is structurally stable and non-trivial on Frame 0, or `None` if invalid.
pub fn generate_candidate_world(seed: u64, spec: &CandidateSpec) -> Option<World> {
    let mut rng = FastRng::seed(seed);
    let mut world = World::new();

    let w = spec.width as i32;
    let h = spec.height as i32;
    let d = spec.depth as i32;

    if w < 4 || h < 4 || d < 1 {
        return None;
    }

    // 1. Spawn boundary walls and floor tiles
    for x in 0..w {
        for y in 0..h {
            // Floor at z = -1
            if spec.recipe.is_allowed(BlockKind::Floor) {
                let fid = world.spawn(BlockKind::Floor, IVec3::new(x, y, -1), unit_shape());
                if let Some(b) = world.body_mut(fid) {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
            }

            // Outer boundary walls at z = 0..d
            if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
                for z in 0..d {
                    let wid = world.spawn(BlockKind::Wall, IVec3::new(x, y, z), unit_shape());
                    if let Some(b) = world.body_mut(wid) {
                        b.tags.set(TagKind::Fixed, TagValue::Unit);
                    }
                }
            }
        }
    }

    // 2. Interior cell pool (1..w-1, 1..h-1, 0..d)
    let mut interior_cells = Vec::new();
    for x in 1..(w - 1) {
        for y in 1..(h - 1) {
            for z in 0..d {
                interior_cells.push(IVec3::new(x, y, z));
            }
        }
    }
    rng.shuffle(&mut interior_cells);

    // 3. Spawn random interior walls according to wall_density
    let max_interior_walls = ((interior_cells.len() as f32) * spec.recipe.wall_density.clamp(0.0, 0.4)) as usize;
    let mut wall_count = 0;
    let mut available_cells = Vec::new();

    for cell in interior_cells {
        if wall_count < max_interior_walls && rng.gen_bool(spec.recipe.wall_density) {
            let wid = world.spawn(BlockKind::Wall, cell, unit_shape());
            if let Some(b) = world.body_mut(wid) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
            wall_count += 1;
        } else {
            available_cells.push(cell);
        }
    }

    if available_cells.len() < 4 {
        return None;
    }

    // 4. Spawn Player
    let player_pos = available_cells.pop()?;
    let pid = world.spawn(BlockKind::Player, player_pos, unit_shape());
    if let Some(b) = world.body_mut(pid) {
        let facings = [IVec3::X, -IVec3::X, IVec3::Y, -IVec3::Y];
        let f = *rng.choose(&facings).unwrap_or(&IVec3::Y);
        b.orientation = CubeRot::from_facing_2d(f);
    }

    // 5. Spawn Laser Source(s)
    let num_lasers = rng.gen_range(spec.recipe.laser_sources.0, spec.recipe.laser_sources.1 + 1);
    if spec.recipe.is_allowed(BlockKind::LaserSource) {
        for _ in 0..num_lasers {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            let lid = world.spawn(BlockKind::LaserSource, pos, unit_shape());
            if let Some(b) = world.body_mut(lid) {
                let facings = [IVec3::X, -IVec3::X, IVec3::Y, -IVec3::Y];
                let f = *rng.choose(&facings).unwrap_or(&IVec3::Y);
                b.orientation = CubeRot::from_facing_2d(f);
                if rng.gen_bool(spec.recipe.fixed_laser_chance) {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
            }
        }
    }

    // 6. Spawn Goal(s)
    let num_goals = rng.gen_range(spec.recipe.goals.0, spec.recipe.goals.1 + 1);
    if spec.recipe.is_allowed(BlockKind::Goal) {
        for _ in 0..num_goals {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            let gid = world.spawn(BlockKind::Goal, pos, unit_shape());
            if let Some(b) = world.body_mut(gid) {
                b.tags.set(TagKind::Fixed, TagValue::Unit);
            }
        }
    }

    // 7. Spawn Mirrors
    let num_mirrors = rng.gen_range(spec.recipe.mirrors.0, spec.recipe.mirrors.1 + 1);
    if spec.recipe.is_allowed(BlockKind::Mirror) {
        let orientations = [
            CubeRot::IDENTITY,
            CubeRot::IDENTITY.rot_world_z_cw(),
            CubeRot::IDENTITY.rot_world_z_cw().rot_world_z_cw(),
            CubeRot::IDENTITY.rot_world_z_ccw(),
        ];
        for _ in 0..num_mirrors {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            let mid = world.spawn(BlockKind::Mirror, pos, unit_shape());
            if let Some(b) = world.body_mut(mid) {
                b.orientation = *rng.choose(&orientations).unwrap_or(&CubeRot::IDENTITY);
                if rng.gen_bool(spec.recipe.fixed_mirror_chance) {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
            }
        }
    }

    // 8. Spawn Pushable Crates
    let num_crates = rng.gen_range(spec.recipe.crates.0, spec.recipe.crates.1 + 1);
    if spec.recipe.is_allowed(BlockKind::Pushable) {
        for _ in 0..num_crates {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            world.spawn(BlockKind::Pushable, pos, unit_shape());
        }
    }

    // 9. Spawn Glass blocks if enabled
    let num_glass = rng.gen_range(spec.recipe.glass.0, spec.recipe.glass.1 + 1);
    if spec.recipe.is_allowed(BlockKind::Glass) {
        for _ in 0..num_glass {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            world.spawn(BlockKind::Glass, pos, unit_shape());
        }
    }

    world.sync_grid();

    // 10. Frame 0 validation: ensure level is physically valid and not immediately solved/dead
    let engine = TurnEngine::new(world.clone());
    if engine.validation_error.is_some() {
        return None;
    }
    if engine.is_won() {
        return None; // Already won on Frame 0 (trivial)
    }
    if engine.is_lost() {
        return None; // Player immediately struck on Frame 0
    }

    Some(world)
}
