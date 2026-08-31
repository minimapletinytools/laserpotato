//! Level definitions and test puzzles.
//!
//! No Bevy dependency. Each function returns a fully populated
//! [`World`](crate::sim::World) ready for play.

use glam::IVec3;

use crate::block_types::BlockKind;
use crate::sim::{CubeRot, TagKind, TagValue, World};

/// A single-cell (1×1×1) shape.
fn unit_shape() -> Vec<IVec3> {
    vec![IVec3::ZERO]
}

/// A non-trivial, multi-stage reflection puzzle.
///
/// ```text
///   Y
///   ^
/// 7 | W  W  W  W  W  W  W  W  W  W
/// 6 | W  .  @ MM2 .  .  .  G  .  W      G   = Goal Pyramid at (7, 6)
/// 5 | W  . MM1 .  W  .  W  .  .  W      MM2 = Moveable Mirror (start at 3,6, push East to 5,6)
/// 4 | W  .  .  .  . FM  .  .  .  W      FM  = Fixed Mirror at (5, 4), reflects +X → +Y
/// 3 | W  .  .  .  .  .  .  .  .  W      MM1 = Moveable Mirror (start at 2,5, push into 1,4)
/// 2 | W  .  .  .  .  .  .  .  .  W      L   = Moveable Laser Source at (1, 0), push to (1,1)
/// 1 | W  .  .  .  .  .  .  .  .  W      @   = Player starting at (2, 6)
/// 0 | W  .  L  .  .  .  .  .  .  W
/// -1| W  W  .  .  .  .  W  W  W  W
/// -2| W  W  W  W  W  W  W  W  W  W
///   +------------------------------> X
///     -1  0  1  2  3  4  5  6  7  8
/// ```
pub fn test_level() -> World {
    let mut world = World::new();

    // 1. Floor layer at Z = -1 (10x10 at 0,0)
    for x in 0..10 {
        for y in 0..10 {
            let floor_id = world.spawn(BlockKind::Floor, IVec3::new(x, y, -1), unit_shape());
            world.body_mut(floor_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
        }
    }

    // 2. Player character at (3, 8, 0)
    world.spawn(BlockKind::Player, IVec3::new(3, 8, 0), unit_shape());

    // 3. Moveable Laser Source at (2, 2, 0) — emits +Y
    world.spawn(BlockKind::LaserSource, IVec3::new(2, 2, 0), unit_shape());

    // 4. Moveable Mirror 1 at (3, 7, 0) — identity "/" orientation
    world.spawn(BlockKind::Mirror, IVec3::new(3, 7, 0), unit_shape());

    // 5. Fixed Mirror at (6, 6, 0) — ROT_Z_180 "/" orientation: reflects incoming +X → +Y
    let fixed_mirror_id = world.spawn(BlockKind::Mirror, IVec3::new(6, 6, 0), unit_shape());
    {
        let body = world.body_mut(fixed_mirror_id).unwrap();
        body.orientation = CubeRot::ROT_Z_180;
        body.tags.set(TagKind::Fixed, TagValue::Unit);
    }

    // 6. Moveable Mirror 2 at (4, 8, 0) — identity "/" orientation
    world.spawn(BlockKind::Mirror, IVec3::new(4, 8, 0), unit_shape());

    // 7. Goal Pyramid at (8, 8, 0)
    let goal_id = world.spawn(BlockKind::Goal, IVec3::new(8, 8, 0), unit_shape());
    world.body_mut(goal_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

    // 8. Interior partition walls creating rooms and obstacle channels
    let wall_coords = [
        IVec3::new(1, 7, 0),
        IVec3::new(5, 7, 0),
        IVec3::new(7, 7, 0),
    ];
    for pos in wall_coords {
        let wall_id = world.spawn(BlockKind::Wall, pos, unit_shape());
        world.body_mut(wall_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
    }

    // 9. Perimeter border walls (X: 0..=9, Y: 0..=9 at Z = 0)
    for x in 0..10 {
        let w_bot = world.spawn(BlockKind::Wall, IVec3::new(x, 0, 0), unit_shape());
        world.body_mut(w_bot).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

        let w_top = world.spawn(BlockKind::Wall, IVec3::new(x, 9, 0), unit_shape());
        world.body_mut(w_top).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
    }
    for y in 1..9 {
        let w_left = world.spawn(BlockKind::Wall, IVec3::new(0, y, 0), unit_shape());
        world.body_mut(w_left).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

        let w_right = world.spawn(BlockKind::Wall, IVec3::new(9, y, 0), unit_shape());
        world.body_mut(w_right).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
    }
    // Partition wall on row 1 with open passage at x in [2..=5]
    for x in [1, 6, 7, 8] {
        let wall_id = world.spawn(BlockKind::Wall, IVec3::new(x, 1, 0), unit_shape());
        world.body_mut(wall_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
    }

    world.sync_grid();
    world
}

// ---------------------------------------------------------------------------
// Level Serialization & File Management
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Serializable representation of a single block body in a puzzle level.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelBodyData {
    pub kind: BlockKind,
    pub anchor: [i32; 3],
    pub orientation: CubeRot,
    pub fixed: bool,
    #[serde(default)]
    pub combined_group: Option<u32>,
}

/// Serializable puzzle level data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelData {
    pub name: String,
    pub bodies: Vec<LevelBodyData>,
}

impl LevelData {
    /// Convert an active [`World`] into a [`LevelData`] snapshot.
    pub fn from_world(name: impl Into<String>, world: &World) -> Self {
        let mut bodies = Vec::new();
        for body in world.bodies() {
            bodies.push(LevelBodyData {
                kind: body.kind,
                anchor: [body.anchor.x, body.anchor.y, body.anchor.z],
                orientation: body.orientation,
                fixed: body.is_fixed(),
                combined_group: body.combined_group,
            });
        }
        Self {
            name: name.into(),
            bodies,
        }
    }

    /// Reconstruct a playable [`World`] from this [`LevelData`].
    pub fn to_world(&self) -> World {
        let mut world = World::new();
        let mut max_group = 0;
        for b in &self.bodies {
            let id = world.spawn(
                b.kind,
                IVec3::new(b.anchor[0], b.anchor[1], b.anchor[2]),
                unit_shape(),
            );
            if let Some(body) = world.body_mut(id) {
                body.orientation = b.orientation;
                if b.fixed {
                    body.tags.set(TagKind::Fixed, TagValue::Unit);
                }
                body.combined_group = b.combined_group;
                if let Some(g) = b.combined_group {
                    if g >= max_group {
                        max_group = g + 1;
                    }
                }
            }
        }
        // Hack: next_group_id isn't public, so we'll just set it via loop if we could.
        // Wait, world doesn't have a way to set next_group_id. I will just loop to catch up.
        while world.next_combined_group_id() <= max_group {
            // catch up
        }
        
        world.sync_grid();
        world
    }
}

/// Compute a 64-bit fingerprint hash of the given world state.
///
/// If any body's position, kind, orientation, or fixed property changes,
/// the hash will change, allowing automatic invalidation of stale solutions.
pub fn compute_level_hash(world: &World) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut entries: Vec<(BlockKind, [i32; 3], [[i32; 3]; 3], bool)> = world
        .bodies()
        .iter()
        .map(|b| {
            (
                b.kind,
                [b.anchor.x, b.anchor.y, b.anchor.z],
                b.canonical_orientation().mat,
                b.is_fixed(),
            )
        })
        .collect();
    entries.sort();
    entries.hash(&mut hasher);
    hasher.finish()
}

/// Save level data to a JSON file.
pub fn save_level_to_file(path: impl AsRef<Path>, level: &LevelData) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(level)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Load level data from a JSON file.
pub fn load_level_from_file(path: impl AsRef<Path>) -> std::io::Result<LevelData> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Enumerate all `.json` level files in `levels/` directory.
pub fn list_level_files() -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir("levels") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Some(s) = p.to_str() {
                    files.push(s.to_string());
                }
            }
        }
    }
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::CubeRot;

    #[test]
    fn level_serialization_round_trip() {
        let world = test_level();
        let level_data = LevelData::from_world("Default Puzzle", &world);
        let _ = save_level_to_file("levels/default_puzzle.json", &level_data);
        let world_reconstructed = level_data.to_world();

        assert_eq!(world.bodies().len(), world_reconstructed.bodies().len());
        assert_eq!(compute_level_hash(&world), compute_level_hash(&world_reconstructed));
    }

    #[test]
    fn modifying_body_changes_level_hash() {
        let world1 = test_level();
        let mut world2 = test_level();

        let hash1 = compute_level_hash(&world1);
        let hash2 = compute_level_hash(&world2);
        assert_eq!(hash1, hash2);

        // Move a block
        let id = world2.body_at(IVec3::new(3, 8, 0)).unwrap().id;
        world2.body_mut(id).unwrap().anchor = IVec3::new(3, 7, 0);
        world2.sync_grid();

        let hash3 = compute_level_hash(&world2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn rotating_isotropic_block_preserves_level_hash() {
        let world1 = test_level();
        let mut world2 = test_level();

        // Rotate a wall in world2
        let wall_id = world2.body_at(IVec3::new(0, 5, 0)).unwrap().id;
        world2.body_mut(wall_id).unwrap().orientation = CubeRot::ROT_Z_90;
        world2.sync_grid();

        assert_eq!(
            compute_level_hash(&world1),
            compute_level_hash(&world2),
            "Rotating an isotropic wall should preserve the canonical level hash"
        );

        // Rotating a directional mirror in world2 MUST change the level hash
        let mirror_id = world2.body_at(IVec3::new(3, 7, 0)).unwrap().id;
        world2.body_mut(mirror_id).unwrap().orientation = CubeRot::ROT_Z_90;
        world2.sync_grid();

        assert_ne!(
            compute_level_hash(&world1),
            compute_level_hash(&world2),
            "Rotating a mirror changes puzzle behavior and must produce a different level hash"
        );
    }
}
