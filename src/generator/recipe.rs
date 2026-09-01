//! Generator recipe configuration for controlling puzzle quotas and allowed/omitted mechanics.

use std::collections::HashSet;
use crate::block_types::BlockKind;

/// Configuration for block quotas and allowed/omitted mechanics in the procedural puzzle generator.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockRecipe {
    /// Explicit set of allowed block types. If empty, all non-omitted block types are allowed.
    pub allowed_blocks: HashSet<BlockKind>,
    /// Explicit set of omitted/disabled block types.
    pub omitted_blocks: HashSet<BlockKind>,
    /// Minimum and maximum number of Laser Source emitters.
    pub laser_sources: (u32, u32),
    /// Minimum and maximum number of Goal Pyramids.
    pub goals: (u32, u32),
    /// Minimum and maximum number of Mirrors.
    pub mirrors: (u32, u32),
    /// Minimum and maximum number of Pushable Crates.
    pub crates: (u32, u32),
    /// Minimum and maximum number of Glass Blocks.
    pub glass: (u32, u32),
    /// Probability that an internal mirror is spawned fixed/stationary instead of pushable.
    pub fixed_mirror_chance: f32,
    /// Probability that a laser source is spawned fixed/stationary instead of pushable.
    pub fixed_laser_chance: f32,
    /// Interior wall obstacle density (0.0 to 1.0).
    pub wall_density: f32,
}

impl Default for BlockRecipe {
    fn default() -> Self {
        Self {
            allowed_blocks: HashSet::new(),
            omitted_blocks: HashSet::new(),
            laser_sources: (1, 1),
            goals: (1, 1),
            mirrors: (1, 3),
            crates: (0, 2),
            glass: (0, 1),
            fixed_mirror_chance: 0.25,
            fixed_laser_chance: 0.50,
            wall_density: 0.15,
        }
    }
}

impl BlockRecipe {
    /// Check if a specific block kind is allowed by this recipe.
    pub fn is_allowed(&self, kind: BlockKind) -> bool {
        if self.omitted_blocks.contains(&kind) {
            return false;
        }
        if !self.allowed_blocks.is_empty() && !self.allowed_blocks.contains(&kind) {
            return false;
        }
        true
    }

    /// Omit a specific mechanic/block kind.
    pub fn omit(&mut self, kind: BlockKind) -> &mut Self {
        self.omitted_blocks.insert(kind);
        self
    }

    /// Explicitly allow a specific mechanic/block kind.
    pub fn allow(&mut self, kind: BlockKind) -> &mut Self {
        self.allowed_blocks.insert(kind);
        self
    }

    /// Parse a block kind by name with strict error handling if an unknown mechanic is provided.
    pub fn parse_block_kind(name: &str) -> Result<BlockKind, String> {
        match name.trim().to_lowercase().as_str() {
            "player" => Ok(BlockKind::Player),
            "wall" => Ok(BlockKind::Wall),
            "floor" => Ok(BlockKind::Floor),
            "mirror" => Ok(BlockKind::Mirror),
            "laser" | "lasersource" | "laser_source" => Ok(BlockKind::LaserSource),
            "goal" | "target" => Ok(BlockKind::Goal),
            "pushable" | "crate" | "box" => Ok(BlockKind::Pushable),
            "glass" => Ok(BlockKind::Glass),
            other => Err(format!(
                "Unknown mechanic/block type '{}'. Supported: player, wall, floor, mirror, laser, goal, pushable, glass",
                other
            )),
        }
    }
}
