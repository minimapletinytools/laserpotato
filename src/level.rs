//! Level definitions and test puzzles.
//!
//! No Bevy dependency. Each function returns a fully populated
//! [`World`](crate::sim::World) ready for play.

use glam::IVec3;

use crate::block_types::BlockKind;
use crate::sim::{TagKind, TagValue, World};

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
/// 6 | W  .  .  .  .  MM2.  G  .  W      G   = Goal Pyramid at (7, 6)
/// 5 | W  W  .  W  W  .  W  .  .  W      MM2 = Moveable Mirror (start at 4,1, push to 5,6)
/// 4 | W  .  .  MM1.  FM .  .  .  W      FM  = Fixed Mirror at (5, 4), reflects +X → +Y
/// 3 | W  .  .  .  .  .  .  .  .  W      MM1 = Moveable Mirror (start at 3,4, push to 1,4)
/// 2 | W  .  P  @  .  .  .  .  .  W      P   = Moveable Crate at (1, 2)
/// 1 | W  .  .  .  .  .  .  .  .  W      L   = Fixed Laser Source at (1, 0), fires +Y
/// 0 | W  .  L  .  .  .  .  .  .  W      @   = Player starting at (2, 2)
/// -1| W  W  W  W  W  W  W  W  W  W
///   +------------------------------> X
///     -1  0  1  2  3  4  5  6  7  8
/// ```
pub fn test_level() -> World {
    let mut world = World::new();

    // 1. Player character at (2, 2, 0)
    world.spawn(BlockKind::Player, IVec3::new(2, 2, 0), unit_shape());

    // 2. Fixed Laser Source at (1, 0, 0) — emits +Y
    let laser_id = world.spawn(BlockKind::LaserSource, IVec3::new(1, 0, 0), unit_shape());
    world.body_mut(laser_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

    // 3. Moveable Blocker Crate at (1, 2, 0) (initially caps laser beam)
    world.spawn(BlockKind::Pushable, IVec3::new(1, 2, 0), unit_shape());

    // 4. Moveable Mirror 1 at (3, 4, 0) — identity "/" orientation
    world.spawn(BlockKind::Mirror, IVec3::new(3, 4, 0), unit_shape());

    // 5. Fixed Mirror at (5, 4, 0) — identity "/" orientation: reflects +X → +Y
    let fixed_mirror_id = world.spawn(BlockKind::Mirror, IVec3::new(5, 4, 0), unit_shape());
    world.body_mut(fixed_mirror_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

    // 6. Moveable Mirror 2 at (4, 1, 0) — identity "/" orientation
    world.spawn(BlockKind::Mirror, IVec3::new(4, 1, 0), unit_shape());

    // 7. Goal Pyramid at (7, 6, 0)
    let goal_id = world.spawn(BlockKind::Goal, IVec3::new(7, 6, 0), unit_shape());
    world.body_mut(goal_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);

    // 8. Interior partition walls creating rooms and obstacle channels
    let wall_coords = [
        IVec3::new(0, 5, 0),
        IVec3::new(2, 5, 0),
        IVec3::new(3, 5, 0),
        IVec3::new(4, 5, 0),
    ];
    for pos in wall_coords {
        let wall_id = world.spawn(BlockKind::Wall, pos, unit_shape());
        world.body_mut(wall_id).unwrap().tags.set(TagKind::Fixed, TagValue::Unit);
    }

    // 9. Perimeter border walls (X: -1..=8, Y: -1..=7)
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
