//! Block type definitions and generic block / block-face properties.
//!
//! [`BlockKind`] captures the *type identity* of a block.
//! [`BlockProperties`] and [`FaceProperties`] define the generic behavioral
//! properties of blocks and their 6 individual faces (e.g. pushability,
//! reflectivity under rotation and spatial reflection).

use std::fmt;
use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::sim::CubeRot;

// ---------------------------------------------------------------------------
// Block Faces
// ---------------------------------------------------------------------------

/// The 6 principal orthogonal faces of a cubic grid block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlockFace {
    /// +X face (East / Right)
    PosX = 0,
    /// -X face (West / Left)
    NegX = 1,
    /// +Y face (North / Forward)
    PosY = 2,
    /// -Y face (South / Backward)
    NegY = 3,
    /// +Z face (Up / Top)
    PosZ = 4,
    /// -Z face (Down / Bottom)
    NegZ = 5,
}

impl BlockFace {
    pub const ALL: [BlockFace; 6] = [
        BlockFace::PosX,
        BlockFace::NegX,
        BlockFace::PosY,
        BlockFace::NegY,
        BlockFace::PosZ,
        BlockFace::NegZ,
    ];

    /// Normal unit vector pointing outward from this face.
    #[inline]
    pub fn normal(self) -> IVec3 {
        match self {
            BlockFace::PosX => IVec3::X,
            BlockFace::NegX => -IVec3::X,
            BlockFace::PosY => IVec3::Y,
            BlockFace::NegY => -IVec3::Y,
            BlockFace::PosZ => IVec3::Z,
            BlockFace::NegZ => -IVec3::Z,
        }
    }

    /// Identify a face from an outward normal unit vector.
    pub fn from_normal(v: IVec3) -> Option<Self> {
        match (v.x, v.y, v.z) {
            (1, 0, 0) => Some(BlockFace::PosX),
            (-1, 0, 0) => Some(BlockFace::NegX),
            (0, 1, 0) => Some(BlockFace::PosY),
            (0, -1, 0) => Some(BlockFace::NegY),
            (0, 0, 1) => Some(BlockFace::PosZ),
            (0, 0, -1) => Some(BlockFace::NegZ),
            _ => None,
        }
    }

    /// The local face struck by a ray traveling in direction `ray_dir`.
    ///
    /// For example, a ray traveling in the +X direction arrives from the -X
    /// side, striking the `NegX` face (whose outward normal points -X).
    #[inline]
    pub fn from_incoming_ray_dir(ray_dir: IVec3) -> Option<Self> {
        Self::from_normal(-ray_dir)
    }

    /// Transform this face by a 3D orthogonal rotation matrix.
    pub fn transform(self, rot: &CubeRot) -> Self {
        let n = rot.apply(self.normal());
        Self::from_normal(n).unwrap_or(self)
    }

    /// Reflect this face across a plane defined by a unit normal vector `plane_normal`.
    ///
    /// Formula: $n' = n - 2 (n \cdot \hat{p}) \hat{p}$
    pub fn reflect_across_plane(self, plane_normal: IVec3) -> Self {
        let n = self.normal();
        let dot = n.dot(plane_normal);
        let reflected_normal = n - 2 * dot * plane_normal;
        Self::from_normal(reflected_normal).unwrap_or(self)
    }
}

// ---------------------------------------------------------------------------
// Per-Face Properties
// ---------------------------------------------------------------------------

/// Behavioral properties specific to a single face of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaceProperties {
    /// If `Some(out_dir)`, an incoming laser beam hitting this face is
    /// reflected towards `out_dir` (expressed in the block's local frame).
    ///
    /// If `None`, this face is non-reflective and absorbs/blocks the beam.
    pub reflects_towards: Option<IVec3>,
}

impl Default for FaceProperties {
    fn default() -> Self {
        Self {
            reflects_towards: None,
        }
    }
}

impl FaceProperties {
    /// Non-reflective solid face.
    pub const fn none() -> Self {
        Self {
            reflects_towards: None,
        }
    }

    /// Reflective face that directs incoming rays towards `out_dir`.
    pub const fn reflects_to(out_dir: IVec3) -> Self {
        Self {
            reflects_towards: Some(out_dir),
        }
    }

    /// Transform face properties under a rotation.
    pub fn transform(&self, rot: &CubeRot) -> Self {
        Self {
            reflects_towards: self.reflects_towards.map(|d| rot.apply(d)),
        }
    }

    /// Reflect face properties across a plane normal.
    pub fn reflect_across_plane(&self, plane_normal: IVec3) -> Self {
        Self {
            reflects_towards: self.reflects_towards.map(|d| {
                let dot = d.dot(plane_normal);
                d - 2 * dot * plane_normal
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Per-Block Properties
// ---------------------------------------------------------------------------

/// Full behavioral specification of a block and its 6 individual faces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockProperties {
    /// Whether this block can be pushed when untagged.
    pub is_pushable: bool,
    /// Whether this block prevents other blocks from occupying its cells.
    pub is_solid: bool,
    /// Movement priority — lower values are processed first in turn resolution.
    pub movement_priority: u32,
    /// Properties for each of the 6 faces indexed by [`BlockFace`].
    pub faces: [FaceProperties; 6],
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self {
            is_pushable: false,
            is_solid: true,
            movement_priority: 100,
            faces: [FaceProperties::none(); 6],
        }
    }
}

impl BlockProperties {
    /// Get the properties of a specific block face.
    #[inline]
    pub fn face(&self, face: BlockFace) -> &FaceProperties {
        &self.faces[face as usize]
    }

    /// Get mutable reference to properties of a specific block face.
    #[inline]
    pub fn face_mut(&mut self, face: BlockFace) -> &mut FaceProperties {
        &mut self.faces[face as usize]
    }

    /// Set properties for a specific face.
    pub fn set_face(&mut self, face: BlockFace, prop: FaceProperties) -> &mut Self {
        self.faces[face as usize] = prop;
        self
    }

    /// Evaluate laser reflection given an incoming ray direction in world space
    /// and the block's current 3D orientation.
    ///
    /// 1. Converts incoming direction $\vec{d}_{world}$ to local frame: $\vec{d}_{local} = R^{-1}(\vec{d}_{world})$.
    /// 2. Identifies the local face struck by the ray: $face = BlockFace::from\_incoming\_ray\_dir(\vec{d}_{local})$.
    /// 3. If that face defines `reflects_towards: Some(out_local)`, converts back to world space: $\vec{d}_{out\_world} = R(out_{local})$.
    /// 4. If `None`, returns `None` (laser is stopped/absorbed by the face).
    pub fn reflect_laser(&self, incoming_world_dir: IVec3, orientation: &CubeRot) -> Option<IVec3> {
        let local_incoming = orientation.inverse().apply(incoming_world_dir);
        let struck_face = BlockFace::from_incoming_ray_dir(local_incoming)?;
        let face_prop = self.face(struck_face);
        let local_out = face_prop.reflects_towards?;
        Some(orientation.apply(local_out))
    }

    /// Transform all face properties and directions by a 3D rotation.
    pub fn transform(&self, rot: &CubeRot) -> Self {
        let mut new_faces = [FaceProperties::none(); 6];
        for face in BlockFace::ALL {
            let transformed_face = face.transform(rot);
            let prop = self.face(face).transform(rot);
            new_faces[transformed_face as usize] = prop;
        }
        Self {
            is_pushable: self.is_pushable,
            is_solid: self.is_solid,
            movement_priority: self.movement_priority,
            faces: new_faces,
        }
    }

    /// Reflect all block properties across a plane defined by `plane_normal`.
    pub fn reflect_across_plane(&self, plane_normal: IVec3) -> Self {
        let mut new_faces = [FaceProperties::none(); 6];
        for face in BlockFace::ALL {
            let reflected_face = face.reflect_across_plane(plane_normal);
            let prop = self.face(face).reflect_across_plane(plane_normal);
            new_faces[reflected_face as usize] = prop;
        }
        Self {
            is_pushable: self.is_pushable,
            is_solid: self.is_solid,
            movement_priority: self.movement_priority,
            faces: new_faces,
        }
    }
}

// ---------------------------------------------------------------------------
// Block Kinds (Vocabularies)
// ---------------------------------------------------------------------------

/// The fixed type identity of a block, determining its base properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlockKind {
    /// The player-controlled character.
    Player,
    /// Immovable wall / boundary.
    Wall,
    /// Standard pushable crate.
    Pushable,
    /// Single-sided 45° reflector prism.
    ///
    /// In local space:
    /// - Ray traveling +X enters `NegX` face $\rightarrow$ reflects to +Y (North).
    /// - Ray traveling +Y enters `NegY` face $\rightarrow$ reflects to +X (East).
    /// - Back faces (`PosX`, `PosY`, `PosZ`, `NegZ`) have NO reflection and stop the laser.
    Mirror,
    /// Emits a laser beam in its forward (+Y local) direction.
    LaserSource,
    /// Target goal pyramid. When struck by a laser beam, the puzzle level is completed.
    Goal,
}

impl BlockKind {
    /// Construct default behavioral properties for this block kind.
    pub fn default_properties(self) -> BlockProperties {
        let mut props = BlockProperties::default();
        match self {
            Self::Player => {
                props.is_pushable = true;
                props.movement_priority = 0;
            }
            Self::Wall => {
                props.is_pushable = false;
                props.movement_priority = 100;
            }
            Self::Pushable => {
                props.is_pushable = true;
                props.movement_priority = 100;
            }
            Self::Goal => {
                props.is_pushable = false;
                props.movement_priority = 100;
            }
            Self::LaserSource => {
                props.is_pushable = true;
                props.movement_priority = 100;
            }
            Self::Mirror => {
                props.is_pushable = true;
                props.movement_priority = 100;

                // Single-sided reflective hypotenuse:
                // Incoming +X (enters NegX face) -> reflects +Y (North)
                props.set_face(BlockFace::NegX, FaceProperties::reflects_to(IVec3::new(0, 1, 0)));
                // Incoming +Y (enters NegY face) -> reflects +X (East)
                props.set_face(BlockFace::NegY, FaceProperties::reflects_to(IVec3::new(1, 0, 0)));
                // Back walls (PosX, PosY) and caps (PosZ, NegZ) remain FaceProperties::none() -> solid absorption.
            }
        }
        props
    }

    /// Whether this block kind is inherently pushable when untagged.
    #[inline]
    pub fn is_pushable(self) -> bool {
        self.default_properties().is_pushable
    }

    /// Whether this block prevents other blocks from entering its cells.
    #[inline]
    pub fn is_solid(self) -> bool {
        self.default_properties().is_solid
    }

    /// Movement priority — lower values are processed first in the
    /// movement queue. The player always moves first.
    #[inline]
    pub fn movement_priority(self) -> u32 {
        self.default_properties().movement_priority
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_sided_mirror_reflects_only_front_faces() {
        let mirror = BlockKind::Mirror.default_properties();
        let rot_id = CubeRot::IDENTITY;

        // Front face reflection: +X incoming (from West) -> reflects +Y (North)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(1, 0, 0), &rot_id),
            Some(IVec3::new(0, 1, 0))
        );

        // Front face reflection: +Y incoming (from South) -> reflects +X (East)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(0, 1, 0), &rot_id),
            Some(IVec3::new(1, 0, 0))
        );

        // Back face: -X incoming (from East hitting back) -> BLOCKED (None)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(-1, 0, 0), &rot_id),
            None
        );

        // Back face: -Y incoming (from North hitting back) -> BLOCKED (None)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(0, -1, 0), &rot_id),
            None
        );

        // Z axis rays hitting caps -> BLOCKED (None)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(0, 0, 1), &rot_id),
            None
        );
        assert_eq!(
            mirror.reflect_laser(IVec3::new(0, 0, -1), &rot_id),
            None
        );
    }

    #[test]
    fn rotated_single_sided_mirror_transforms_correctly() {
        let mirror = BlockKind::Mirror.default_properties();
        // Rotate 90° CCW about Z
        let rot_ccw = CubeRot::ROT_Z_90;

        // +Y incoming (from South) strikes front -> reflects -X (West)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(0, 1, 0), &rot_ccw),
            Some(IVec3::new(-1, 0, 0))
        );

        // -X incoming (from East) strikes front -> reflects +Y (North)
        assert_eq!(
            mirror.reflect_laser(IVec3::new(-1, 0, 0), &rot_ccw),
            Some(IVec3::new(0, 1, 0))
        );

        // +X incoming strikes back wall -> BLOCKED
        assert_eq!(
            mirror.reflect_laser(IVec3::new(1, 0, 0), &rot_ccw),
            None
        );

        // -Y incoming strikes back wall -> BLOCKED
        assert_eq!(
            mirror.reflect_laser(IVec3::new(0, -1, 0), &rot_ccw),
            None
        );
    }

    #[test]
    fn planar_reflection_property_transform() {
        let mirror = BlockKind::Mirror.default_properties();
        // Reflect across X=0 plane (normal = X)
        let reflected_mirror = mirror.reflect_across_plane(IVec3::X);

        // Now incoming -X ray reflects to +Y
        assert_eq!(
            reflected_mirror.reflect_laser(IVec3::new(-1, 0, 0), &CubeRot::IDENTITY),
            Some(IVec3::new(0, 1, 0))
        );
    }
}
