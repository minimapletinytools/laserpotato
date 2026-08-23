//! Laser ray casting and mirror reflection.
//!
//! No Bevy dependency. Given a [`World`](crate::sim::World), this module casts
//! rays from every [`LaserSource`](crate::block_types::BlockKind::LaserSource)
//! body, reflects off mirrors, and records what each beam hits.

use std::collections::HashSet;

use glam::IVec3;

use crate::block_types::BlockKind;
use crate::sim::{BodyId, CubeRot, World};

/// Maximum number of cells a single ray will traverse before giving up.
const MAX_RAY_LENGTH: usize = 100;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single laser beam segment — from a source (or reflecting mirror) to the
/// point where it terminates (hit a block, left the max range, or looped).
#[derive(Clone, Debug)]
pub struct LaserSegment {
    /// The body that emitted this segment (a `LaserSource` or reflecting `Mirror`).
    pub source_id: BodyId,
    /// Cell the ray originates from (one cell past the source body).
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
/// Mirrors generate secondary segments that are also included. Closed mirror
/// loops are detected via a visited set of `(cell, direction)` pairs.
pub fn cast_all_lasers(world: &World) -> Vec<LaserSegment> {
    let mut segments = Vec::new();
    // Track (origin, direction) pairs to detect mirror loops.
    let mut visited: HashSet<(IVec3, IVec3)> = HashSet::new();

    // Seed the work queue with every LaserSource.
    let mut queue: Vec<(BodyId, IVec3, IVec3)> = world
        .bodies()
        .iter()
        .filter(|b| b.kind == BlockKind::LaserSource)
        .map(|b| {
            // "Forward" in local space is +Y; transform to world space.
            let forward = b.orientation.apply(IVec3::new(0, 1, 0));
            (b.id, b.anchor, forward)
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
                // If it's a mirror, enqueue the reflected beam.
                if body.kind == BlockKind::Mirror {
                    if let Some(reflected_dir) = reflect_mirror(direction, &body.orientation) {
                        queue.push((body.id, body.anchor, reflected_dir));
                    }
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
// Mirror reflection
// ---------------------------------------------------------------------------

/// Compute the reflected direction when a laser hits a mirror.
///
/// In local space the default mirror is a 45° "/" reflector that swaps the
/// X and Y components of the incoming direction. Rotating the mirror body
/// changes which world-space axes get swapped.
///
/// Returns `None` if the mirror cannot reflect from this angle (e.g. a beam
/// arriving along the mirror's local Z axis).
fn reflect_mirror(incoming: IVec3, mirror_orientation: &CubeRot) -> Option<IVec3> {
    // Transform the incoming direction into the mirror's local frame.
    let local_dir = mirror_orientation.inverse().apply(incoming);

    // "/" reflection in local space: swap X ↔ Y.
    let reflected_local = match (local_dir.x, local_dir.y, local_dir.z) {
        (x, 0, 0) => IVec3::new(0, x, 0),
        (0, y, 0) => IVec3::new(y, 0, 0),
        _ => return None, // Z-axis or diagonal — no clean reflection
    };

    // Transform back to world space.
    Some(mirror_orientation.apply(reflected_local))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;
    use crate::sim::World;

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
        assert_eq!(segments[0].hit.as_ref().unwrap().cell, IVec3::new(0, 5, 0));
    }

    #[test]
    fn laser_into_void_caps_at_max_length() {
        let mut world = World::new();
        world.spawn(BlockKind::LaserSource, IVec3::ZERO, vec![IVec3::ZERO]);

        let segments = cast_all_lasers(&world);
        assert_eq!(segments.len(), 1);
        assert!(segments[0].hit.is_none());
        assert_eq!(segments[0].cells.len(), MAX_RAY_LENGTH);
    }

    #[test]
    fn mirror_reflects_beam() {
        let mut world = World::new();
        // Laser at origin pointing +Y.
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        // "/" mirror at (0,3,0) — reflects +Y → +X.
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
    fn rotated_mirror_reflects_differently() {
        let mut world = World::new();
        // Laser at origin pointing +Y.
        world.spawn(BlockKind::LaserSource, IVec3::ZERO, vec![IVec3::ZERO]);
        // Mirror at (0,3,0) rotated 90° about Z — should reflect +Y → -X.
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
        // Laser pointing +Y.
        world.spawn(BlockKind::LaserSource, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);

        // Square of 4 mirrors forming a closed loop:
        //   (0,3) "/"  reflects +Y → +X
        //   (3,3) "\"  reflects +X → -Y  (ROT_Z_90)
        //   (3,0) "/"  reflects -Y → -X
        //   -- laser source is at (0,0), so -X beam hits it, stopping the loop.
        let _m1 = world.spawn(BlockKind::Mirror, IVec3::new(0, 3, 0), vec![IVec3::ZERO]);
        let m2 = world.spawn(BlockKind::Mirror, IVec3::new(3, 3, 0), vec![IVec3::ZERO]);
        let _m3 = world.spawn(BlockKind::Mirror, IVec3::new(3, 0, 0), vec![IVec3::ZERO]);

        // m1 is "/" (identity) — +Y → +X ✓
        // m2 needs to be "\" — +X → -Y: use ROT_Z_90
        world.body_mut(m2).unwrap().orientation = CubeRot::ROT_Z_90;
        // m3 is "/" (identity) — -Y → -X ✓
        world.sync_grid();

        // Should terminate without infinite loop.
        let segments = cast_all_lasers(&world);
        // At least 3 segments (source→m1, m1→m2, m2→m3) + possibly m3→source.
        assert!(segments.len() >= 3);
        assert!(segments.len() <= 5); // bounded
    }
}
