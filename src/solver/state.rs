//! Canonical puzzle state representation for hashing, state equivalence,
//! and loop/cycle detection.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use glam::IVec3;

use crate::sim::{Body, CubeRot, World};

/// Compact representation of a single dynamic body's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompactBodyState {
    pub kind: BlockKind,
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub anchor_z: i32,
    pub orientation: CubeRot,
    pub combined_group: Option<u32>,
    pub tags_hash: u64,
}

impl CompactBodyState {
    pub fn from_body(body: &Body) -> Self {
        let mut hasher = DefaultHasher::new();
        body.tags.hash(&mut hasher);
        let tags_hash = hasher.finish();

        Self {
            kind: body.kind,
            anchor_x: body.anchor.x,
            anchor_y: body.anchor.y,
            anchor_z: body.anchor.z,
            orientation: body.canonical_orientation(),
            combined_group: body.combined_group,
            tags_hash,
        }
    }

    pub fn anchor(&self) -> IVec3 {
        IVec3::new(self.anchor_x, self.anchor_y, self.anchor_z)
    }
}

/// Canonical state of the puzzle world for equivalence checking and cycle detection.
///
/// Immutable walls are omitted so hashing is minimal and ultra-fast.
/// Moveable blocks of identical kind/tags are sorted canonically so permutations
/// of indistinguishable blocks map to the exact same hash key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonicalState {
    /// The player's exact state (position and orientation).
    pub player: Option<CompactBodyState>,
    /// Sorted list of all other dynamic / moveable bodies.
    pub dynamic_bodies: Vec<CompactBodyState>,
}

impl CanonicalState {
    /// Extract the canonical state from a [`World`].
    pub fn from_world(world: &World) -> Self {
        let mut player = None;
        let mut dynamic_bodies = Vec::new();

        for body in world.bodies() {
            if body.kind == BlockKind::Player {
                player = Some(CompactBodyState::from_body(body));
            } else if body.is_pushable() || !body.is_fixed() {
                // Include any body that can move or change state
                dynamic_bodies.push(CompactBodyState::from_body(body));
            }
        }

        // Sort dynamic bodies to ensure canonical order (e.g. interchangeable pushable crates)
        dynamic_bodies.sort();

        // Normalize combined_group IDs based on first appearance
        let mut next_norm_id = 0;
        let mut group_map = std::collections::HashMap::new();
        for b in &mut dynamic_bodies {
            if let Some(orig_id) = b.combined_group {
                let norm_id = *group_map.entry(orig_id).or_insert_with(|| {
                    let id = next_norm_id;
                    next_norm_id += 1;
                    id
                });
                b.combined_group = Some(norm_id);
            }
        }

        // Re-sort after normalization in case IDs changed order
        dynamic_bodies.sort();

        Self {
            player,
            dynamic_bodies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;
    use crate::sim::World;

    #[test]
    fn identical_worlds_produce_identical_canonical_states() {
        let mut w1 = World::new();
        w1.spawn(BlockKind::Player, IVec3::new(1, 2, 0), vec![IVec3::ZERO]);
        w1.spawn(BlockKind::Pushable, IVec3::new(3, 4, 0), vec![IVec3::ZERO]);

        let mut w2 = World::new();
        w2.spawn(BlockKind::Player, IVec3::new(1, 2, 0), vec![IVec3::ZERO]);
        w2.spawn(BlockKind::Pushable, IVec3::new(3, 4, 0), vec![IVec3::ZERO]);

        assert_eq!(
            CanonicalState::from_world(&w1),
            CanonicalState::from_world(&w2)
        );
    }

    #[test]
    fn swapping_identical_crates_produces_identical_canonical_state() {
        let mut w1 = World::new();
        w1.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        w1.spawn(BlockKind::Pushable, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        w1.spawn(BlockKind::Pushable, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);

        let mut w2 = World::new();
        w2.spawn(BlockKind::Player, IVec3::new(0, 0, 0), vec![IVec3::ZERO]);
        // Spawn in reverse order
        w2.spawn(BlockKind::Pushable, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w2.spawn(BlockKind::Pushable, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);

        assert_eq!(
            CanonicalState::from_world(&w1),
            CanonicalState::from_world(&w2)
        );
    }

    #[test]
    fn different_player_facing_produces_different_canonical_state() {
        let mut w1 = World::new();
        let p1 = w1.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        w1.body_mut(p1).unwrap().orientation = CubeRot::IDENTITY;

        let mut w2 = World::new();
        let p2 = w2.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        w2.body_mut(p2).unwrap().orientation = CubeRot::ROT_Z_90;

        assert_ne!(
            CanonicalState::from_world(&w1),
            CanonicalState::from_world(&w2)
        );
    }

    #[test]
    fn rotating_or_flipping_isotropic_crate_produces_identical_canonical_state() {
        let mut w1 = World::new();
        w1.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let c1 = w1.spawn(BlockKind::Pushable, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w1.body_mut(c1).unwrap().orientation = CubeRot::IDENTITY;

        let mut w2 = World::new();
        w2.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        let c2 = w2.spawn(BlockKind::Pushable, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w2.body_mut(c2).unwrap().orientation = CubeRot::REFLECT_X.then(CubeRot::ROT_Z_270);

        assert_eq!(
            CanonicalState::from_world(&w1),
            CanonicalState::from_world(&w2),
            "Flipping/rotating an isotropic pushable crate must produce the identical canonical state"
        );
    }

    #[test]
    fn swapping_glass_blocks_is_interchangeable() {
        let mut w1 = World::new();
        w1.spawn(BlockKind::Glass, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        w1.spawn(BlockKind::Glass, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        
        let mut w2 = World::new();
        w2.spawn(BlockKind::Glass, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w2.spawn(BlockKind::Glass, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        
        assert_eq!(CanonicalState::from_world(&w1), CanonicalState::from_world(&w2));
    }
    
    #[test]
    fn combined_group_not_interchangeable() {
        let mut w1 = World::new();
        let g1 = w1.spawn(BlockKind::Glass, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        let _g2 = w1.spawn(BlockKind::Glass, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w1.body_mut(g1).unwrap().combined_group = Some(1);
        
        let mut w2 = World::new();
        let _g3 = w2.spawn(BlockKind::Glass, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        let g4 = w2.spawn(BlockKind::Glass, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w2.body_mut(g4).unwrap().combined_group = Some(1);
        
        assert_ne!(CanonicalState::from_world(&w1), CanonicalState::from_world(&w2));
    }
    
    #[test]
    fn group_id_normalization() {
        let mut w1 = World::new();
        let g1 = w1.spawn(BlockKind::Glass, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        let g2 = w1.spawn(BlockKind::Glass, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w1.body_mut(g1).unwrap().combined_group = Some(10);
        w1.body_mut(g2).unwrap().combined_group = Some(10);
        
        let mut w2 = World::new();
        let g3 = w2.spawn(BlockKind::Glass, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        let g4 = w2.spawn(BlockKind::Glass, IVec3::new(2, 0, 0), vec![IVec3::ZERO]);
        w2.body_mut(g3).unwrap().combined_group = Some(20);
        w2.body_mut(g4).unwrap().combined_group = Some(20);
        
        assert_eq!(CanonicalState::from_world(&w1), CanonicalState::from_world(&w2));
    }
    
    #[test]
    fn floor_excluded() {
        let mut w1 = World::new();
        w1.spawn(BlockKind::Floor, IVec3::new(1, 0, 0), vec![IVec3::ZERO]);
        
        let w2 = World::new();
        
        assert_eq!(CanonicalState::from_world(&w1), CanonicalState::from_world(&w2));
    }
}
