//! Sim state → Bevy mesh synchronization.
//!
//! This module never mutates the simulation. It reads the current
//! [`World`](crate::sim::World) and laser state each frame and keeps the
//! Bevy entity graph in sync.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

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

    // Moveable (vibrant, saturated, smooth)
    pub moveable_pushable_mat: Handle<StandardMaterial>,
    pub moveable_mirror_mat: Handle<StandardMaterial>,
    pub moveable_laser_mat: Handle<StandardMaterial>,

    // Stationary / Fixed (desaturated, darker, with stone grid texture)
    pub fixed_wall_mat: Handle<StandardMaterial>,
    pub fixed_pushable_mat: Handle<StandardMaterial>,
    pub fixed_mirror_mat: Handle<StandardMaterial>,
    pub fixed_laser_mat: Handle<StandardMaterial>,

    // Laser Source Indicator & Beams
    pub laser_indicator_mat: Handle<StandardMaterial>,
    pub laser_core_mat: Handle<StandardMaterial>,
    pub laser_glow_mat: Handle<StandardMaterial>,
    pub laser_impact_mat: Handle<StandardMaterial>,

    // Goal
    pub goal_mat: Handle<StandardMaterial>,
    pub goal_won_mat: Handle<StandardMaterial>,
    pub rounded_cube_mesh: Handle<Mesh>,
    pub rounded_mirror_mesh: Handle<Mesh>,
    pub rounded_mirror_mesh_chiral: Handle<Mesh>,
    pub rounded_pyramid_mesh: Handle<Mesh>,

    pub floor_mat: Handle<StandardMaterial>,
    pub moveable_glass_mat: Handle<StandardMaterial>,
    pub fixed_glass_mat: Handle<StandardMaterial>,
    pub moveable_goal_mat: Handle<StandardMaterial>,

}

/// Startup system — create shared meshes and materials.
pub fn setup_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let cube = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
    let player = meshes.add(create_dodecahedron_mesh());
    let mirror = meshes.add(create_mirror_mesh());
    let mirror_chiral = meshes.add(create_chiral_mirror_mesh());
    let pyramid = meshes.add(create_pyramid_mesh());
    let indicator = meshes.add(Cuboid::new(0.3, 0.3, 0.15));

    
    let rounded_cube = meshes.add(create_rounded_cube_mesh(0.9, 0.08));
    let rounded_mirror = meshes.add(create_rounded_mirror_mesh(0.08));
    let rounded_mirror_chiral = meshes.add(create_rounded_chiral_mirror_mesh(0.08));
    let rounded_pyramid = meshes.add(create_rounded_pyramid_mesh(0.08));

    // Continuous cylinder meshes for lasers:
    let laser_core_mesh = meshes.add(Cylinder::new(0.04, 1.0));
    let laser_glow_mesh = meshes.add(Cylinder::new(0.12, 1.0));
    let laser_impact_mesh = meshes.add(Sphere::new(0.12));

    // Procedural texture for stationary/fixed blocks
    let stone_texture = images.add(create_fixed_block_texture());

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

        // Moveable Blocks (Vibrant, Saturated, Smooth finish)
        moveable_pushable_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.65, 0.12), // Bright golden orange
            perceptual_roughness: 0.3,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        moveable_mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.98, 1.0), // Gleaming bright polished silver
            metallic: 0.6,
            perceptual_roughness: 0.08,
            emissive: LinearRgba::new(0.14, 0.16, 0.20, 1.0), // Clean silver specular glow
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        moveable_laser_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.15, 0.15), // Bright crimson
            perceptual_roughness: 0.3,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),

        // Stationary / Fixed Blocks (Desaturated, darker, with 12-dot polka dot texture)
        fixed_wall_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.48, 0.48, 0.52),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.8,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        fixed_pushable_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.46, 0.42),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.7,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        fixed_mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.88, 0.91, 0.96), // Bright silver with polka dot pattern
            base_color_texture: Some(stone_texture.clone()),
            metallic: 0.45,
            perceptual_roughness: 0.15,
            emissive: LinearRgba::new(0.10, 0.12, 0.16, 1.0), // Bright silver radiance
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
        fixed_laser_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.35, 0.35),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.7,
            cull_mode: None,
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

        // Goal Pyramid: default golden crystal
        goal_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.8, 0.15),
            metallic: 0.7,
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.4, 0.3, 0.05, 1.0),
            double_sided: true,
            ..default()
        }),

        // Goal Pyramid: triumphant radiant victory glow
        goal_won_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 1.0, 0.8),
            metallic: 0.8,
            perceptual_roughness: 0.1,
            emissive: LinearRgba::new(20.0, 15.0, 3.0, 1.0),
            double_sided: true,
            ..default()
        }),
        moveable_goal_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.25),
            metallic: 0.7,
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.5, 0.4, 0.1, 1.0),
            double_sided: true,
            ..default()
        }),
        floor_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.38, 0.42),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.9,
            double_sided: true,
            ..default()
        }),
        moveable_glass_mat: materials.add(StandardMaterial {
            base_color: Color::srgba(0.7, 0.85, 1.0, 0.25),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.05,
            emissive: LinearRgba::new(0.1, 0.15, 0.25, 1.0),
            double_sided: true,
            ..default()
        }),
        fixed_glass_mat: materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.55, 0.6, 0.3),
            alpha_mode: AlphaMode::Blend,
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.1,
            double_sided: true,
            ..default()
        }),
    });
}

// ---------------------------------------------------------------------------
// Procedural Textures & Custom Meshes
// ---------------------------------------------------------------------------

/// Create a procedural polka dot texture for stationary/immovable blocks.
/// Features bold polka dots tilted at a 25° angle with skewed rows and soft contrast (12 dots per face).
fn create_fixed_block_texture() -> Image {
    let width = 128;
    let height = 128;
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    // 25-degree tilt angle
    let angle_rad = 25.0_f32.to_radians();
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // 3 rows along Y
    let y_rows = [21.33_f32, 64.0, 106.67];
    let row_skews = [0.0_f32, 12.0, 24.0];
    let base_x = [16.0_f32, 48.0, 80.0, 112.0];
    let radius = 11.5_f32;

    for y in 0..height {
        let py = y as f32 + 0.5;
        for x in 0..width {
            let px = x as f32 + 0.5;

            // Rotate coordinates by 25° about center (64, 64)
            let cx_offset = px - 64.0;
            let cy_offset = py - 64.0;
            let rx = (cx_offset * cos_a - cy_offset * sin_a + 64.0).rem_euclid(128.0);
            let ry = (cx_offset * sin_a + cy_offset * cos_a + 64.0).rem_euclid(128.0);

            // Find distance to closest dot center in the tilted frame
            let mut min_dist_sq = f32::MAX;
            for (row_idx, &cy) in y_rows.iter().enumerate() {
                let skew = row_skews[row_idx];
                for &bx in &base_x {
                    let cx = (bx + skew) % 128.0;

                    let mut dx = (rx - cx).abs();
                    if dx > 64.0 {
                        dx = 128.0 - dx;
                    }
                    let mut dy = (ry - cy).abs();
                    if dy > 64.0 {
                        dy = 128.0 - dy;
                    }

                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                    }
                }
            }

            let dist = min_dist_sq.sqrt();
            let dot_factor = (1.0 - (dist - (radius - 1.2)).clamp(0.0, 1.5) / 1.5).clamp(0.0, 1.0);

            // Subtle border frame around the block face
            let is_border = x <= 2 || x >= width - 3 || y <= 2 || y >= height - 3;

            let (r, g, b) = if is_border {
                (42, 45, 52) // Dark border rim
            } else {
                // Background dark slate base: (72, 75, 82)
                // Soft muted polka dot color (less bright): (132, 138, 148)
                let base_r = 72.0_f32;
                let base_g = 75.0_f32;
                let base_b = 82.0_f32;

                let dot_r = 132.0_f32;
                let dot_g = 138.0_f32;
                let dot_b = 148.0_f32;

                let r = (base_r * (1.0 - dot_factor) + dot_r * dot_factor) as u8;
                let g = (base_g * (1.0 - dot_factor) + dot_g * dot_factor) as u8;
                let b = (base_b * (1.0 - dot_factor) + dot_b * dot_factor) as u8;
                (r, g, b)
            };

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(255);
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

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

fn create_rounded_mirror_mesh(_bevel: f32) -> Mesh {
    create_mirror_mesh()
}

fn create_rounded_chiral_mirror_mesh(_bevel: f32) -> Mesh {
    create_chiral_mirror_mesh()
}

fn create_rounded_pyramid_mesh(_bevel: f32) -> Mesh {
    create_pyramid_mesh()
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
                    if game.engine.is_won() {
                        mat_handle.0 = assets.goal_won_mat.clone();
                    } else if is_moveable {
                        mat_handle.0 = assets.moveable_goal_mat.clone();
                    } else {
                        mat_handle.0 = assets.goal_mat.clone();
                    }
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
                let mat = if game.engine.is_won() {
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

    let min_x = -1.5_f32;
    let max_x = 8.5_f32;
    let min_sim_y = -1.5_f32;
    let max_sim_y = 7.5_f32;

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
