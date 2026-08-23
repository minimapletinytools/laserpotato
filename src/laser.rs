//! Laser ray casting and reflection based on generic block face properties.
//!
//! No Bevy dependency. Given a [`World`](crate::sim::World), this module casts
//! rays from every [`LaserSource`](crate::block_types::BlockKind::LaserSource)
//! body, queries generic per-face reflection properties of struck bodies, and
//! records what each beam hits.

use std::collections::HashSet;

use glam::IVec3;

use crate::sim::{BodyId, World};

/// Maximum number of cells a single ray will traverse before giving up.
const MAX_RAY_LENGTH: usize = 100;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single laser beam segment — from a source (or reflecting block face) to the
/// point where it terminates (hit a block, left the max range, or looped).
#[derive(Clone, Debug)]
pub struct LaserSegment {
    /// The body that emitted or reflected this segment.
    pub source_id: BodyId,
    /// Cell the ray originates from (one cell past the source/reflector body).
    pub origin: IVec3,
    /// Unit direction the ray travels.
    pub direction: IVec3,
    /// All *empty* cells the beam passes through (excludes source and hit cells).
    pub cells: Vec<IVec3>,
    /// What the beam hit at the end, if anything.
    pub hit: Option<LaserHit>,
}

/// Records what a laser beam hit.
#[derive(Clone, Debug)]
pub struct LaserHit {
    pub body_id: BodyId,
    pub cell: IVec3,
}

// ---------------------------------------------------------------------------
// Casting
// ---------------------------------------------------------------------------

/// Cast all lasers in the world and return the complete set of beam segments.
///
/// Reflected faces generate secondary segments that are also included. Closed
/// loops are detected via a visited set of `(origin, direction)` pairs.
pub fn cast_all_lasers(world: &World) -> Vec<LaserSegment> {
    let mut segments = Vec::new();
    // Track (origin, direction) pairs to detect mirror loops.
    let mut visited: HashSet<(IVec3, IVec3)> = HashSet::new();

    // Seed the work queue with every laser emitter body.
    let mut queue: Vec<(BodyId, IVec3, IVec3)> = world
        .bodies()
        .iter()
        .filter_map(|b| {
            let emit_local = b.properties().emits_laser_towards?;
            let world_dir = b.orientation.apply(emit_local);
            Some((b.id, b.anchor, world_dir))
        })
        .collect();

    while let Some((source_id, source_pos, direction)) = queue.pop() {
        let origin = source_pos + direction;

        // Loop / duplicate detection.
        if !visited.insert((origin, direction)) {
            continue;
        }

        let mut cells = Vec::new();
        let mut current = origin;
        let mut hit = None;

        for _ in 0..MAX_RAY_LENGTH {
            // Check if this cell is occupied by a body.
            if let Some(body) = world.body_at(current) {
                // Query generic per-face reflection property:
                if let Some(reflected_dir) = body.properties().reflect_laser(direction, &body.orientation) {
                    queue.push((body.id, body.anchor, reflected_dir));
                }

                hit = Some(LaserHit {
                    body_id: body.id,
                    cell: current,
                });
                break;
            }

            cells.push(current);
            current += direction;
        }

        segments.push(LaserSegment {
            source_id,
            origin,
            direction,
            cells,
            hit,
        });
    }

    segments
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;
    use crate::sim::{CubeRot, World};

    #[test]
    fn straight_laser_hits_wall() {
        let mut world = World::new();
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        world.spawn(BlockKind::Wall, IVec3::new(0, 5, 0), vec![IVec3::ZERO]);

        let segments = cast_all_lasers(&world);
        assert_eq!(segments.len(), 1);
        // Beam passes through cells (0,1,0)..(0,4,0) — 4 empty cells.
        assert_eq!(segments[0].cells.len(), 4);
        assert!(segments[0].hit.is_some());
        let hit = segments[0].hit.as_ref().unwrap();
        assert_eq!(hit.cell, IVec3::new(0, 5, 0));
    }

    #[test]
    fn laser_into_void_caps_at_max_length() {
        let mut world = World::new();
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);

        let segments = cast_all_lasers(&world);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].cells.len(), MAX_RAY_LENGTH);
        assert!(segments[0].hit.is_none());
    }

    #[test]
    fn mirror_reflects_front_beam() {
        let mut world = World::new();
        // Laser at origin pointing +Y.
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        // Single-sided "/" mirror at (0,3,0) — reflects +Y → +X.
        world.spawn(BlockKind::Mirror, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);
        // Wall at (5,3,0) to stop reflected beam.
        world.spawn(BlockKind::Wall, IVec3::new(5, 3, 0), vec![IVec3::ZERO]);

        let segments = cast_all_lasers(&world);
        // Should have 2 segments: source→mirror, mirror→wall.
        assert_eq!(segments.len(), 2);

        // Find the reflected segment (direction = +X).
        let reflected = segments.iter().find(|s| s.direction == IVec3::new(1, 0, 0));
        assert!(reflected.is_some());
        let reflected = reflected.unwrap();
        assert!(reflected.hit.is_some());
        assert_eq!(reflected.hit.as_ref().unwrap().cell, IVec3::new(5, 3, 0));
    }

    #[test]
    fn mirror_blocks_back_beam() {
        let mut world = World::new();
        // Laser at (0, 6, 0) pointing -Y (South).
        let lid = world.spawn(BlockKind::LaserSource, IVec3::new(0, 6, 0), vec![IVec3::ZERO]);
        world.body_mut(lid).unwrap().orientation = CubeRot::ROT_Z_180;
        // Single-sided "/" mirror at (0, 3, 0) (Identity orientation).
        // Front faces South-West; -Y ray strikes North (back) face.
        world.spawn(BlockKind::Mirror, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);
        world.sync_grid();

        let segments = cast_all_lasers(&world);
        // Should have only 1 segment (hits back of mirror and stops with NO reflection).
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].hit.as_ref().unwrap().cell, IVec3::new(0, 3, 0));
    }

    #[test]
    fn rotated_mirror_reflects_differently() {
        let mut world = World::new();
        // Laser at origin pointing +Y.
        world.spawn(BlockKind::LaserSource, IVec3::ZERO, vec![IVec3::ZERO]);
        // Mirror at (0,3,0) rotated 90° CCW about Z — reflects +Y → -X.
        let mirror_id = world.spawn(BlockKind::Mirror, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);
        world.body_mut(mirror_id).unwrap().orientation = CubeRot::ROT_Z_90;
        world.sync_grid();

        let segments = cast_all_lasers(&world);
        assert_eq!(segments.len(), 2);

        let reflected = segments.iter().find(|s| s.direction == IVec3::new(-1, 0, 0));
        assert!(reflected.is_some(), "expected beam reflected to -X");
    }

    #[test]
    fn closed_mirror_loop_terminates() {
        let mut world = World::new();
        // Laser pointing +Y from (0, 0).
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);

        // Square of 4 mirrors forming a closed loop with single-sided mirrors:
        //   (0,3) "/"  (Identity)    reflects +Y → +X
        //   (3,3) "\"  (ROT_Z_270)   reflects +X → -Y
        //   (3,0) "/"  (ROT_Z_180)   reflects -Y → -X
        //   -- laser source is at (0,0), so -X beam hits it, stopping the loop.
        let _m1 = world.spawn(BlockKind::Mirror, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);
        let m2 = world.spawn(BlockKind::Mirror, IVec3::new(3, 3, 0), vec![IVec3::ZERO]);
        let m3 = world.spawn(BlockKind::Mirror, IVec3::new(3, 0, 0), vec![IVec3::ZERO]);

        world.body_mut(m2).unwrap().orientation = CubeRot::ROT_Z_270;
        world.body_mut(m3).unwrap().orientation = CubeRot::ROT_Z_180;
        world.sync_grid();

        // Should terminate without infinite loop.
        let segments = cast_all_lasers(&world);
        assert!(segments.len() >= 3);
        assert!(segments.len() <= 5); // bounded
    }

    #[test]
    fn reflected_mirror_transforms_reflection_angles() {
        let mut world = World::new();
        // Laser pointing +Y from (0, 0).
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);

        // Mirror at (0, 3) reflected across X axis (x ↦ -x).
        // Default "/" mirror becomes "\" mirror facing South-East.
        let mid = world.spawn(BlockKind::Mirror, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);
        world.body_mut(mid).unwrap().orientation = CubeRot::REFLECT_X;
        world.spawn(BlockKind::Wall, IVec3::new(-5, 3, 0), vec![IVec3::ZERO]);
        world.sync_grid();

        let segments = cast_all_lasers(&world);
        assert_eq!(segments.len(), 2);

        // Reflected segment must travel along -X (West towards wall at -5, 3, 0).
        let reflected = segments.iter().find(|s| s.direction == IVec3::new(-1, 0, 0));
        assert!(reflected.is_some(), "expected beam reflected to -X for REFLECT_X mirror");
        assert_eq!(reflected.unwrap().hit.as_ref().unwrap().cell, IVec3::new(-5, 3, 0));
    }
}
