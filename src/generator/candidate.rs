//! Procedural candidate generator: builds structured, multi-tier, combined-block candidate [`World`]s.

use std::collections::HashSet;
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
            width: 8,
            height: 8,
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

    if w < 5 || h < 5 || d < 1 {
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

    let mut occupied_cells: HashSet<IVec3> = HashSet::new();

    // 2. Structured Wall Layouts (Architectural Templates)
    if spec.recipe.is_allowed(BlockKind::Wall) {
        if spec.recipe.structured_walls {
            let template_type = rng.gen_range(0, 5);
            match template_type {
                0 => {
                    // Central Partition with 1-2 Chokepoint Apertures
                    let is_vertical = rng.gen_bool(0.5);
                    let split = if is_vertical { w / 2 } else { h / 2 };
                    let gap_pos1 = rng.gen_range(2, if is_vertical { h - 2 } else { w - 2 } as u32) as i32;
                    let gap_pos2 = (gap_pos1 + 2).min(if is_vertical { h - 2 } else { w - 2 });

                    let span = if is_vertical { 1..(h - 1) } else { 1..(w - 1) };
                    for pos in span {
                        if pos == gap_pos1 || pos == gap_pos2 {
                            continue; // Chokepoint doorway / laser window
                        }
                        let wall_cell = if is_vertical {
                            IVec3::new(split, pos, 0)
                        } else {
                            IVec3::new(pos, split, 0)
                        };
                        let wid = world.spawn(BlockKind::Wall, wall_cell, unit_shape());
                        if let Some(b) = world.body_mut(wid) {
                            b.tags.set(TagKind::Fixed, TagValue::Unit);
                        }
                        occupied_cells.insert(wall_cell);
                    }
                }
                1 => {
                    // 4-Pillar Symmetry Layout
                    let px1 = w / 3;
                    let px2 = w - 1 - (w / 3);
                    let py1 = h / 3;
                    let py2 = h - 1 - (h / 3);
                    for &px in &[px1, px2] {
                        for &py in &[py1, py2] {
                            let wall_cell = IVec3::new(px, py, 0);
                            let wid = world.spawn(BlockKind::Wall, wall_cell, unit_shape());
                            if let Some(b) = world.body_mut(wid) {
                                b.tags.set(TagKind::Fixed, TagValue::Unit);
                            }
                            occupied_cells.insert(wall_cell);
                        }
                    }
                }
                2 => {
                    // L-Shaped Corner Alcove
                    let alcove_corner_x = if rng.gen_bool(0.5) { 2 } else { w - 3 };
                    let alcove_corner_y = if rng.gen_bool(0.5) { 2 } else { h - 3 };
                    for dx in 0..2 {
                        let wall_cell = IVec3::new(alcove_corner_x + dx, alcove_corner_y, 0);
                        let wid = world.spawn(BlockKind::Wall, wall_cell, unit_shape());
                        if let Some(b) = world.body_mut(wid) {
                            b.tags.set(TagKind::Fixed, TagValue::Unit);
                        }
                        occupied_cells.insert(wall_cell);
                    }
                    for dy in 1..2 {
                        let wall_cell = IVec3::new(alcove_corner_x, alcove_corner_y + dy, 0);
                        let wid = world.spawn(BlockKind::Wall, wall_cell, unit_shape());
                        if let Some(b) = world.body_mut(wid) {
                            b.tags.set(TagKind::Fixed, TagValue::Unit);
                        }
                        occupied_cells.insert(wall_cell);
                    }
                }
                3 => {
                    // Corridor Baffle
                    let baffle_y = h / 2;
                    let baffle_start = if rng.gen_bool(0.5) { 1 } else { 3 };
                    let baffle_len = (w / 2).max(3);
                    for x in baffle_start..(baffle_start + baffle_len).min(w - 1) {
                        let wall_cell = IVec3::new(x, baffle_y, 0);
                        let wid = world.spawn(BlockKind::Wall, wall_cell, unit_shape());
                        if let Some(b) = world.body_mut(wid) {
                            b.tags.set(TagKind::Fixed, TagValue::Unit);
                        }
                        occupied_cells.insert(wall_cell);
                    }
                }
                _ => {
                    // Freeform sparse pillars
                    let num_pillars = rng.gen_range(2, 5);
                    for _ in 0..num_pillars {
                        let px = rng.gen_range(2, (w - 2) as u32) as i32;
                        let py = rng.gen_range(2, (h - 2) as u32) as i32;
                        let wall_cell = IVec3::new(px, py, 0);
                        if !occupied_cells.contains(&wall_cell) {
                            let wid = world.spawn(BlockKind::Wall, wall_cell, unit_shape());
                            if let Some(b) = world.body_mut(wid) {
                                b.tags.set(TagKind::Fixed, TagValue::Unit);
                            }
                            occupied_cells.insert(wall_cell);
                        }
                    }
                }
            }
        }
    }

    // 3. Pool of interior available ground cells
    let mut available_cells = Vec::new();
    for x in 1..(w - 1) {
        for y in 1..(h - 1) {
            let cell = IVec3::new(x, y, 0);
            if !occupied_cells.contains(&cell) {
                available_cells.push(cell);
            }
        }
    }
    rng.shuffle(&mut available_cells);

    if available_cells.len() < 6 {
        return None;
    }

    let orientations_4 = [
        CubeRot::IDENTITY,
        CubeRot::IDENTITY.rot_world_z_cw(),
        CubeRot::IDENTITY.rot_world_z_cw().rot_world_z_cw(),
        CubeRot::IDENTITY.rot_world_z_ccw(),
    ];
    let facings_4 = [IVec3::X, -IVec3::X, IVec3::Y, -IVec3::Y];

    // 4. Combined Blocks / Joined Polyominos (Domino pairs and Trominos of diverse block kinds)
    let num_combined = rng.gen_range(spec.recipe.combined_blocks.0, spec.recipe.combined_blocks.1 + 1);
    if num_combined > 0 && spec.recipe.is_allowed(BlockKind::Pushable) {
        let mut combined_spawned = 0;
        let mut i = 0;
        while i < available_cells.len() && combined_spawned < num_combined {
            let cell1 = available_cells[i];
            let neighbors = [cell1 + IVec3::X, cell1 - IVec3::X, cell1 + IVec3::Y, cell1 - IVec3::Y];

            if let Some(&cell2) = neighbors.iter().find(|n| {
                n.x > 0 && n.x < w - 1 && n.y > 0 && n.y < h - 1 && !occupied_cells.contains(n) && available_cells.contains(n)
            }) {
                // Check if a 3rd neighbor is available for a 3-block Tromino (30% chance)
                let make_tromino = rng.gen_bool(0.30);
                let cell3_opt = if make_tromino {
                    let n3_candidates = [
                        cell2 + IVec3::X, cell2 - IVec3::X, cell2 + IVec3::Y, cell2 - IVec3::Y,
                        cell1 + IVec3::X, cell1 - IVec3::X, cell1 + IVec3::Y, cell1 - IVec3::Y,
                    ];
                    n3_candidates.iter().copied().find(|n| {
                        *n != cell1 && *n != cell2 && n.x > 0 && n.x < w - 1 && n.y > 0 && n.y < h - 1 && !occupied_cells.contains(n) && available_cells.contains(n)
                    })
                } else {
                    None
                };

                // Remove cells from available pool
                available_cells.retain(|&c| c != cell1 && c != cell2 && Some(c) != cell3_opt);
                occupied_cells.insert(cell1);
                occupied_cells.insert(cell2);
                if let Some(c3) = cell3_opt {
                    occupied_cells.insert(c3);
                }

                let gid = world.next_combined_group_id();

                // Select group composition
                let group_type = rng.gen_range(0, 4);
                let (k1, k2) = match group_type {
                    0 => (BlockKind::Pushable, BlockKind::Mirror),
                    1 if spec.recipe.is_allowed(BlockKind::Mirror) => (BlockKind::Mirror, BlockKind::Mirror),
                    2 => (BlockKind::Pushable, BlockKind::Pushable),
                    3 if spec.recipe.is_allowed(BlockKind::Glass) => (BlockKind::Glass, BlockKind::Mirror),
                    _ => (BlockKind::Pushable, BlockKind::Mirror),
                };

                // Spawn block 1
                let id1 = world.spawn(k1, cell1, unit_shape());
                if let Some(b) = world.body_mut(id1) {
                    b.combined_group = Some(gid);
                    if k1 == BlockKind::Mirror {
                        b.orientation = *rng.choose(&orientations_4).unwrap_or(&CubeRot::IDENTITY);
                    }
                }

                // Spawn block 2
                let id2 = world.spawn(k2, cell2, unit_shape());
                if let Some(b) = world.body_mut(id2) {
                    b.combined_group = Some(gid);
                    if k2 == BlockKind::Mirror {
                        b.orientation = *rng.choose(&orientations_4).unwrap_or(&CubeRot::IDENTITY);
                    }
                }

                // Spawn optional block 3 for Tromino
                if let Some(c3) = cell3_opt {
                    let k3 = if rng.gen_bool(0.5) && spec.recipe.is_allowed(BlockKind::Mirror) {
                        BlockKind::Mirror
                    } else {
                        BlockKind::Pushable
                    };
                    let id3 = world.spawn(k3, c3, unit_shape());
                    if let Some(b) = world.body_mut(id3) {
                        b.combined_group = Some(gid);
                        if k3 == BlockKind::Mirror {
                            b.orientation = *rng.choose(&orientations_4).unwrap_or(&CubeRot::IDENTITY);
                        }
                    }
                }

                combined_spawned += 1;
            } else {
                i += 1;
            }
        }
    }

    // 5. Vertically Stacked Blocks (e.g. Mirror resting on top of Crate at Z=1 on Z=0)
    let num_stacked = rng.gen_range(spec.recipe.stacked_blocks.0, spec.recipe.stacked_blocks.1 + 1);
    if num_stacked > 0 && spec.recipe.is_allowed(BlockKind::Pushable) && spec.recipe.is_allowed(BlockKind::Mirror) {
        for _ in 0..num_stacked {
            if let Some(base_pos) = available_cells.pop() {
                occupied_cells.insert(base_pos);

                // Base block at Z=0
                let base_id = world.spawn(BlockKind::Pushable, base_pos, unit_shape());
                if rng.gen_bool(spec.recipe.fixed_mirror_chance) {
                    if let Some(b) = world.body_mut(base_id) {
                        b.tags.set(TagKind::Fixed, TagValue::Unit);
                    }
                }

                // Stacked block at Z=1
                let top_pos = base_pos + IVec3::Z;
                let top_id = world.spawn(BlockKind::Mirror, top_pos, unit_shape());
                if let Some(b) = world.body_mut(top_id) {
                    b.orientation = *rng.choose(&orientations_4).unwrap_or(&CubeRot::IDENTITY);
                }
            }
        }
    }

    if available_cells.len() < 4 {
        return None;
    }

    // 6. Spawn Player
    let player_pos = available_cells.pop()?;
    let pid = world.spawn(BlockKind::Player, player_pos, unit_shape());
    if let Some(b) = world.body_mut(pid) {
        let f = *rng.choose(&facings_4).unwrap_or(&IVec3::Y);
        b.orientation = CubeRot::from_facing_2d(f);
    }

    // 7. Spawn Laser Source(s)
    let num_lasers = rng.gen_range(spec.recipe.laser_sources.0, spec.recipe.laser_sources.1 + 1);
    if spec.recipe.is_allowed(BlockKind::LaserSource) {
        for _ in 0..num_lasers {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            let lid = world.spawn(BlockKind::LaserSource, pos, unit_shape());
            if let Some(b) = world.body_mut(lid) {
                let f = *rng.choose(&facings_4).unwrap_or(&IVec3::Y);
                b.orientation = CubeRot::from_facing_2d(f);
                if rng.gen_bool(spec.recipe.fixed_laser_chance) {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
            }
        }
    }

    // 8. Spawn Goal(s)
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

    // 9. Spawn Individual Mirrors
    let num_mirrors = rng.gen_range(spec.recipe.mirrors.0, spec.recipe.mirrors.1 + 1);
    if spec.recipe.is_allowed(BlockKind::Mirror) {
        for _ in 0..num_mirrors {
            let pos = match available_cells.pop() {
                Some(p) => p,
                None => break,
            };
            let mid = world.spawn(BlockKind::Mirror, pos, unit_shape());
            if let Some(b) = world.body_mut(mid) {
                b.orientation = *rng.choose(&orientations_4).unwrap_or(&CubeRot::IDENTITY);
                if rng.gen_bool(spec.recipe.fixed_mirror_chance) {
                    b.tags.set(TagKind::Fixed, TagValue::Unit);
                }
            }
        }
    }

    // 10. Spawn Pushable Crates
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

    // 11. Spawn Glass blocks if enabled
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

    // 12. Frame 0 validation: ensure level is physically valid and not immediately solved/dead
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
