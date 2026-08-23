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

/// Large interactive puzzle level with moveable and fixed blocks, moveable mirrors,
/// moveable laser source, fixed obstacles, and a target Goal pyramid.
///
/// ```text
///   Y
///   ^
/// 7 | W  W  W  W  W  W  W  W  W  W  W
/// 6 | W  .  G  .  W  .  .  FM .  .  W      G  = Goal Pyramid at (1, 6)
/// 5 | W  .  .  .  W  .  .  .  .  .  W      FM = Fixed Mirror at (6, 6), reflects +Y → -X
/// 4 | W  .  .  .  W  .  MM .  .  .  W      W  = Wall pillars dividing chambers
/// 3 | W  .  .  P  .  .  .  .  .  .  W      MM = Moveable Mirror at (5, 4), reflects +Y → +X
/// 2 | W  @  .  .  .  .  .  .  .  .  W      P  = Moveable Pushable crate at (2, 3)
/// 1 | W  .  .  ML .  .  .  .  .  .  W      ML = Moveable Laser source at (2, 1), fires +Y
/// 0 | W  .  .  .  .  .  .  .  .  .  W      @  = Player character at (0, 2)
/// -1| W  W  W  W  W  W  W  W  W  W  W
///   +----------------------------------> X
///     -1  0  1  2  3  4  5  6  7  8  9
/// ```
pub fn test_level() -> World {
    let mut world = World::new();

    // 1. Player character at (0, 2, 0)
    world.spawn(BlockKind::Player, IVec3::new(0, 2, 0), unit_shape());

    // 2. Moveable Laser Source at (2, 1, 0) — default orientation emits +Y
    world.spawn(BlockKind::LaserSource, IVec3::new(2, 1, 0), unit_shape());

    // 3. Moveable Pushable Block at (2, 3, 0)
    world.spawn(BlockKind::Pushable, IVec3::new(2, 3, 0), unit_shape());

    // 4. Moveable Mirror at (5, 4, 0) — identity "/" orientation: +Y → +X
    world.spawn(BlockKind::Mirror, IVec3::new(5, 4, 0), unit_shape());

    // 5. Fixed Mirror at (6, 6, 0) — rotated 90° about Z: reflects +Y → -X
    let fixed_mirror_id = world.spawn(BlockKind::Mirror, IVec3::new(6, 6, 0), unit_shape());
    {
        let mirror = world.body_mut(fixed_mirror_id).unwrap();
        mirror.orientation = CubeRot::ROT_Z_90;
        mirror.tags.set(TagKind::Fixed, TagValue::Unit);
    }

    // 6. Target Goal Pyramid at (1, 6, 0)
    let goal_id = world.spawn(BlockKind::Goal, IVec3::new(1, 6, 0), unit_shape());
    world.body_mut(goal_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

    // 7. Interior partition wall pillars
    for y in 4..=6 {
        let wall_id = world.spawn(BlockKind::Wall, IVec3::new(3, y, 0), unit_shape());
        world.body_mut(wall_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
    }

    // 8. Perimeter border walls (X: -1..=8, Y: -1..=7)
    for x in -1..=8 {
        world.spawn(BlockKind::Wall, IVec3::new(x, -1, 0), unit_shape());
        world.spawn(BlockKind::Wall, IVec3::new(x, 7, 0), unit_shape());
    }
    for y in 0..=6 {
        world.spawn(BlockKind::Wall, IVec3::new(-1, y, 0), unit_shape());
        world.spawn(BlockKind::Wall, IVec3::new(8, y, 0), unit_shape());
    }

    world.sync_grid();
    world
}
