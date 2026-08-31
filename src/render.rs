//! Sim state → Bevy mesh synchronization.
//!
//! This module never mutates the simulation. It reads the current
//! [`World`](crate::sim::World) and laser state each frame and keeps the
//! Bevy entity graph in sync.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

use crate::block_types::BlockKind;
use crate::sim::{BodyId, CubeRot};
use crate::GameState;

// ---------------------------------------------------------------------------
// Components & markers
// ---------------------------------------------------------------------------

/// Links a Bevy entity to a simulation [`Body`](crate::sim::Body).
#[derive(Component)]
pub struct SimBodyLink(pub BodyId);

/// Marks a Bevy entity as a laser beam visualization (for bulk despawn).
#[derive(Component)]
pub struct LaserBeamMarker;

/// Marker for pulsing laser glow effect.
#[derive(Component)]
pub struct LaserGlowPulse {
    pub base_scale: Vec3,
}

// ---------------------------------------------------------------------------
// Render assets (pre-created meshes & materials)
// ---------------------------------------------------------------------------

/// Shared mesh / material handles, created once at startup.
#[derive(Resource)]
pub struct RenderAssets {
    pub cube_mesh: Handle<Mesh>,
    pub player_mesh: Handle<Mesh>,
    pub mirror_mesh: Handle<Mesh>,
    pub mirror_mesh_chiral: Handle<Mesh>,
    pub pyramid_mesh: Handle<Mesh>,
    pub indicator_mesh: Handle<Mesh>,
    pub laser_core_mesh: Handle<Mesh>,
    pub laser_glow_mesh: Handle<Mesh>,
    pub laser_impact_mesh: Handle<Mesh>,

    // Player
    pub player_mat: Handle<StandardMaterial>,
    pub player_burnt_mat: Handle<StandardMaterial>,
    pub player_indicator_mat: Handle<StandardMaterial>,

    // Moveable (vibrant, saturated, chamfered)
    pub moveable_pushable_mat: Handle<StandardMaterial>,
    pub moveable_mirror_mat: Handle<StandardMaterial>,
    pub moveable_laser_mat: Handle<StandardMaterial>,
    pub moveable_glass_mat: Handle<StandardMaterial>,
    pub moveable_goal_mat: Handle<StandardMaterial>,

    // Stationary / Fixed (darker shade, sharp hard corners, solid smooth PBR)
    pub fixed_wall_mat: Handle<StandardMaterial>,
    pub fixed_pushable_mat: Handle<StandardMaterial>,
    pub fixed_mirror_mat: Handle<StandardMaterial>,
    pub fixed_laser_mat: Handle<StandardMaterial>,
    pub fixed_glass_mat: Handle<StandardMaterial>,
    pub floor_mat: Handle<StandardMaterial>,
    pub goal_mat: Handle<StandardMaterial>,

    // Laser Source Indicator & Beams
    pub laser_indicator_mat: Handle<StandardMaterial>,
    pub laser_core_mat: Handle<StandardMaterial>,
    pub laser_glow_mat: Handle<StandardMaterial>,
    pub laser_impact_mat: Handle<StandardMaterial>,

    // Goal Victory
    pub goal_won_mat: Handle<StandardMaterial>,

    // Chamfered mesh handles for moveable blocks
    pub rounded_cube_mesh: Handle<Mesh>,
    pub rounded_mirror_mesh: Handle<Mesh>,
    pub rounded_mirror_mesh_chiral: Handle<Mesh>,
    pub rounded_pyramid_mesh: Handle<Mesh>,
}

/// Startup system — create shared meshes and materials.
pub fn setup_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
    let player = meshes.add(create_dodecahedron_mesh());
    let mirror = meshes.add(create_mirror_mesh());
    let mirror_chiral = meshes.add(create_chiral_mirror_mesh());
    let pyramid = meshes.add(create_pyramid_mesh());
    let indicator = meshes.add(Cuboid::new(0.3, 0.3, 0.15));

    let rounded_cube = meshes.add(create_rounded_cube_mesh(0.9, 0.08));
    let rounded_mirror = meshes.add(create_rounded_mirror_mesh(0.06));
    let rounded_mirror_chiral = meshes.add(create_rounded_chiral_mirror_mesh(0.06));
    let rounded_pyramid = meshes.add(create_rounded_pyramid_mesh(0.06));

    // Continuous cylinder meshes for lasers:
    let laser_core_mesh = meshes.add(Cylinder::new(0.04, 1.0));
    let laser_glow_mesh = meshes.add(Cylinder::new(0.12, 1.0));
    let laser_impact_mesh = meshes.add(Sphere::new(0.12));

    commands.insert_resource(RenderAssets {
        cube_mesh: cube,
        player_mesh: player,
        mirror_mesh: mirror,
        mirror_mesh_chiral: mirror_chiral,
        pyramid_mesh: pyramid,
        indicator_mesh: indicator,
        laser_core_mesh,
        laser_glow_mesh,
        laser_impact_mesh,

        rounded_cube_mesh: rounded_cube,
        rounded_mirror_mesh: rounded_mirror,
        rounded_mirror_mesh_chiral: rounded_mirror_chiral,
        rounded_pyramid_mesh: rounded_pyramid,

        // Player (vibrant blue dodecahedron)
        player_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.6, 1.0),
            perceptual_roughness: 0.2,
            double_sided: true,
            ..default()
        }),
        // Player defeated/burnt material (charred dark red with glowing embers)
        player_burnt_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.05, 0.05),
            emissive: LinearRgba::new(5.0, 0.8, 0.1, 1.0),
            perceptual_roughness: 0.8,
            double_sided: true,
            ..default()
        }),
        player_indicator_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.95, 1.0),
            emissive: LinearRgba::new(0.6, 0.6, 1.0, 1.0),
            ..default()
        }),

        // Moveable Blocks (Chamfered, Vibrant, Saturated finish)
        moveable_pushable_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.65, 0.12), // Bright golden orange
            perceptual_roughness: 0.3,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        moveable_mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.98, 1.0), // Gleaming bright polished silver
            metallic: 0.75,
            perceptual_roughness: 0.06,
            emissive: LinearRgba::new(0.14, 0.16, 0.20, 1.0), // Clean silver specular glow
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        moveable_laser_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.15, 0.15), // Bright crimson
            perceptual_roughness: 0.3,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        moveable_glass_mat: materials.add(StandardMaterial {
            base_color: Color::srgba(0.18, 0.60, 1.0, 0.35),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.05,
            emissive: LinearRgba::new(0.15, 0.50, 1.30, 1.0),
            double_sided: true,
            ..default()
        }),
        moveable_goal_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.22),
            metallic: 0.7,
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(0.4, 0.3, 0.05, 1.0),
            double_sided: true,
            ..default()
        }),

        // Stationary / Immovable Blocks (Sharp hard corners, darker shade, solid smooth PBR)
        fixed_wall_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.15, 0.18), // Darker charcoal slate
            perceptual_roughness: 0.85,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        floor_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.20, 0.24), // Darker muted slate grey
            perceptual_roughness: 0.85,
            double_sided: true,
            ..default()
        }),
        fixed_pushable_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.26, 0.20), // Darker bronze/charcoal
            perceptual_roughness: 0.75,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        fixed_mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.78, 0.82, 0.88), // Only slightly darker polished steel
            metallic: 0.65,
            perceptual_roughness: 0.12,
            emissive: LinearRgba::new(0.06, 0.08, 0.10, 1.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        fixed_laser_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.16, 0.16), // Darker brick red
            perceptual_roughness: 0.75,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        fixed_glass_mat: materials.add(StandardMaterial {
            base_color: Color::srgba(0.10, 0.20, 0.40, 0.50), // Deeper translucent dark blue glass
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(0.04, 0.08, 0.20, 1.0),
            double_sided: true,
            ..default()
        }),
        goal_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.58, 0.45, 0.15), // Darker antique gold
            metallic: 0.6,
            perceptual_roughness: 0.3,
            double_sided: true,
            ..default()
        }),

        // Laser source emission indicator
        laser_indicator_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.3, 0.1),
            emissive: LinearRgba::new(4.0, 0.8, 0.2, 1.0),
            ..default()
        }),

        // Solid continuous laser core
        laser_core_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.9),
            emissive: LinearRgba::new(25.0, 10.0, 4.0, 1.0),
            ..default()
        }),

        // Laser outer glow sheath
        laser_glow_mat: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.3, 0.05, 0.35),
            emissive: LinearRgba::new(6.0, 1.5, 0.3, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),

        // Laser impact spark flare
        laser_impact_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.7),
            emissive: LinearRgba::new(30.0, 12.0, 4.0, 1.0),
            ..default()
        }),

        // Goal Victory (Soft radiant crystal glow)
        goal_won_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.95, 0.75),
            metallic: 0.7,
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(1.8, 1.4, 0.5, 1.0),
            double_sided: true,
            ..default()
        }),
    });
}

// ---------------------------------------------------------------------------
// Procedural Custom Meshes
// ---------------------------------------------------------------------------

/// Construct a pyramid mesh for the Goal block.
/// All faces are wound Counter-Clockwise (CCW) facing outward.
fn create_pyramid_mesh() -> Mesh {
    let s: f32 = 0.45;
    let apex_y: f32 = 0.35; // Apex height in Bevy local coordinates

    // 4 Sloping sides + base square cap (strictly CCW viewed from outside)
    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        // North face (facing -Z in Bevy: bottom-right -> bottom-left -> apex)
        [ s, -s, -s],  [-s, -s, -s],  [0.0, apex_y, 0.0], // 0 1 2

        // East face (facing +X in Bevy: bottom-right -> bottom-left -> apex)
        [ s, -s,  s],  [ s, -s, -s],  [0.0, apex_y, 0.0], // 3 4 5

        // South face (facing +Z in Bevy: bottom-right -> bottom-left -> apex)
        [-s, -s,  s],  [ s, -s,  s],  [0.0, apex_y, 0.0], // 6 7 8

        // West face (facing -X in Bevy: bottom-right -> bottom-left -> apex)
        [-s, -s, -s],  [-s, -s,  s],  [0.0, apex_y, 0.0], // 9 10 11

        // Bottom square cap (facing -Y: CCW viewed from below)
        [-s, -s, -s],  [-s, -s,  s],  [ s, -s,  s],  [ s, -s, -s], // 12 13 14 15
    ];

    let h = apex_y + s; // Height from base to apex
    let len = (s * s + h * h).sqrt();
    let ny = s / len;
    let nside = h / len;

    #[rustfmt::skip]
    let normals: Vec<[f32; 3]> = vec![
        // North (-Z)
        [0.0, ny, -nside], [0.0, ny, -nside], [0.0, ny, -nside],
        // East (+X)
        [nside, ny, 0.0],  [nside, ny, 0.0],  [nside, ny, 0.0],
        // South (+Z)
        [0.0, ny, nside],  [0.0, ny, nside],  [0.0, ny, nside],
        // West (-X)
        [-nside, ny, 0.0], [-nside, ny, 0.0], [-nside, ny, 0.0],
        // Bottom (-Y)
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
    ];

    #[rustfmt::skip]
    let uvs: Vec<[f32; 2]> = vec![
        // North
        [1.0, 0.0], [0.0, 0.0], [0.5, 1.0],
        // East
        [1.0, 0.0], [0.0, 0.0], [0.5, 1.0],
        // South
        [1.0, 0.0], [0.0, 0.0], [0.5, 1.0],
        // West
        [1.0, 0.0], [0.0, 0.0], [0.5, 1.0],
        // Bottom
        [0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
    ];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0, 1, 2,                 // North
        3, 4, 5,                 // East
        6, 7, 8,                 // South
        9, 10, 11,               // West
        12, 13, 14,  12, 14, 15, // Bottom
    ];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// Construct a regular dodecahedron mesh for the player character.
fn create_dodecahedron_mesh() -> Mesh {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let inv_phi = 1.0 / phi;
    let s = 0.25;

    let a = s;
    let b = s * inv_phi;
    let c = s * phi;

    let verts = [
        Vec3::new(-a, -a, -a), Vec3::new(-a, -a,  a),
        Vec3::new(-a,  a, -a), Vec3::new(-a,  a,  a),
        Vec3::new( a, -a, -a), Vec3::new( a, -a,  a),
        Vec3::new( a,  a, -a), Vec3::new( a,  a,  a),
        Vec3::new(0.0, -b, -c), Vec3::new(0.0, -b,  c),
        Vec3::new(0.0,  b, -c), Vec3::new(0.0,  b,  c),
        Vec3::new(-b, -c, 0.0), Vec3::new(-b,  c, 0.0),
        Vec3::new( b, -c, 0.0), Vec3::new( b,  c, 0.0),
        Vec3::new(-c, 0.0, -b), Vec3::new(-c, 0.0,  b),
        Vec3::new( c, 0.0, -b), Vec3::new( c, 0.0,  b),
    ];

    let p = phi;
    let face_normals = [
        Vec3::new( 0.0,  1.0,  p), Vec3::new( 0.0,  1.0, -p),
        Vec3::new( 0.0, -1.0,  p), Vec3::new( 0.0, -1.0, -p),
        Vec3::new( 1.0,  p,  0.0), Vec3::new( 1.0, -p,  0.0),
        Vec3::new(-1.0,  p,  0.0), Vec3::new(-1.0, -p,  0.0),
        Vec3::new( p,  0.0,  1.0), Vec3::new( p,  0.0, -1.0),
        Vec3::new(-p,  0.0,  1.0), Vec3::new(-p,  0.0, -1.0),
    ];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();

    for n in face_normals {
        let norm = n.normalize();
        let mut face_verts: Vec<Vec3> = verts.iter().copied().collect();
        face_verts.sort_by(|v1, v2| {
            v2.dot(norm).partial_cmp(&v1.dot(norm)).unwrap()
        });
        let mut pentagon: Vec<Vec3> = face_verts.into_iter().take(5).collect();

        let center: Vec3 = pentagon.iter().sum::<Vec3>() / 5.0;
        let tangent = (pentagon[0] - center).normalize();
        let bitangent = norm.cross(tangent);

        pentagon.sort_by(|v1, v2| {
            let d1 = *v1 - center;
            let d2 = *v2 - center;
            let angle1 = d1.dot(bitangent).atan2(d1.dot(tangent));
            let angle2 = d2.dot(bitangent).atan2(d2.dot(tangent));
            angle1.partial_cmp(&angle2).unwrap()
        });

        let base_idx = positions.len() as u32;
        for v in &pentagon {
            positions.push([v.x, v.y, v.z]);
            normals.push([norm.x, norm.y, norm.z]);
            uvs.push([0.5, 0.5]);
        }

        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
            base_idx, base_idx + 3, base_idx + 4,
        ]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// Right-triangular prism for the canonical "/" mirror.
/// Hypotenuse faces South-East ([+n, 0, n] in Bevy / [+n, -n] in Sim), back walls on West (-X) and North (+Y in Sim / -Z in Bevy).
/// All faces are wound Counter-Clockwise (CCW) facing outward.
fn create_mirror_mesh() -> Mesh {
    let s: f32 = 0.45;
    let n: f32 = std::f32::consts::FRAC_1_SQRT_2;

    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        // --- Top cap (+Y in Bevy = +Z in Sim: CCW viewed from +Y) ---
        [-s,  s, -s],  [-s,  s,  s],  [ s,  s, -s],   // 0 1 2

        // --- Bottom cap (-Y in Bevy = -Z in Sim: CCW viewed from -Y) ---
        [-s, -s, -s],  [ s, -s, -s],  [-s, -s,  s],   // 3 4 5

        // --- Back wall 1 (-X in Sim = West, -X in Bevy: CCW viewed from -X) ---
        [-s,  s,  s],  [-s,  s, -s],  [-s, -s, -s],  [-s, -s,  s],  // 6 7 8 9

        // --- Back wall 2 (+Y in Sim = North, -Z in Bevy: CCW viewed from -Z) ---
        [-s,  s, -s],  [ s,  s, -s],  [ s, -s, -s],  [-s, -s, -s],  // 10 11 12 13

        // --- Hypotenuse (South-East in Sim: [+n, 0, n] in Bevy: CCW viewed from [+n, 0, n]) ---
        [ s,  s, -s],  [-s,  s,  s],  [-s, -s,  s],  [ s, -s, -s],  // 14 15 16 17
    ];

    #[rustfmt::skip]
    let normals: Vec<[f32; 3]> = vec![
        // Top (+Y)
        [0., 1., 0.],  [0., 1., 0.],  [0., 1., 0.],
        // Bottom (-Y)
        [0.,-1., 0.],  [0.,-1., 0.],  [0.,-1., 0.],
        // Back wall 1 (-X)
        [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.],
        // Back wall 2 (-Z)
        [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],
        // Hypotenuse ([+n, 0, n])
        [n, 0., n],    [n, 0., n],    [n, 0., n],    [n, 0., n],
    ];

    #[rustfmt::skip]
    let uvs: Vec<[f32; 2]> = vec![
        // Top
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        // Bottom
        [0.0, 1.0], [1.0, 0.0], [1.0, 1.0],
        // Back wall 1
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Back wall 2
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Hypotenuse
        [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0],
    ];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0,  1,  2,              // Top cap
        3,  4,  5,              // Bottom cap
        6,  7,  8,   6,  8,  9, // Back wall 1
        10, 11, 12,  10, 12, 13, // Back wall 2
        14, 15, 16,  14, 16, 17, // Hypotenuse
    ];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// Right-triangular prism for the chiral / reflected mirror (flipped across local X: x ↦ -x).
/// Hypotenuse faces South-West ([-n, 0, n] in Bevy / [-n, -n] in Sim), back walls on East (+X) and North (+Y in Sim / -Z in Bevy).
/// All faces are wound Counter-Clockwise (CCW) facing outward with outward-pointing normals.
fn create_chiral_mirror_mesh() -> Mesh {
    let s: f32 = 0.45;
    let n: f32 = std::f32::consts::FRAC_1_SQRT_2;

    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        // --- Top cap (+Y in Bevy = +Z in Sim: CCW viewed from +Y) ---
        [ s,  s, -s],  [-s,  s, -s],  [ s,  s,  s],   // 0 1 2

        // --- Bottom cap (-Y in Bevy = -Z in Sim: CCW viewed from -Y) ---
        [ s, -s, -s],  [ s, -s,  s],  [-s, -s, -s],   // 3 4 5

        // --- Back wall 1 (+X in Sim = East, +X in Bevy: CCW viewed from +X) ---
        [ s,  s, -s],  [ s,  s,  s],  [ s, -s,  s],  [ s, -s, -s],  // 6 7 8 9

        // --- Back wall 2 (+Y in Sim = North, -Z in Bevy: CCW viewed from -Z) ---
        [-s,  s, -s],  [ s,  s, -s],  [ s, -s, -s],  [-s, -s, -s],  // 10 11 12 13

        // --- Hypotenuse (South-West in Sim: [-n, 0, n] in Bevy: CCW viewed from [-n, 0, n]) ---
        [-s,  s, -s],  [ s,  s,  s],  [ s, -s,  s],  [-s, -s, -s],  // 14 15 16 17
    ];

    #[rustfmt::skip]
    let normals: Vec<[f32; 3]> = vec![
        // Top (+Y)
        [0., 1., 0.],  [0., 1., 0.],  [0., 1., 0.],
        // Bottom (-Y)
        [0.,-1., 0.],  [0.,-1., 0.],  [0.,-1., 0.],
        // Back wall 1 (+X)
        [1., 0., 0.],  [1., 0., 0.],  [1., 0., 0.],  [1., 0., 0.],
        // Back wall 2 (-Z)
        [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],
        // Hypotenuse ([-n, 0, n])
        [-n, 0., n],   [-n, 0., n],   [-n, 0., n],   [-n, 0., n],
    ];

    #[rustfmt::skip]
    let uvs: Vec<[f32; 2]> = vec![
        // Top
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        // Bottom
        [0.0, 1.0], [1.0, 0.0], [1.0, 1.0],
        // Back wall 1
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Back wall 2
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Hypotenuse
        [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0],
    ];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0,  1,  2,              // Top cap
        3,  4,  5,              // Bottom cap
        6,  7,  8,   6,  8,  9, // Back wall 1
        10, 11, 12,  10, 12, 13, // Back wall 2
        14, 15, 16,  14, 16, 17, // Hypotenuse
    ];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}


fn create_rounded_cube_mesh(size: f32, bevel: f32) -> Mesh {
    let hs = size * 0.5;
    let b = bevel;
    let hsb = hs - b;
    
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    
    macro_rules! add_quad {
        ($p1:expr, $p2:expr, $p3:expr, $p4:expr, $n:expr) => {
            let base = positions.len() as u32;
            positions.extend_from_slice(&[$p1, $p2, $p3, $p4]);
            normals.extend_from_slice(&[$n, $n, $n, $n]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
        };
    }

    macro_rules! add_tri {
        ($p1:expr, $p2:expr, $p3:expr, $n:expr) => {
            let base = positions.len() as u32;
            positions.extend_from_slice(&[$p1, $p2, $p3]);
            normals.extend_from_slice(&[$n, $n, $n]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]);
            indices.extend_from_slice(&[base, base+1, base+2]);
        };
    }

    // 6 faces
    add_quad!([-hsb, hs, -hsb], [-hsb, hs, hsb], [hsb, hs, hsb], [hsb, hs, -hsb], [0.0, 1.0, 0.0]); // +Y
    add_quad!([-hsb, -hs, hsb], [-hsb, -hs, -hsb], [hsb, -hs, -hsb], [hsb, -hs, hsb], [0.0, -1.0, 0.0]); // -Y
    add_quad!([-hsb, hsb, hs], [-hsb, -hsb, hs], [hsb, -hsb, hs], [hsb, hsb, hs], [0.0, 0.0, 1.0]); // +Z
    add_quad!([hsb, hsb, -hs], [hsb, -hsb, -hs], [-hsb, -hsb, -hs], [-hsb, hsb, -hs], [0.0, 0.0, -1.0]); // -Z
    add_quad!([hs, hsb, hsb], [hs, -hsb, hsb], [hs, -hsb, -hsb], [hs, hsb, -hsb], [1.0, 0.0, 0.0]); // +X
    add_quad!([-hs, hsb, -hsb], [-hs, -hsb, -hsb], [-hs, -hsb, hsb], [-hs, hsb, hsb], [-1.0, 0.0, 0.0]); // -X

    let n = std::f32::consts::FRAC_1_SQRT_2;
    add_quad!([-hsb, hs, hsb], [-hsb, hs, -hsb], [-hs, hsb, -hsb], [-hs, hsb, hsb], [-n, n, 0.0]); // L
    add_quad!([hsb, hs, -hsb], [hsb, hs, hsb], [hs, hsb, hsb], [hs, hsb, -hsb], [n, n, 0.0]); // R
    add_quad!([-hsb, hs, -hsb], [hsb, hs, -hsb], [hsb, hsb, -hs], [-hsb, hsb, -hs], [0.0, n, -n]); // B
    add_quad!([hsb, hs, hsb], [-hsb, hs, hsb], [-hsb, hsb, hs], [hsb, hsb, hs], [0.0, n, n]); // F

    add_quad!([-hs, -hsb, hsb], [-hs, -hsb, -hsb], [-hsb, -hs, -hsb], [-hsb, -hs, hsb], [-n, -n, 0.0]); // L
    add_quad!([hs, -hsb, -hsb], [hs, -hsb, hsb], [hsb, -hs, hsb], [hsb, -hs, -hsb], [n, -n, 0.0]); // R
    add_quad!([-hsb, -hs, -hsb], [hsb, -hs, -hsb], [hsb, -hsb, -hs], [-hsb, -hsb, -hs], [0.0, -n, -n]); // B
    add_quad!([hsb, -hs, hsb], [-hsb, -hs, hsb], [-hsb, -hsb, hs], [hsb, -hsb, hs], [0.0, -n, n]); // F

    add_quad!([-hs, hsb, hsb], [-hs, -hsb, hsb], [-hsb, -hsb, hs], [-hsb, hsb, hs], [-n, 0.0, n]); // FL
    add_quad!([hsb, hsb, hs], [hsb, -hsb, hs], [hs, -hsb, hsb], [hs, hsb, hsb], [n, 0.0, n]); // FR
    add_quad!([-hsb, hsb, -hs], [-hsb, -hsb, -hs], [-hs, -hsb, -hsb], [-hs, hsb, -hsb], [-n, 0.0, -n]); // BL
    add_quad!([hs, hsb, -hsb], [hs, -hsb, -hsb], [hsb, -hsb, -hs], [hsb, hsb, -hs], [n, 0.0, -n]); // BR

    let n3 = 1.0 / 3.0f32.sqrt();
    add_tri!([-hsb, hs, hsb], [-hs, hsb, hsb], [-hsb, hsb, hs], [-n3, n3, n3]); // TFL
    add_tri!([hsb, hs, hsb], [hsb, hsb, hs], [hs, hsb, hsb], [n3, n3, n3]); // TFR
    add_tri!([-hsb, hs, -hsb], [-hsb, hsb, -hs], [-hs, hsb, -hsb], [-n3, n3, -n3]); // TBL
    add_tri!([hsb, hs, -hsb], [hs, hsb, -hsb], [hsb, hsb, -hs], [n3, n3, -n3]); // TBR

    add_tri!([-hsb, -hs, hsb], [-hsb, -hsb, hs], [-hs, -hsb, hsb], [-n3, -n3, n3]); // BFL
    add_tri!([hsb, -hs, hsb], [hs, -hsb, hsb], [hsb, -hsb, hs], [n3, -n3, n3]); // BFR
    add_tri!([-hsb, -hs, -hsb], [-hs, -hsb, -hsb], [-hsb, -hsb, -hs], [-n3, -n3, -n3]); // BBL
    add_tri!([hsb, -hs, -hsb], [hsb, -hsb, -hs], [hs, -hsb, -hsb], [n3, -n3, -n3]); // BBR

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

fn create_rounded_mirror_mesh(bevel: f32) -> Mesh {
    let s = 0.45;
    let b = bevel.min(0.08);
    let hsb = s - b;
    let bd = b * std::f32::consts::SQRT_2;
    let n = std::f32::consts::FRAC_1_SQRT_2;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    macro_rules! add_tri {
        ($p1:expr, $p2:expr, $p3:expr, $norm:expr) => {
            let base = positions.len() as u32;
            positions.extend_from_slice(&[$p1, $p2, $p3]);
            normals.extend_from_slice(&[$norm, $norm, $norm]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]);
            indices.extend_from_slice(&[base, base + 1, base + 2]);
        };
    }

    macro_rules! add_quad {
        ($p1:expr, $p2:expr, $p3:expr, $p4:expr, $norm:expr) => {
            let base = positions.len() as u32;
            positions.extend_from_slice(&[$p1, $p2, $p3, $p4]);
            normals.extend_from_slice(&[$norm, $norm, $norm, $norm]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        };
    }

    // 1. Top Cap (+Y) - Inset Triangle
    let t0 = [-hsb, s, -hsb];
    let t1 = [-hsb, s, s - bd];
    let t2 = [s - bd, s, -hsb];
    add_tri!(t0, t1, t2, [0.0, 1.0, 0.0]);

    // 2. Bottom Cap (-Y) - Inset Triangle
    let b0 = [-hsb, -s, -hsb];
    let b1 = [s - bd, -s, -hsb];
    let b2 = [-hsb, -s, s - bd];
    add_tri!(b0, b1, b2, [0.0, -1.0, 0.0]);

    // 3. Back Wall 1 (-X Face) - Inset Quad
    let w1_top_front = [-s, hsb, s - bd];
    let w1_top_back = [-s, hsb, -hsb];
    let w1_bot_back = [-s, -hsb, -hsb];
    let w1_bot_front = [-s, -hsb, s - bd];
    add_quad!(w1_top_front, w1_top_back, w1_bot_back, w1_bot_front, [-1.0, 0.0, 0.0]);

    // 4. Back Wall 2 (-Z Face) - Inset Quad
    let w2_top_back = [-hsb, hsb, -s];
    let w2_top_right = [s - bd, hsb, -s];
    let w2_bot_right = [s - bd, -hsb, -s];
    let w2_bot_back = [-hsb, -hsb, -s];
    add_quad!(w2_top_back, w2_top_right, w2_bot_right, w2_bot_back, [0.0, 0.0, -1.0]);

    // 5. Hypotenuse Mirror Face (+X/+Z Diagonal) - Inset Quad
    let hyp_top_right = [s - b, hsb, -s + bd];
    let hyp_top_front = [-s + bd, hsb, s - b];
    let hyp_bot_front = [-s + bd, -hsb, s - b];
    let hyp_bot_right = [s - b, -hsb, -s + bd];
    add_quad!(hyp_top_right, hyp_top_front, hyp_bot_front, hyp_bot_right, [n, 0.0, n]);

    // 6. Horizontal Edge Chamfers (Top)
    add_quad!(t1, t0, w1_top_back, w1_top_front, [-n, n, 0.0]);
    add_quad!(t0, t2, w2_top_right, w2_top_back, [0.0, n, -n]);
    add_quad!(t2, t1, hyp_top_front, hyp_top_right, [0.5, n, 0.5]);

    // 7. Horizontal Edge Chamfers (Bottom)
    add_quad!(w1_bot_front, w1_bot_back, b0, b2, [-n, -n, 0.0]);
    add_quad!(w2_bot_back, w2_bot_right, b1, b0, [0.0, -n, -n]);
    add_quad!(hyp_bot_right, hyp_bot_front, b2, b1, [0.5, -n, 0.5]);

    // 8. Vertical Edge Chamfers
    add_quad!(w1_top_back, w2_top_back, w2_bot_back, w1_bot_back, [-n, 0.0, -n]);
    let front_tip_n = Vec3::new(-0.38268, 0.0, 0.92388).normalize();
    add_quad!(w1_top_front, hyp_top_front, hyp_bot_front, w1_bot_front, [front_tip_n.x, front_tip_n.y, front_tip_n.z]);
    let right_tip_n = Vec3::new(0.92388, 0.0, -0.38268).normalize();
    add_quad!(hyp_top_right, w2_top_right, w2_bot_right, hyp_bot_right, [right_tip_n.x, right_tip_n.y, right_tip_n.z]);

    // 9. Corner Triangles (6 corners)
    let n3 = 1.0 / 3.0f32.sqrt();
    add_tri!(t0, w1_top_back, w2_top_back, [-n3, n3, -n3]);
    add_tri!(t1, hyp_top_front, w1_top_front, [-0.4, 0.7, 0.6]);
    add_tri!(t2, w2_top_right, hyp_top_right, [0.6, 0.7, -0.4]);

    add_tri!(b0, w2_bot_back, w1_bot_back, [-n3, -n3, -n3]);
    add_tri!(b2, w1_bot_front, hyp_bot_front, [-0.4, -0.7, 0.6]);
    add_tri!(b1, hyp_bot_right, w2_bot_right, [0.6, -0.7, -0.4]);

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

fn reflect_mesh_x(mut mesh: Mesh) -> Mesh {
    if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        for p in positions.iter_mut() {
            p[0] = -p[0];
        }
    }
    if let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        for n in normals.iter_mut() {
            n[0] = -n[0];
        }
    }
    if let Some(Indices::U32(indices)) = mesh.indices_mut() {
        for chunk in indices.chunks_exact_mut(3) {
            chunk.swap(1, 2);
        }
    }
    mesh
}

fn create_rounded_chiral_mirror_mesh(bevel: f32) -> Mesh {
    reflect_mesh_x(create_rounded_mirror_mesh(bevel))
}

fn create_rounded_pyramid_mesh(bevel: f32) -> Mesh {
    let s: f32 = 0.45;
    let b = bevel.min(0.08);
    let hsb = s - b;
    let apex_y: f32 = 0.35;
    let n = std::f32::consts::FRAC_1_SQRT_2;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    macro_rules! add_tri {
        ($p1:expr, $p2:expr, $p3:expr, $norm:expr) => {
            let base = positions.len() as u32;
            positions.extend_from_slice(&[$p1, $p2, $p3]);
            normals.extend_from_slice(&[$norm, $norm, $norm]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]);
            indices.extend_from_slice(&[base, base + 1, base + 2]);
        };
    }

    macro_rules! add_quad {
        ($p1:expr, $p2:expr, $p3:expr, $p4:expr, $norm:expr) => {
            let base = positions.len() as u32;
            positions.extend_from_slice(&[$p1, $p2, $p3, $p4]);
            normals.extend_from_slice(&[$norm, $norm, $norm, $norm]);
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        };
    }

    let h = apex_y + s;
    let len = (s * s + h * h).sqrt();
    let ny = s / len;
    let nside = h / len;

    let apex = [0.0, apex_y - b * 0.5, 0.0];
    add_tri!([hsb, -s + b, -hsb], [-hsb, -s + b, -hsb], apex, [0.0, ny, -nside]); // North (-Z)
    add_tri!([hsb, -s + b, hsb], [hsb, -s + b, -hsb], apex, [nside, ny, 0.0]); // East (+X)
    add_tri!([-hsb, -s + b, hsb], [hsb, -s + b, hsb], apex, [0.0, ny, nside]); // South (+Z)
    add_tri!([-hsb, -s + b, -hsb], [-hsb, -s + b, hsb], apex, [-nside, ny, 0.0]); // West (-X)

    // Bottom Base Cap
    add_quad!([-hsb, -s, -hsb], [-hsb, -s, hsb], [hsb, -s, hsb], [hsb, -s, -hsb], [0.0, -1.0, 0.0]);

    // 4 Base Edge Chamfers
    add_quad!([-hsb, -s, -hsb], [hsb, -s, -hsb], [hsb, -s + b, -hsb], [-hsb, -s + b, -hsb], [0.0, -n, -n]); // North
    add_quad!([hsb, -s, -hsb], [hsb, -s, hsb], [hsb, -s + b, hsb], [hsb, -s + b, -hsb], [n, -n, 0.0]); // East
    add_quad!([hsb, -s, hsb], [-hsb, -s, hsb], [-hsb, -s + b, hsb], [hsb, -s + b, hsb], [0.0, -n, n]); // South
    add_quad!([-hsb, -s, hsb], [-hsb, -s, -hsb], [-hsb, -s + b, -hsb], [-hsb, -s + b, hsb], [-n, -n, 0.0]); // West

    // 4 Base Corner Triangles
    let n3 = 1.0 / 3.0f32.sqrt();
    add_tri!([-hsb, -s, -hsb], [-hsb, -s + b, -hsb], [-hsb, -s, -hsb], [-n3, -n3, -n3]);
    add_tri!([hsb, -s, -hsb], [hsb, -s, -hsb], [hsb, -s + b, -hsb], [n3, -n3, -n3]);
    add_tri!([hsb, -s, hsb], [hsb, -s + b, hsb], [hsb, -s, hsb], [n3, -n3, n3]);
    add_tri!([-hsb, -s, hsb], [-hsb, -s, hsb], [-hsb, -s + b, hsb], [-n3, -n3, n3]);

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
// ---------------------------------------------------------------------------
// Coordinate helpers
// ---------------------------------------------------------------------------

pub fn sim_to_bevy(pos: glam::IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32)
}

pub fn sim_to_bevy_f32(pos: Vec3) -> Vec3 {
    Vec3::new(pos.x, pos.z, -pos.y)
}

/// Convert a simulation symmetry transformation in Oh (proper rotation or reflection)
/// to a proper rotation quaternion in Bevy space.
///
/// For improper rotations / reflections (det = -1), the reflection across local X
/// is factored into the chiral mesh definition, and the remaining proper rotation
/// is converted to a `Quat`.
pub fn cube_rot_to_quat(rot: &CubeRot) -> Quat {
    let rot_proper = if rot.is_reflection() {
        rot.reflect_x()
    } else {
        *rot
    };
    let m = rot_proper.mat();
    let bevy_mat = Mat3::from_cols_array(&[
        m[0][0] as f32,  m[2][0] as f32, -m[1][0] as f32,
        m[0][2] as f32,  m[2][2] as f32, -m[1][2] as f32,
       -m[0][1] as f32, -m[2][1] as f32,  m[1][1] as f32,
    ]);
    Quat::from_mat3(&bevy_mat)
}

// ---------------------------------------------------------------------------
// Body sync
// ---------------------------------------------------------------------------

/// Every frame: update existing entity transforms, spawn new bodies, despawn
/// removed bodies, and assign materials based on moveable vs fixed status.
pub fn sync_bodies(
    mut commands: Commands,
    game: Res<GameState>,
    assets: Res<RenderAssets>,
    mut query: Query<(Entity, &SimBodyLink, &mut Transform, &mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let world = &game.engine.world;
    let mut seen = std::collections::HashSet::new();
    let mut to_despawn = Vec::new();

    let hit_body_ids: std::collections::HashSet<BodyId> = game
        .engine
        .laser_state
        .iter()
        .filter_map(|seg| seg.hit.as_ref().map(|h| h.body_id))
        .collect();

    // Update existing entities (position, rotation, mesh, and dynamic goal material).
    for (entity, link, mut transform, mut mesh_handle, mut mat_handle) in &mut query {
        if let Some(body) = world.body(link.0) {
            transform.translation = sim_to_bevy(body.anchor);
            transform.rotation = cube_rot_to_quat(&body.orientation);
            transform.scale = Vec3::ONE;

            // Update dynamic materials and chiral meshes based on orientation/fixed/moveable state
            let is_moveable = body.is_pushable();
            match body.kind {
                BlockKind::Floor => {
                    mesh_handle.0 = assets.cube_mesh.clone();
                    mat_handle.0 = assets.floor_mat.clone();
                }
                BlockKind::Glass => {
                    mesh_handle.0 = if is_moveable { assets.rounded_cube_mesh.clone() } else { assets.cube_mesh.clone() };
                    mat_handle.0 = if is_moveable { assets.moveable_glass_mat.clone() } else { assets.fixed_glass_mat.clone() };
                }
                BlockKind::Goal => {
                    let is_hit = hit_body_ids.contains(&body.id);
                    mat_handle.0 = if is_hit {
                        assets.goal_won_mat.clone()
                    } else if is_moveable {
                        assets.moveable_goal_mat.clone()
                    } else {
                        assets.goal_mat.clone()
                    };
                    mesh_handle.0 = if is_moveable { assets.rounded_pyramid_mesh.clone() } else { assets.pyramid_mesh.clone() };
                }
                BlockKind::Player => {
                    mat_handle.0 = if game.engine.is_lost() {
                        assets.player_burnt_mat.clone()
                    } else {
                        assets.player_mat.clone()
                    };
                    mesh_handle.0 = assets.player_mesh.clone();
                }
                BlockKind::Wall => {
                    mat_handle.0 = assets.fixed_wall_mat.clone();
                    mesh_handle.0 = assets.cube_mesh.clone();
                }
                BlockKind::Pushable => {
                    mat_handle.0 = if is_moveable {
                        assets.moveable_pushable_mat.clone()
                    } else {
                        assets.fixed_pushable_mat.clone()
                    };
                    mesh_handle.0 = if is_moveable {
                        assets.rounded_cube_mesh.clone()
                    } else {
                        assets.cube_mesh.clone()
                    };
                }
                BlockKind::Mirror => {
                    mesh_handle.0 = if is_moveable {
                        if body.orientation.is_reflection() {
                            assets.rounded_mirror_mesh_chiral.clone()
                        } else {
                            assets.rounded_mirror_mesh.clone()
                        }
                    } else {
                        if body.orientation.is_reflection() {
                            assets.mirror_mesh_chiral.clone()
                        } else {
                            assets.mirror_mesh.clone()
                        }
                    };
                    mat_handle.0 = if is_moveable {
                        assets.moveable_mirror_mat.clone()
                    } else {
                        assets.fixed_mirror_mat.clone()
                    };
                }
                BlockKind::LaserSource => {
                    mat_handle.0 = if is_moveable {
                        assets.moveable_laser_mat.clone()
                    } else {
                        assets.fixed_laser_mat.clone()
                    };
                    mesh_handle.0 = if is_moveable {
                        assets.rounded_cube_mesh.clone()
                    } else {
                        assets.cube_mesh.clone()
                    };
                }
            }

            seen.insert(link.0);
        } else {
            to_despawn.push(entity);
        }
    }

    // Despawn entities whose body no longer exists.
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }

    // Spawn entities for new bodies.
    for body in world.bodies() {
        if seen.contains(&body.id) {
            continue;
        }

        let is_moveable = body.is_pushable();

        let (mesh, material) = match body.kind {
            BlockKind::Floor => {
                (assets.cube_mesh.clone(), assets.floor_mat.clone())
            }
            BlockKind::Glass => {
                let mesh = if is_moveable { assets.rounded_cube_mesh.clone() } else { assets.cube_mesh.clone() };
                let mat = if is_moveable { assets.moveable_glass_mat.clone() } else { assets.fixed_glass_mat.clone() };
                (mesh, mat)
            }
            BlockKind::Player => {
                let mat = if game.engine.is_lost() {
                    assets.player_burnt_mat.clone()
                } else {
                    assets.player_mat.clone()
                };
                (assets.player_mesh.clone(), mat)
            }
            BlockKind::Goal => {
                let is_hit = hit_body_ids.contains(&body.id);
                let mat = if is_hit {
                    assets.goal_won_mat.clone()
                } else if is_moveable {
                    assets.moveable_goal_mat.clone()
                } else {
                    assets.goal_mat.clone()
                };
                let mesh = if is_moveable { assets.rounded_pyramid_mesh.clone() } else { assets.pyramid_mesh.clone() };
                (mesh, mat)
            }
            BlockKind::Wall => (assets.cube_mesh.clone(), assets.fixed_wall_mat.clone()),
            BlockKind::Pushable => {
                let mat = if is_moveable {
                    assets.moveable_pushable_mat.clone()
                } else {
                    assets.fixed_pushable_mat.clone()
                };
                let mesh = if is_moveable {
                    assets.rounded_cube_mesh.clone()
                } else {
                    assets.cube_mesh.clone()
                };
                (mesh, mat)
            }
            BlockKind::Mirror => {
                let m = if is_moveable {
                    if body.orientation.is_reflection() {
                        assets.rounded_mirror_mesh_chiral.clone()
                    } else {
                        assets.rounded_mirror_mesh.clone()
                    }
                } else {
                    if body.orientation.is_reflection() {
                        assets.mirror_mesh_chiral.clone()
                    } else {
                        assets.mirror_mesh.clone()
                    }
                };
                let mat = if is_moveable {
                    assets.moveable_mirror_mat.clone()
                } else {
                    assets.fixed_mirror_mat.clone()
                };
                (m, mat)
            }
            BlockKind::LaserSource => {
                let mat = if is_moveable {
                    assets.moveable_laser_mat.clone()
                } else {
                    assets.fixed_laser_mat.clone()
                };
                let mesh = if is_moveable {
                    assets.rounded_cube_mesh.clone()
                } else {
                    assets.cube_mesh.clone()
                };
                (mesh, mat)
            }
        };

        let transform = Transform::from_translation(sim_to_bevy(body.anchor))
            .with_rotation(cube_rot_to_quat(&body.orientation));

        let mut entity_cmds = commands.spawn((
            SimBodyLink(body.id),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
        ));

        // Directional child indicators (forward in Bevy local space is -Z).
        match body.kind {
            BlockKind::Player => {
                entity_cmds.with_children(|parent| {
                    parent.spawn((
                        Mesh3d(assets.indicator_mesh.clone()),
                        MeshMaterial3d(assets.player_indicator_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.45),
                    ));
                });
            }
            BlockKind::LaserSource => {
                entity_cmds.with_children(|parent| {
                    parent.spawn((
                        Mesh3d(assets.indicator_mesh.clone()),
                        MeshMaterial3d(assets.laser_indicator_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.45),
                    ));
                });
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Laser beam sync & PFX
// ---------------------------------------------------------------------------

/// Every frame: recalculate live laser raycasts and render continuous solid laser lines
/// with glowing cores, outer sheaths, hit flares, and dynamic point lights.
pub fn sync_lasers(
    mut commands: Commands,
    mut game: ResMut<GameState>,
    assets: Res<RenderAssets>,
    beams: Query<Entity, With<LaserBeamMarker>>,
) {
    // 1. Live dynamic recalculation of laser raycasts from current world state every frame
    // (active on frame 0, in the level editor, during solution playback, and during playtest)
    game.engine.laser_state = crate::laser::cast_all_lasers(&game.engine.world);

    for entity in &beams {
        commands.entity(entity).despawn();
    }

    let world = &game.engine.world;

    for segment in &game.engine.laser_state {
        let (start_sim, end_sim) = if let Some(source) = world.body(segment.source_id) {
            let s_start = match source.kind {
                BlockKind::LaserSource => {
                    source.anchor.as_vec3() + segment.direction.as_vec3() * 0.5
                }
                _ => source.anchor.as_vec3(),
            };

            let s_end = if let Some(hit) = &segment.hit {
                if let Some(hit_body) = world.body(hit.body_id) {
                    if hit_body.properties().reflect_laser(segment.direction, &hit_body.orientation).is_some() {
                        hit.cell.as_vec3()
                    } else {
                        hit.cell.as_vec3() - segment.direction.as_vec3() * 0.5
                    }
                } else {
                    hit.cell.as_vec3() - segment.direction.as_vec3() * 0.5
                }
            } else {
                s_start + segment.direction.as_vec3() * (segment.cells.len().max(1) as f32)
            };
            (s_start, s_end)
        } else {
            continue;
        };

        let start_bevy = sim_to_bevy_f32(start_sim);
        let end_bevy = sim_to_bevy_f32(end_sim);
        let delta = end_bevy - start_bevy;
        let length = delta.length();
        if length < 0.001 {
            continue;
        }

        let dir = delta / length;
        let midpoint = (start_bevy + end_bevy) * 0.5;
        let rotation = Quat::from_rotation_arc(Vec3::Y, dir);

        // 1. Intense solid core beam
        commands.spawn((
            LaserBeamMarker,
            Mesh3d(assets.laser_core_mesh.clone()),
            MeshMaterial3d(assets.laser_core_mat.clone()),
            Transform {
                translation: midpoint,
                rotation,
                scale: Vec3::new(1.0, length, 1.0),
            },
        ));

        // 2. Outer glowing translucent sheath
        let glow_scale = Vec3::new(1.0, length, 1.0);
        commands.spawn((
            LaserBeamMarker,
            LaserGlowPulse { base_scale: glow_scale },
            Mesh3d(assets.laser_glow_mesh.clone()),
            MeshMaterial3d(assets.laser_glow_mat.clone()),
            Transform {
                translation: midpoint,
                rotation,
                scale: glow_scale,
            },
        ));

        // 3. Impact flare and dynamic lighting if the laser hits an object
        if segment.hit.is_some() {
            commands.spawn((
                LaserBeamMarker,
                Mesh3d(assets.laser_impact_mesh.clone()),
                MeshMaterial3d(assets.laser_impact_mat.clone()),
                Transform::from_translation(end_bevy),
            ));

            commands.spawn((
                LaserBeamMarker,
                PointLight {
                    color: Color::srgb(1.0, 0.45, 0.15),
                    intensity: 1_500.0,
                    radius: 0.15,
                    ..default()
                },
                Transform::from_translation(end_bevy),
            ));
        }
    }
}

/// Animate pulsing laser glow sheath for energetic PFX feel.
pub fn animate_laser_pfx(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &LaserGlowPulse)>,
) {
    let pulse = 1.0 + 0.15 * (time.elapsed_secs() * 14.0).sin();
    for (mut transform, glow) in &mut query {
        transform.scale = Vec3::new(
            glow.base_scale.x * pulse,
            glow.base_scale.y,
            glow.base_scale.z * pulse,
        );
    }
}

// ---------------------------------------------------------------------------
// Coordinate Gizmo, Grid & Debug Displays
// ---------------------------------------------------------------------------

/// Toggle for the 3D coordinate frame gizmo in the bottom-left.
pub const SHOW_COORDINATE_GIZMO: bool = false;

/// Toggle for the 2D coordinate axes legend HUD overlay in the bottom-left.
pub const SHOW_COORDINATE_LEGEND: bool = false;

/// Toggle for the ground grid lines overlay.
pub const SHOW_GRID: bool = true;

/// Toggle for the ground grid coordinate number labels along X and Y axes.
pub const SHOW_GRID_LABELS: bool = true;

/// Draw 3D coordinate arrows in the bottom-left corner showing the game's Sim coordinate axes:
/// - Red arrow: +X (Right)
/// - Green arrow: +Y (Forward)
/// - Blue arrow: +Z (Up)
pub fn draw_coordinate_gizmo(mut gizmos: Gizmos) {
    if !SHOW_COORDINATE_GIZMO {
        return;
    }

    // Bottom-left origin in Bevy coordinates (-X, +Z)
    let origin = Vec3::new(-2.6, 0.2, 2.6);
    let len = 1.0;

    // +X (Right in Sim = +X in Bevy) - Red
    gizmos.arrow(origin, origin + Vec3::new(len, 0.0, 0.0), Color::srgb(1.0, 0.25, 0.25));
    // +Y (Forward in Sim = -Z in Bevy) - Green
    gizmos.arrow(origin, origin + Vec3::new(0.0, 0.0, -len), Color::srgb(0.25, 1.0, 0.25));
    // +Z (Up in Sim = +Y in Bevy) - Blue
    gizmos.arrow(origin, origin + Vec3::new(0.0, len, 0.0), Color::srgb(0.3, 0.6, 1.0));
}

/// Draw ground grid lines across the level area in Bevy coordinates at the active Z level.
pub fn draw_grid_gizmos(
    mut gizmos: Gizmos,
    editor: Option<Res<crate::editor::EditorState>>,
) {
    if !SHOW_GRID {
        return;
    }

    let min_x = -0.5_f32;
    let max_x = 9.5_f32;
    let min_sim_y = -0.5_f32;
    let max_sim_y = 9.5_f32;

    let floor_y0 = -0.49_f32;
    let base_color = Color::srgba(0.3, 0.35, 0.45, 0.35);

    let active_z = editor.as_ref().map(|ed| {
        if ed.z_mode == crate::editor::ZPlacementMode::FixedLayer {
            ed.current_z
        } else {
            0
        }
    }).unwrap_or(0);

    let active_floor_y = (active_z as f32) - 0.49_f32;

    let grid_color = if active_z != 0 {
        Color::srgba(0.2, 0.75, 1.0, 0.65)
    } else {
        base_color
    };

    // If active_z != 0, also draw base ground grid faintly
    if active_z != 0 {
        let mut sim_y = min_sim_y;
        while sim_y <= max_sim_y + 0.01 {
            let z = -sim_y;
            gizmos.line(
                Vec3::new(min_x, floor_y0, z),
                Vec3::new(max_x, floor_y0, z),
                Color::srgba(0.2, 0.25, 0.35, 0.20),
            );
            sim_y += 1.0;
        }
        let mut x = min_x;
        while x <= max_x + 0.01 {
            gizmos.line(
                Vec3::new(x, floor_y0, -min_sim_y),
                Vec3::new(x, floor_y0, -max_sim_y),
                Color::srgba(0.2, 0.25, 0.35, 0.20),
            );
            x += 1.0;
        }
    }

    // Grid lines along X (varying Sim Y => Bevy Z = -sim_y)
    let mut sim_y = min_sim_y;
    while sim_y <= max_sim_y + 0.01 {
        let z = -sim_y;
        gizmos.line(
            Vec3::new(min_x, active_floor_y, z),
            Vec3::new(max_x, active_floor_y, z),
            grid_color,
        );
        sim_y += 1.0;
    }

    // Grid lines along Y (varying Sim X => Bevy X)
    let mut x = min_x;
    while x <= max_x + 0.01 {
        gizmos.line(
            Vec3::new(x, active_floor_y, -min_sim_y),
            Vec3::new(x, active_floor_y, -max_sim_y),
            grid_color,
        );
        x += 1.0;
    }
}

/// Spawn 3D text labels on the floor along the -X and -Y borders showing grid coordinate numbers.
pub fn setup_grid_labels(mut commands: Commands) {
    if !SHOW_GRID_LABELS {
        return;
    }

    let floor_y = -0.30_f32;
    let label_color = Color::srgb(0.9, 0.95, 1.0);
    // Angled slightly towards the camera for clear isometric readability
    let rotation = Quat::from_rotation_x(-1.15);
    let scale = Vec3::splat(0.032);

    // X-axis coordinate numbers along the -Y (bottom) border (Sim Y = -2.6 => Bevy Z = 2.6)
    for x in -1..=8 {
        commands.spawn((
            Text2d::new(format!("{x}")),
            TextFont::from_font_size(26.0),
            TextColor(label_color),
            Transform::from_translation(Vec3::new(x as f32, floor_y, 2.6))
                .with_rotation(rotation)
                .with_scale(scale),
        ));
    }
    commands.spawn((
        Text2d::new("+X →"),
        TextFont::from_font_size(26.0),
        TextColor(Color::srgb(1.0, 0.4, 0.4)),
        Transform::from_translation(Vec3::new(9.4, floor_y, 2.6))
            .with_rotation(rotation)
            .with_scale(scale),
    ));

    // Y-axis coordinate numbers along the -X (left) border (Sim X = -2.6 => Bevy X = -2.6)
    for y in -1..=7 {
        commands.spawn((
            Text2d::new(format!("{y}")),
            TextFont::from_font_size(26.0),
            TextColor(label_color),
            Transform::from_translation(Vec3::new(-2.6, floor_y, -(y as f32)))
                .with_rotation(rotation)
                .with_scale(scale),
        ));
    }
    commands.spawn((
        Text2d::new("+Y ↑"),
        TextFont::from_font_size(26.0),
        TextColor(Color::srgb(0.4, 1.0, 0.4)),
        Transform::from_translation(Vec3::new(-2.6, floor_y, -8.2))
            .with_rotation(rotation)
            .with_scale(scale),
    ));
}

pub fn draw_combined_group_gizmos(
    mut gizmos: Gizmos,
    game: Res<GameState>,
) {
    use std::collections::HashMap;

    let mut groups: HashMap<u32, Vec<bevy::math::Vec3>> = HashMap::new();
    
    for body in game.engine.world.bodies() {
        if let Some(group_id) = body.combined_group {
            let pos = sim_to_bevy_f32(body.anchor.as_vec3());
            groups.entry(group_id).or_default().push(pos);
        }
    }
    
    for positions in groups.values() {
        if positions.len() < 2 {
            continue;
        }
        
        let color = Color::srgb(0.0, 1.0, 0.0);
        
        // Draw lines connecting the members of the group
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                gizmos.line(positions[i], positions[j], color);
            }
        }
    }
}
