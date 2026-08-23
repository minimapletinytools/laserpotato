//! Core simulation data model: the spatial grid and the rigid, possibly
//! multi-cell "bodies" that occupy it.
//!
//! This module has no dependency on Bevy. The goal is a small, cheaply
//! cloneable/hashable state (`World`'s body list) and pure step functions on
//! top of it, so the exact same state can later run headless — undo/redo, a
//! BFS/IDA* solver for level validation — as well as drive rendering.

use std::collections::HashMap;

use glam::IVec3;

use crate::block_types::BlockKind;

/// Stable handle for a [`Body`]. Indices, not references, so state stays
/// `Copy`/`Hash`/serializable without borrow-checker fights.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BodyId(pub u32);

/// One of the 24 rotations of a cube, stored as an orthogonal integer matrix
/// (row `r` says where world-axis `r` gets its value from in local space).
/// A matrix instead of a lookup table means new orientations compose
/// naturally via matrix multiplication as a body physically rotates 90° at a
/// time, rather than needing a hand-enumerated 24-entry table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CubeRot {
    mat: [[i32; 3]; 3],
}

impl CubeRot {
    pub const IDENTITY: CubeRot = CubeRot {
        mat: [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
    };

    /// 90° about the local X axis.
    pub const ROT_X_90: CubeRot = CubeRot {
        mat: [[1, 0, 0], [0, 0, -1], [0, 1, 0]],
    };

    /// 90° about the local Y axis.
    pub const ROT_Y_90: CubeRot = CubeRot {
        mat: [[0, 0, 1], [0, 1, 0], [-1, 0, 0]],
    };

    /// 90° about the local Z axis.
    pub const ROT_Z_90: CubeRot = CubeRot {
        mat: [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
    };

    /// 180° about the local Z axis.
    pub const ROT_Z_180: CubeRot = CubeRot {
        mat: [[-1, 0, 0], [0, -1, 0], [0, 0, 1]],
    };

    /// 270° about the local Z axis (equivalently, −90°).
    pub const ROT_Z_270: CubeRot = CubeRot {
        mat: [[0, 1, 0], [-1, 0, 0], [0, 0, 1]],
    };

    /// Apply this rotation to a local-space offset, producing a world-space offset.
    pub fn apply(self, v: IVec3) -> IVec3 {
        let m = self.mat;
        IVec3::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        )
    }

    /// Compose rotations: applying `self.then(other)` is the same as applying
    /// `self` first, then `other`.
    pub fn then(self, other: CubeRot) -> CubeRot {
        let a = other.mat;
        let b = self.mat;
        let mut mat = [[0i32; 3]; 3];
        for r in 0..3 {
            for c in 0..3 {
                mat[r][c] = (0..3).map(|k| a[r][k] * b[k][c]).sum();
            }
        }
        CubeRot { mat }
    }

    /// Inverse rotation. Cheap because rotation matrices are orthogonal, so
    /// the inverse is just the transpose.
    pub fn inverse(self) -> CubeRot {
        let m = self.mat;
        CubeRot {
            mat: [
                [m[0][0], m[1][0], m[2][0]],
                [m[0][1], m[1][1], m[2][1]],
                [m[0][2], m[1][2], m[2][2]],
            ],
        }
    }

    /// Access the raw 3×3 rotation matrix (rows of column components).
    pub fn mat(&self) -> [[i32; 3]; 3] {
        self.mat
    }
}

impl Default for CubeRot {
    fn default() -> Self {
        CubeRot::IDENTITY
    }
}

/// Identifies *what kind* of tag is present; any associated state lives in
/// [`TagValue`]. This is the fixed vocabulary of mechanics — add a variant
/// here whenever a new one gets designed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TagKind {
    Fixed,
    Pushable,
    Sticky,
    Fragile,
    Burnt,
    Charge,
}

/// A tag's associated state. `Unit` for plain marker tags (e.g. `Fixed`);
/// `Level` for tags that carry a magnitude, e.g. `Burnt` at level 2 meaning
/// "hit by 2 units of burn power so far".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TagValue {
    Unit,
    Level(i32),
}

/// The set of tags on a single [`Body`].
///
/// Backed by a `Vec` kept sorted by [`TagKind`], rather than a `HashMap` —
/// `HashMap` doesn't implement `Hash`, and keeping `TagSet` (and therefore
/// `Body`) hashable matters later for solver state deduplication. Tag counts
/// per body are tiny, so sorted-vec lookup costs nothing in practice.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TagSet {
    tags: Vec<(TagKind, TagValue)>,
}

impl TagSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, kind: TagKind) -> bool {
        self.tags.binary_search_by_key(&kind, |(k, _)| *k).is_ok()
    }

    pub fn get(&self, kind: TagKind) -> Option<TagValue> {
        self.tags
            .binary_search_by_key(&kind, |(k, _)| *k)
            .ok()
            .map(|i| self.tags[i].1)
    }

    /// Numeric level for a tag, or 0 if the tag is absent or isn't a `Level`.
    pub fn level(&self, kind: TagKind) -> i32 {
        match self.get(kind) {
            Some(TagValue::Level(n)) => n,
            _ => 0,
        }
    }

    pub fn set(&mut self, kind: TagKind, value: TagValue) {
        match self.tags.binary_search_by_key(&kind, |(k, _)| *k) {
            Ok(i) => self.tags[i].1 = value,
            Err(i) => self.tags.insert(i, (kind, value)),
        }
    }

    /// Convenience for stateful tags like `Burnt`: bump the level by `delta`,
    /// treating an absent tag as level 0 (so e.g. `add_level(Burnt, 1)` takes
    /// an untouched block straight to "burnt 1").
    pub fn add_level(&mut self, kind: TagKind, delta: i32) {
        let n = self.level(kind) + delta;
        self.set(kind, TagValue::Level(n));
    }

    pub fn remove(&mut self, kind: TagKind) -> Option<TagValue> {
        match self.tags.binary_search_by_key(&kind, |(k, _)| *k) {
            Ok(i) => Some(self.tags.remove(i).1),
            Err(_) => None,
        }
    }
}

/// A rigid, possibly multi-cell object in the grid.
///
/// `shape` is a list of occupied offsets in local, unrotated space; world-space
/// occupied cells are `anchor + orientation.apply(offset)` for each offset.
/// `anchor` is just a reference cell, not necessarily the body's geometric
/// center — rotating a multi-cell body about its true centroid (needed if we
/// end up wanting Sausage-Roll-style edge-over-edge tumbling for even-length
/// bodies) would need a half-unit-scaled lattice. Deferred until the
/// rotate-in-place-vs-roll question is settled.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Body {
    pub id: BodyId,
    pub kind: BlockKind,
    pub anchor: IVec3,
    pub orientation: CubeRot,
    pub shape: Vec<IVec3>,
    pub tags: TagSet,
}

impl Body {
    pub fn new(id: BodyId, kind: BlockKind, anchor: IVec3, shape: Vec<IVec3>) -> Self {
        Self {
            id,
            kind,
            anchor,
            orientation: CubeRot::IDENTITY,
            shape,
            tags: TagSet::new(),
        }
    }

    /// World-space cells this body currently occupies.
    pub fn world_cells(&self) -> Vec<IVec3> {
        self.shape
            .iter()
            .map(|&offset| self.anchor + self.orientation.apply(offset))
            .collect()
    }

    /// Whether this specific body instance can be pushed.
    /// TagKind::Fixed explicitly disables pushing.
    /// TagKind::Pushable explicitly enables pushing.
    /// Otherwise falls back to the default pushability of its BlockKind.
    pub fn is_pushable(&self) -> bool {
        if self.tags.has(TagKind::Fixed) {
            return false;
        }
        if self.tags.has(TagKind::Pushable) {
            return true;
        }
        self.kind.is_pushable()
    }
}

/// Spatial occupancy index over [`Body`] cells.
///
/// This is a *derived* structure, not source-of-truth state — it can always
/// be rebuilt from a body list — so it's kept separate from (and out of the
/// hash/equality of) the canonical body list in [`World`].
#[derive(Clone, Debug, Default)]
pub struct Grid {
    occupancy: HashMap<IVec3, BodyId>,
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn occupant_at(&self, pos: IVec3) -> Option<BodyId> {
        self.occupancy.get(&pos).copied()
    }

    pub fn is_occupied(&self, pos: IVec3) -> bool {
        self.occupancy.contains_key(&pos)
    }

    /// Rebuild the occupancy index from scratch. Cheap enough at puzzle-game
    /// scale to just call this after any body moves, rather than maintaining
    /// incremental inserts/removes by hand.
    pub fn rebuild(&mut self, bodies: &[Body]) {
        self.occupancy.clear();
        for body in bodies {
            for cell in body.world_cells() {
                self.occupancy.insert(cell, body.id);
            }
        }
    }
}

/// The full simulation state: the canonical list of bodies plus a derived
/// spatial index over them.
///
/// Note `World` itself doesn't derive `Hash`/`Eq` — `Grid` holds a `HashMap`
/// and can't. For solver/undo state comparisons later, hash or compare
/// `world.bodies()` (a `&[Body]`), which is the actual source of truth; the
/// grid is just a cache that can always be rebuilt from it.
#[derive(Clone, Debug, Default)]
pub struct World {
    bodies: Vec<Body>,
    grid: Grid,
    next_id: u32,
    player_id: Option<BodyId>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new body of the given kind at `anchor` with the given `shape`.
    /// If `kind` is [`BlockKind::Player`], the world remembers it for fast
    /// lookup via [`player_id()`](Self::player_id).
    pub fn spawn(&mut self, kind: BlockKind, anchor: IVec3, shape: Vec<IVec3>) -> BodyId {
        let id = BodyId(self.next_id);
        self.next_id += 1;
        if kind == BlockKind::Player {
            self.player_id = Some(id);
        }
        self.bodies.push(Body::new(id, kind, anchor, shape));
        self.grid.rebuild(&self.bodies);
        id
    }

    pub fn body(&self, id: BodyId) -> Option<&Body> {
        self.bodies.iter().find(|b| b.id == id)
    }

    pub fn body_mut(&mut self, id: BodyId) -> Option<&mut Body> {
        self.bodies.iter_mut().find(|b| b.id == id)
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// The player body's id, if one has been spawned.
    pub fn player_id(&self) -> Option<BodyId> {
        self.player_id
    }

    /// Look up the body occupying `pos`, if any.
    pub fn body_at(&self, pos: IVec3) -> Option<&Body> {
        let id = self.grid.occupant_at(pos)?;
        self.body(id)
    }

    /// Call after directly mutating body positions/orientations to keep the
    /// spatial index in sync.
    pub fn sync_grid(&mut self) {
        self.grid.rebuild(&self.bodies);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_types::BlockKind;

    #[test]
    fn identity_rotation_is_a_no_op() {
        let v = IVec3::new(1, 2, 3);
        assert_eq!(CubeRot::IDENTITY.apply(v), v);
    }

    #[test]
    fn four_quarter_turns_return_to_identity() {
        let v = IVec3::new(1, 2, 3);
        let mut rot = CubeRot::IDENTITY;
        for _ in 0..4 {
            rot = rot.then(CubeRot::ROT_Z_90);
        }
        assert_eq!(rot.apply(v), v);
    }

    #[test]
    fn inverse_undoes_rotation() {
        let v = IVec3::new(1, 2, 3);
        let rot = CubeRot::ROT_X_90.then(CubeRot::ROT_Y_90);
        let round_trip = rot.inverse().apply(rot.apply(v));
        assert_eq!(round_trip, v);
    }

    #[test]
    fn multi_cell_body_world_cells_follow_orientation() {
        // a 2-cell body lying along local X, anchored at (0,0,0)
        let mut body = Body::new(
            BodyId(0),
            BlockKind::Pushable,
            IVec3::new(0, 0, 0),
            vec![IVec3::new(0, 0, 0), IVec3::new(1, 0, 0)],
        );
        assert_eq!(
            body.world_cells(),
            vec![IVec3::new(0, 0, 0), IVec3::new(1, 0, 0)]
        );

        // after a 90-degree turn about Z, the body should now lie along Y
        body.orientation = CubeRot::ROT_Z_90;
        assert_eq!(
            body.world_cells(),
            vec![IVec3::new(0, 0, 0), IVec3::new(0, 1, 0)]
        );
    }

    #[test]
    fn tag_levels_stack_like_burnt_n() {
        let mut tags = TagSet::new();
        assert_eq!(tags.level(TagKind::Burnt), 0);

        tags.add_level(TagKind::Burnt, 1);
        assert_eq!(tags.level(TagKind::Burnt), 1);

        tags.add_level(TagKind::Burnt, 1);
        assert_eq!(tags.level(TagKind::Burnt), 2);

        assert!(!tags.has(TagKind::Fixed));
        tags.set(TagKind::Fixed, TagValue::Unit);
        assert!(tags.has(TagKind::Fixed));

        tags.remove(TagKind::Burnt);
        assert_eq!(tags.level(TagKind::Burnt), 0);
    }

    #[test]
    fn grid_tracks_all_cells_of_a_multi_cell_body() {
        let mut world = World::new();
        let id = world.spawn(
            BlockKind::Pushable,
            IVec3::new(0, 0, 0),
            vec![IVec3::new(0, 0, 0), IVec3::new(1, 0, 0)],
        );
        assert_eq!(world.grid().occupant_at(IVec3::new(0, 0, 0)), Some(id));
        assert_eq!(world.grid().occupant_at(IVec3::new(1, 0, 0)), Some(id));
        assert_eq!(world.grid().occupant_at(IVec3::new(2, 0, 0)), None);
    }

    #[test]
    fn player_id_tracked_on_spawn() {
        let mut world = World::new();
        assert_eq!(world.player_id(), None);
        let pid = world.spawn(BlockKind::Player, IVec3::ZERO, vec![IVec3::ZERO]);
        assert_eq!(world.player_id(), Some(pid));
    }

    #[test]
    fn body_at_returns_occupant() {
        let mut world = World::new();
        world.spawn(BlockKind::Wall, IVec3::new(3, 0, 0), vec![IVec3::ZERO]);
        assert!(world.body_at(IVec3::new(3, 0, 0)).is_some());
        assert!(world.body_at(IVec3::new(4, 0, 0)).is_none());
    }
}
