//! Block type definitions — the fixed vocabulary of game objects.
//!
//! [`BlockKind`] captures the *type identity* of a block. Per-instance mutable
//! state (e.g. "has been hit by a laser 2 times") lives in [`super::sim::TagSet`].

use serde::{Deserialize, Serialize};
use std::fmt;

/// The fixed type identity of a block, determining its behavior during
/// turn resolution (pushability, solidity, laser interactions).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlockKind {
    /// The player-controlled character.
    Player,
    /// Immovable wall / boundary.
    Wall,
    /// Standard pushable crate.
    Pushable,
    /// 45° reflector — redirects laser beams. In local space the default
    /// (identity-orientation) mirror acts as a "/" reflector, swapping the
    /// X and Y components of an incoming beam direction.
    Mirror,
    /// Emits a laser beam in its forward (+Y local) direction.
    LaserSource,
    /// Target goal pyramid. When struck by a laser beam, the puzzle level is completed.
    Goal,
}

impl BlockKind {
    /// Whether this block kind is inherently pushable when untagged.
    pub fn is_pushable(self) -> bool {
        matches!(self, Self::Pushable | Self::Mirror | Self::LaserSource)
    }

    /// Whether this block prevents other blocks from entering its cells.
    pub fn is_solid(self) -> bool {
        true // all current block types are solid
    }

    /// Movement priority — lower values are processed first in the
    /// movement queue. The player always moves first.
    pub fn movement_priority(self) -> u32 {
        match self {
            Self::Player => 0,
            _ => 100,
        }
    }
}

impl fmt::Display for BlockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Player => write!(f, "Player"),
            Self::Wall => write!(f, "Wall"),
            Self::Pushable => write!(f, "Pushable"),
            Self::Mirror => write!(f, "Mirror"),
            Self::LaserSource => write!(f, "LaserSource"),
            Self::Goal => write!(f, "Goal"),
        }
    }
}
