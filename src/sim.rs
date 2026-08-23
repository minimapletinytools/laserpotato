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

use serde::{Deserialize, Serialize};

/// One of the 24 rotations of a cube, stored as an orthogonal integer matrix
/// (row `r` says where world-axis `r` gets its value from in local space).
/// A matrix instead of a lookup table means new orientations compose
/// naturally via matrix multiplication as a body physically rotates 90° at a
/// time, rather than needing a hand-enumerated 24-entry table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CubeRot {
    pub mat: [[i32; 3]; 3],
}

impl CubeRot {
    pub const fn from_matrix(mat: [[i32; 3]; 3]) -> Self {
        Self { mat }
    }

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

    /// Reflection across the YZ plane: x ↦ -x.
    pub const REFLECT_X: CubeRot = CubeRot {
        mat: [[-1, 0, 0], [0, 1, 0], [0, 0, 1]],
    };

    /// Reflection across the XZ plane: y ↦ -y.
    pub const REFLECT_Y: CubeRot = CubeRot {
        mat: [[1, 0, 0], [0, -1, 0], [0, 0, 1]],
    };

    /// Reflection across the XY plane: z ↦ -z.
    pub const REFLECT_Z: CubeRot = CubeRot {
        mat: [[1, 0, 0], [0, 1, 0], [0, 0, -1]],
    };

    /// Reflection across the diagonal x=y plane: (x, y, z) ↦ (y, x, z).
    pub const REFLECT_XY: CubeRot = CubeRot {
        mat: [[0, 1, 0], [1, 0, 0], [0, 0, 1]],
    };

    /// Full point inversion: x ↦ -x, y ↦ -y, z ↦ -z.
    pub const INVERSION: CubeRot = CubeRot {
        mat: [[-1, 0, 0], [0, -1, 0], [0, 0, -1]],
    };

    /// Apply this rotation/reflection to a local-space offset, producing a world-space offset.
    pub fn apply(self, v: IVec3) -> IVec3 {
        let m = self.mat;
        IVec3::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        )
    }

    /// Compose transformations: applying `self.then(other)` is the same as applying
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

    /// Inverse transformation. Cheap because all signed permutation matrices in Oh
    /// are orthogonal, so the inverse is just the matrix transpose.
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

    /// Determinant of the transformation matrix (+1 for proper rotations, -1 for reflections).
    pub fn det(&self) -> i32 {
        let m = self.mat;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// Returns true if this transformation includes a spatial reflection / improper rotation (det == -1).
    pub fn is_reflection(&self) -> bool {
        self.det() < 0
    }

    /// Returns true if this transformation is a pure proper rotation (det == +1).
    pub fn is_proper_rotation(&self) -> bool {
        self.det() > 0
    }

    /// Flip / reflect across the local X axis (x ↦ -x).
    pub fn reflect_x(self) -> Self {
        self.then(Self::REFLECT_X)
    }

    /// Flip / reflect across the local Y axis (y ↦ -y).
    pub fn reflect_y(self) -> Self {
        self.then(Self::REFLECT_Y)
    }

    /// Flip / reflect across the local Z axis (z ↦ -z).
    pub fn reflect_z(self) -> Self {
        self.then(Self::REFLECT_Z)
    }

    /// Return the 2D rotation around the Z axis that maps local +Y (forward) to the given 2D direction.
    pub fn from_facing_2d(dir: IVec3) -> Self {
        match (dir.x, dir.y) {
            (0, 1) => Self::IDENTITY,     // North (+Y)
            (1, 0) => Self::ROT_Z_270,    // East (+X)
            (0, -1) => Self::ROT_Z_180,   // South (-Y)
            (-1, 0) => Self::ROT_Z_90,    // West (-X)
            _ => Self::IDENTITY,
        }
    }

    /// Rotate 90° clockwise around the local Z axis.
    pub fn rotate_z_cw(self) -> Self {
        self.then(Self::ROT_Z_270)
    }

    /// Rotate 90° counter-clockwise around the local Z axis.
    pub fn rotate_z_ccw(self) -> Self {
        self.then(Self::ROT_Z_90)
    }

    /// Enumerate all 48 distinct symmetry operations of the full octahedral group Oh.
    /// Consists of 24 proper rotations (det = +1) and 24 reflections/improper rotations (det = -1).
    pub fn all_48() -> Vec<CubeRot> {
        let mut group = std::collections::HashSet::new();
        // Base generators for Oh: 90° rotations along X and Y, plus reflection across X.
        let generators = [
            Self::ROT_X_90,
            Self::ROT_Y_90,
            Self::REFLECT_X,
        ];

        let mut queue = vec![Self::IDENTITY];
        group.insert(Self::IDENTITY);

        while let Some(current) = queue.pop() {
            for &g in &generators {
                let next = current.then(g);
                if group.insert(next) {
                    queue.push(next);
                }
            }
        }

        group.into_iter().collect()
    }

    /// Return all 48 unique symmetry operations of Oh, deterministically sorted with
    /// `CubeRot::IDENTITY` at index 0, followed by proper rotations (det = +1), followed by
    /// reflections/improper rotations (det = -1).
    pub fn all_48_sorted() -> Vec<Self> {
        let mut list = Self::all_48();
        list.sort_by_key(|r| {
            (
                if *r == Self::IDENTITY { 0 } else { 1 },
                -r.det(), // det = +1 before det = -1
                r.mat[0],
                r.mat[1],
                r.mat[2],
            )
        });
        list
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

    /// Return the resolved behavioral properties for this body instance,
    /// taking into account tags (such as TagKind::Fixed).
    pub fn properties(&self) -> crate::block_types::BlockProperties {
        let mut props = self.kind.default_properties();
        if self.tags.has(TagKind::Fixed) || matches!(self.kind, BlockKind::Wall | BlockKind::Goal) {
            props.is_pushable = false;
        } else if self.tags.has(TagKind::Pushable) {
            props.is_pushable = true;
        }
        props
    }

    /// Whether this specific body instance can be pushed.
    pub fn is_pushable(&self) -> bool {
        self.properties().is_pushable
    }

    /// Whether this specific body is fixed/stationary.
    pub fn is_fixed(&self) -> bool {
        self.tags.has(TagKind::Fixed) || matches!(self.kind, BlockKind::Wall | BlockKind::Goal)
    }

    /// Computes the canonical orientation representing the equivalence class of this body's
    /// physical properties under the 48 symmetry operations of Oh.
    ///
    /// Two orientations M1, M2 that produce identical laser reflections on all 6 faces,
    /// identical laser emission direction, and identical occupied voxel cells will map to the
    /// exact same canonical `CubeRot`.
    pub fn canonical_orientation(&self) -> CubeRot {
        let props = self.properties();
        let all_symmetries = CubeRot::all_48_sorted();
        for candidate in all_symmetries {
            if self.is_physically_equivalent_to(&candidate, &props) {
                return candidate;
            }
        }
        self.orientation
    }

    fn is_physically_equivalent_to(
        &self,
        candidate: &CubeRot,
        props: &crate::block_types::BlockProperties,
    ) -> bool {
        // 1. World-space occupied cells must match:
        for &offset in &self.shape {
            if self.orientation.apply(offset) != candidate.apply(offset) {
                return false;
            }
        }

        // 2. If player-controlled, movement facing direction (+Y) must match:
        if props.is_player_controlled && self.orientation.apply(IVec3::Y) != candidate.apply(IVec3::Y) {
            return false;
        }

        // 3. If laser emitter, emitted laser direction must match in world space:
        if let Some(emit_local) = props.emits_laser_towards {
            if self.orientation.apply(emit_local) != candidate.apply(emit_local) {
                return false;
            }
        }

        // 4. Laser reflection response on all 6 cardinal incoming directions must match:
        let directions = [
            IVec3::X,
            IVec3::NEG_X,
            IVec3::Y,
            IVec3::NEG_Y,
            IVec3::Z,
            IVec3::NEG_Z,
        ];
        for &d in &directions {
            if props.reflect_laser(d, &self.orientation) != props.reflect_laser(d, candidate) {
                return false;
            }
        }

        true
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

    /// Despawn and remove a body by ID from the simulation.
    pub fn despawn(&mut self, id: BodyId) {
        if self.player_id == Some(id) {
            self.player_id = None;
        }
        self.bodies.retain(|b| b.id != id);
        self.grid.rebuild(&self.bodies);
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

    #[test]
    fn full_octahedral_group_has_48_unique_symmetries() {
        let symmetries = CubeRot::all_48();
        assert_eq!(symmetries.len(), 48);

        let mut proper_count = 0;
        let mut reflection_count = 0;

        for sym in &symmetries {
            let det = sym.det();
            assert!(det == 1 || det == -1, "Determinant must be ±1, got {}", det);
            if det == 1 {
                proper_count += 1;
                assert!(sym.is_proper_rotation());
                assert!(!sym.is_reflection());
            } else {
                reflection_count += 1;
                assert!(sym.is_reflection());
                assert!(!sym.is_proper_rotation());
            }

            // Verify orthogonality: M * M^T = I
            let inv = sym.inverse();
            let identity_check = sym.then(inv);
            assert_eq!(identity_check, CubeRot::IDENTITY);
        }

        assert_eq!(proper_count, 24, "Expected exactly 24 proper rotations");
        assert_eq!(reflection_count, 24, "Expected exactly 24 reflections/improper rotations");
    }

    #[test]
    fn reflection_reverses_coordinates_along_axis() {
        let v = IVec3::new(2, 3, 4);
        assert_eq!(CubeRot::REFLECT_X.apply(v), IVec3::new(-2, 3, 4));
        assert_eq!(CubeRot::REFLECT_Y.apply(v), IVec3::new(2, -3, 4));
        assert_eq!(CubeRot::REFLECT_Z.apply(v), IVec3::new(2, 3, -4));
        assert_eq!(CubeRot::INVERSION.apply(v), IVec3::new(-2, -3, -4));
    }

    #[test]
    fn canonical_orientation_equivalence_reduction() {
        let all_48 = CubeRot::all_48();

        // 1. For isotropic 1x1x1 Wall: all 48 orientations reduce to IDENTITY (1 class).
        let mut wall = Body::new(BodyId(1), BlockKind::Wall, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut wall_canonical_set = std::collections::HashSet::new();
        for &rot in &all_48 {
            wall.orientation = rot;
            let canonical = wall.canonical_orientation();
            assert_eq!(canonical, CubeRot::IDENTITY, "Wall orientation should reduce to IDENTITY");
            wall_canonical_set.insert(canonical);
        }
        assert_eq!(wall_canonical_set.len(), 1);

        // 2. For isotropic Pushable Crate: all 48 orientations reduce to IDENTITY (1 class).
        let mut crate_body = Body::new(BodyId(2), BlockKind::Pushable, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut crate_canonical_set = std::collections::HashSet::new();
        for &rot in &all_48 {
            crate_body.orientation = rot;
            let canonical = crate_body.canonical_orientation();
            assert_eq!(canonical, CubeRot::IDENTITY, "Pushable crate orientation should reduce to IDENTITY");
            crate_canonical_set.insert(canonical);
        }
        assert_eq!(crate_canonical_set.len(), 1);

        // 3. For LaserSource: 48 orientations reduce to exactly 6 canonical emission directions.
        let mut laser = Body::new(BodyId(3), BlockKind::LaserSource, IVec3::ZERO, vec![IVec3::ZERO]);
        let mut laser_canonical_set = std::collections::HashSet::new();
        for &rot in &all_48 {
            laser.orientation = rot;
            laser_canonical_set.insert(laser.canonical_orientation());
        }
        assert_eq!(laser_canonical_set.len(), 6, "Laser source should have exactly 6 canonical direction classes");
    }
}
