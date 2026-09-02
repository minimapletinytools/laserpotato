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

pub use crate::turn::validate_solution;

/// A named sequence of player actions representing a valid solution to a puzzle,
/// optionally paired with quality/epiphany analysis metadata.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LevelSolution {
    pub name: String,
    pub actions: Vec<crate::turn::PlayerAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<crate::solver::PuzzleProfile>,
}

impl LevelSolution {
    pub fn new(name: impl Into<String>, actions: Vec<crate::turn::PlayerAction>) -> Self {
        Self {
            name: name.into(),
            actions,
            profile: None,
        }
    }

    pub fn with_profile(
        name: impl Into<String>,
        actions: Vec<crate::turn::PlayerAction>,
        profile: Option<crate::solver::PuzzleProfile>,
    ) -> Self {
        Self {
            name: name.into(),
            actions,
            profile,
        }
    }
}

/// Serializable puzzle level data.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LevelData {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub bodies: Vec<LevelBodyData>,
    #[serde(default)]
    pub solutions: Vec<LevelSolution>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_profile: Option<crate::solver::PuzzleProfile>,
}

impl LevelData {
    /// Convert an active [`World`] into a [`LevelData`] snapshot with solutions and quality profile.
    pub fn from_world_with_solutions_and_profile(
        name: impl Into<String>,
        world: &World,
        solutions: Vec<LevelSolution>,
        quality_profile: Option<crate::solver::PuzzleProfile>,
    ) -> Self {
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
            description: None,
            bodies,
            solutions,
            quality_profile,
        }
    }

    /// Convert an active [`World`] into a [`LevelData`] snapshot with solutions.
    pub fn from_world_with_solutions(name: impl Into<String>, world: &World, solutions: Vec<LevelSolution>) -> Self {
        Self::from_world_with_solutions_and_profile(name, world, solutions, None)
    }

    /// Convert an active [`World`] into a [`LevelData`] snapshot.
    pub fn from_world(name: impl Into<String>, world: &World) -> Self {
        Self::from_world_with_solutions_and_profile(name, world, Vec::new(), None)
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

/// Kind of entry in the level file picker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePickerEntryKind {
    Directory,
    JsonLevelFile,
}

/// An entry (directory or level file) displayed in the file picker dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePickerEntry {
    pub name: String,
    pub path: String,
    pub kind: FilePickerEntryKind,
}

/// List subdirectories and `.json` level files in `dir_path`, along with an optional parent path to go up.
pub fn list_directory_entries(dir_path: &str) -> (Option<String>, Vec<FilePickerEntry>) {
    let raw_path = if dir_path.is_empty() { "levels" } else { dir_path };
    let path = Path::new(raw_path);

    let parent_path = path.parent().and_then(|p| {
        let s = p.to_string_lossy().to_string();
        if s.is_empty() {
            if raw_path != "." && raw_path != "levels" {
                Some(".".to_string())
            } else if raw_path == "levels" {
                Some(".".to_string())
            } else {
                None
            }
        } else {
            Some(s)
        }
    });

    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(path) {
        for entry in read_dir.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if name.starts_with('.') {
                continue; // Skip hidden items
            }
            if p.is_dir() {
                entries.push(FilePickerEntry {
                    name,
                    path: p.to_string_lossy().to_string(),
                    kind: FilePickerEntryKind::Directory,
                });
            } else if p.is_file() && p.extension().and_then(|ext| ext.to_str()) == Some("json") {
                entries.push(FilePickerEntry {
                    name,
                    path: p.to_string_lossy().to_string(),
                    kind: FilePickerEntryKind::JsonLevelFile,
                });
            }
        }
    }

    // Sort: directories first, then alphabetical by name
    entries.sort_by(|a, b| {
        match (&a.kind, &b.kind) {
            (FilePickerEntryKind::Directory, FilePickerEntryKind::JsonLevelFile) => std::cmp::Ordering::Less,
            (FilePickerEntryKind::JsonLevelFile, FilePickerEntryKind::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    (parent_path, entries)
}

/// Summary metadata row for the Level Tester browser table.
#[derive(Clone, Debug, PartialEq)]
pub struct TesterLevelEntry {
    pub path: String,
    pub filename: String,
    pub name: String,
    pub description: String,
    pub is_directory: bool,
    pub macro_steps: u32,
    pub atomic_turns: u32,
    pub epiphany: f32,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub mirrors: u32,
    pub crates: u32,
    pub polyominos: u32,
    pub lasers: u32,
    pub goals: u32,
    pub total_blocks: u32,
    pub load_bearing_pct: f32,
    pub has_comment: bool,
}

/// Create a directory entry row for the Level Tester browser table.
pub fn extract_tester_dir_entry(path: &str, name: &str) -> TesterLevelEntry {
    TesterLevelEntry {
        path: path.to_string(),
        filename: name.to_string(),
        name: format!("📁 {}/", name),
        description: "Folder".into(),
        is_directory: true,
        macro_steps: 0,
        atomic_turns: 0,
        epiphany: 0.0,
        width: 0,
        height: 0,
        depth: 0,
        mirrors: 0,
        crates: 0,
        polyominos: 0,
        lasers: 0,
        goals: 0,
        total_blocks: 0,
        load_bearing_pct: 0.0,
        has_comment: false,
    }
}

/// Extract summary metadata from a level file for the Level Tester table.
pub fn extract_tester_level_entry(path: &str, filename: &str) -> Option<TesterLevelEntry> {
    let lvl = load_level_from_file(path).ok()?;
    let macro_steps = lvl
        .quality_profile
        .as_ref()
        .map(|p| p.macro_steps as u32)
        .or_else(|| {
            lvl.solutions
                .first()
                .and_then(|s| s.profile.as_ref())
                .map(|p| p.macro_steps as u32)
        })
        .unwrap_or(0);

    let atomic_turns = lvl
        .quality_profile
        .as_ref()
        .map(|p| p.atomic_turns as u32)
        .or_else(|| {
            lvl.solutions
                .first()
                .and_then(|s| s.profile.as_ref())
                .map(|p| p.atomic_turns as u32)
        })
        .or_else(|| lvl.solutions.first().map(|s| s.actions.len() as u32))
        .unwrap_or(0);

    let epiphany = lvl
        .quality_profile
        .as_ref()
        .map(|p| p.epiphany_score)
        .or_else(|| {
            lvl.solutions
                .first()
                .and_then(|s| s.profile.as_ref())
                .map(|p| p.epiphany_score)
        })
        .unwrap_or(0.0);

    let load_bearing_pct = lvl
        .quality_profile
        .as_ref()
        .map(|p| p.load_bearing_factor * 100.0)
        .unwrap_or(100.0);

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;

    let mut mirrors = 0;
    let mut crates = 0;
    let mut lasers = 0;
    let mut goals = 0;
    let mut polyomino_groups = std::collections::HashSet::new();

    for b in &lvl.bodies {
        min_x = min_x.min(b.anchor[0]);
        max_x = max_x.max(b.anchor[0]);
        min_y = min_y.min(b.anchor[1]);
        max_y = max_y.max(b.anchor[1]);
        min_z = min_z.min(b.anchor[2]);
        max_z = max_z.max(b.anchor[2]);

        match b.kind {
            BlockKind::Mirror => mirrors += 1,
            BlockKind::Pushable => crates += 1,
            BlockKind::LaserSource => lasers += 1,
            BlockKind::Goal => goals += 1,
            _ => {}
        }
        if let Some(g) = b.combined_group {
            polyomino_groups.insert(g);
        }
    }

    let width = if min_x <= max_x { max_x - min_x + 1 } else { 0 };
    let height = if min_y <= max_y { max_y - min_y + 1 } else { 0 };
    let depth = if min_z <= max_z { max_z - min_z + 1 } else { 0 };
    let polyominos = polyomino_groups.len() as u32;
    let total_blocks = mirrors + crates + lasers + goals;
    let description = lvl.description.clone().unwrap_or_default();
    let has_comment = !description.trim().is_empty();

    Some(TesterLevelEntry {
        path: path.to_string(),
        filename: filename.to_string(),
        name: if lvl.name.is_empty() {
            filename.to_string()
        } else {
            lvl.name
        },
        description,
        is_directory: false,
        macro_steps,
        atomic_turns,
        epiphany,
        width,
        height,
        depth,
        mirrors,
        crates,
        polyominos,
        lasers,
        goals,
        total_blocks,
        load_bearing_pct,
        has_comment,
    })
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

    #[test]
    fn list_directory_entries_test() {
        let (parent, entries) = list_directory_entries("levels");
        assert!(parent.is_some());
        assert!(!entries.is_empty());

        let has_json = entries.iter().any(|e| e.kind == FilePickerEntryKind::JsonLevelFile);
        assert!(has_json);
    }

    #[test]
    fn level_solution_serialization_test() {
        let world = test_level();
        let solutions = vec![
            LevelSolution::new(
                "Solver Solution",
                vec![crate::turn::PlayerAction::Forward, crate::turn::PlayerAction::TurnLeft],
            )
        ];
        let profile = crate::solver::analyze_puzzle(&world);
        let data = LevelData::from_world_with_solutions_and_profile(
            "Test Level With Solutions",
            &world,
            solutions.clone(),
            Some(profile.clone()),
        );
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: LevelData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.solutions, solutions);
        assert_eq!(deserialized.quality_profile, Some(profile));
    }
}
