//! Level definitions and test puzzles.
//!
//! No Bevy dependency. Each function returns a fully populated
//! [`World`](crate::sim::World) ready for play.

use glam::IVec3;

use crate::block_types::BlockKind;
use crate::sim::World;

/// A single-cell (1×1×1) shape.
fn unit_shape() -> Vec<IVec3> {
    vec![IVec3::ZERO]
}

/// Test puzzle that exercises pushing, laser occlusion, and mirror reflection.
///
/// ```text
///     Y
///     ^
///     |
///   4 |  W  W  W  W  W  W  W
///   3 |  W  .  M  .  .  .  W      M = Mirror ("/" at identity, reflects +Y → +X)
///   2 |  W  P  .  .  .  .  W      P = Pushable block
///   1 |  W  .  .  .  .  .  W      @ = Player
///   0 |  W  @  L  .  .  .  W      L = Laser source (pointing +Y)
///  -1 |  W  W  W  W  W  W  W
///     +--------------------->  X
///       -1  0  1  2  3  4  5
/// ```
///
/// **What this tests:**
/// - Player pushes `P` northward.
/// - Laser `L` fires +Y, hits mirror `M` at (1,3), reflects +X.
/// - Pushing `P` into the laser path (column x=1) blocks the beam before it
///   reaches the mirror.
/// - Pushing `M` changes the reflected beam's endpoint.
pub fn test_level() -> World {
    let mut world = World::new();

    // -- interactive bodies ------------------------------------------------
    // Player at (0, 0, 0)
    world.spawn(BlockKind::Player, IVec3::new(0, 0, 0), unit_shape());

    // Laser source at (1, 0, 0) — default orientation emits +Y
    world.spawn(BlockKind::LaserSource, IVec3::new(1, 0, 0), unit_shape());

    // Pushable block at (0, 2, 0)
    world.spawn(BlockKind::Pushable, IVec3::new(0, 2, 0), unit_shape());

    // Mirror at (1, 3, 0) — identity "/" orientation: +Y → +X
    world.spawn(BlockKind::Mirror, IVec3::new(1, 3, 0), unit_shape());

    // -- walls (border) ----------------------------------------------------
    for x in -1..=5 {
        world.spawn(BlockKind::Wall, IVec3::new(x, -1, 0), unit_shape());
        world.spawn(BlockKind::Wall, IVec3::new(x, 4, 0), unit_shape());
    }
    for y in 0..=3 {
        world.spawn(BlockKind::Wall, IVec3::new(-1, y, 0), unit_shape());
        world.spawn(BlockKind::Wall, IVec3::new(5, y, 0), unit_shape());
    }

    world
}
