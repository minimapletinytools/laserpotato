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
    pub pyramid_mesh: Handle<Mesh>,
    pub indicator_mesh: Handle<Mesh>,
    pub laser_core_mesh: Handle<Mesh>,
    pub laser_glow_mesh: Handle<Mesh>,
    pub laser_impact_mesh: Handle<Mesh>,

    // Player
    pub player_mat: Handle<StandardMaterial>,
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
    let pyramid = meshes.add(create_pyramid_mesh());
    let indicator = meshes.add(Cuboid::new(0.3, 0.3, 0.15));

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
        pyramid_mesh: pyramid,
        indicator_mesh: indicator,
        laser_core_mesh,
        laser_glow_mesh,
        laser_impact_mesh,

        // Player (vibrant blue dodecahedron)
        player_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.6, 1.0),
            perceptual_roughness: 0.2,
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
            ..default()
        }),
        moveable_mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.96, 1.0), // Polished bright chrome
            metallic: 0.95,
            perceptual_roughness: 0.05,
            double_sided: true,
            ..default()
        }),
        moveable_laser_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.15, 0.15), // Bright crimson
            perceptual_roughness: 0.3,
            ..default()
        }),

        // Stationary / Fixed Blocks (Desaturated, darker, stone texture)
        fixed_wall_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.45, 0.48),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.8,
            ..default()
        }),
        fixed_pushable_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.48, 0.44, 0.4),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.7,
            ..default()
        }),
        fixed_mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.72, 0.78), // Desaturated gunmetal mirror
            metallic: 0.85,
            perceptual_roughness: 0.15,
            double_sided: true,
            ..default()
        }),
        fixed_laser_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.3, 0.3),
            base_color_texture: Some(stone_texture.clone()),
            perceptual_roughness: 0.7,
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
            ..default()
        }),

        // Goal Pyramid: triumphant radiant victory glow
        goal_won_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 1.0, 0.8),
            metallic: 0.8,
            perceptual_roughness: 0.1,
            emissive: LinearRgba::new(20.0, 15.0, 3.0, 1.0),
            ..default()
        }),
    });
}

// ---------------------------------------------------------------------------
// Procedural Textures & Custom Meshes
// ---------------------------------------------------------------------------

/// Create a procedural stone grid / masonry texture for stationary blocks.
fn create_fixed_block_texture() -> Image {
    let width = 64;
    let height = 64;
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let is_border = x <= 2 || x >= width - 3 || y <= 2 || y >= height - 3;
            let is_groove = (x >= 30 && x <= 33) || (y >= 30 && y <= 33);
            let noise = ((x * 13 + y * 37) % 23) as u8;

            let (r, g, b) = if is_border {
                (45, 45, 50)
            } else if is_groove {
                (65, 65, 72)
            } else {
                let base = 145 + noise;
                (base, base, base + 4)
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

/// Right-triangular prism for the mirror.
/// All faces are wound Counter-Clockwise (CCW) facing outward.
fn create_mirror_mesh() -> Mesh {
    let s: f32 = 0.45;
    let n: f32 = std::f32::consts::FRAC_1_SQRT_2;

    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        // --- Top cap (+Y: CCW when viewed from above) ---
        [-s,  s,  s],  [ s,  s, -s],  [-s,  s, -s],   // 0 1 2

        // --- Bottom cap (-Y: CCW when viewed from below) ---
        [-s, -s,  s],  [-s, -s, -s],  [ s, -s, -s],   // 3 4 5

        // --- Back wall 1 (-Z: CCW when viewed from -Z) ---
        [-s, -s, -s],  [-s,  s, -s],  [ s,  s, -s],  [ s, -s, -s],  // 6 7 8 9

        // --- Back wall 2 (-X: CCW when viewed from -X) ---
        [-s, -s,  s],  [-s,  s,  s],  [-s,  s, -s],  [-s, -s, -s],  // 10 11 12 13

        // --- Hypotenuse (+X, +Z: CCW when viewed from +X, +Z) ---
        [-s, -s,  s],  [ s, -s, -s],  [ s,  s, -s],  [-s,  s,  s],  // 14 15 16 17
    ];

    #[rustfmt::skip]
    let normals: Vec<[f32; 3]> = vec![
        // Top (+Y)
        [0., 1., 0.],  [0., 1., 0.],  [0., 1., 0.],
        // Bottom (-Y)
        [0.,-1., 0.],  [0.,-1., 0.],  [0.,-1., 0.],
        // Back wall 1 (-Z)
        [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],
        // Back wall 2 (-X)
        [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.],
        // Hypotenuse (+X, +Z)
        [n, 0., n],    [n, 0., n],    [n, 0., n],    [n, 0., n],
    ];

    #[rustfmt::skip]
    let uvs: Vec<[f32; 2]> = vec![
        // Top
        [0.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        // Bottom
        [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        // Back wall 1
        [0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        // Back wall 2
        [0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0],
        // Hypotenuse
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
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

// ---------------------------------------------------------------------------
// Coordinate helpers
// ---------------------------------------------------------------------------

pub fn sim_to_bevy(pos: glam::IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32)
}

pub fn sim_to_bevy_f32(pos: Vec3) -> Vec3 {
    Vec3::new(pos.x, pos.z, -pos.y)
}

fn cube_rot_to_quat(rot: &CubeRot) -> Quat {
    let m = rot.mat();
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
    mut query: Query<(Entity, &SimBodyLink, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let world = &game.engine.world;
    let mut seen = std::collections::HashSet::new();
    let mut to_despawn = Vec::new();

    // Update existing entities (position, rotation, and dynamic goal material).
    for (entity, link, mut transform, mut mat_handle) in &mut query {
        if let Some(body) = world.body(link.0) {
            transform.translation = sim_to_bevy(body.anchor);
            transform.rotation = cube_rot_to_quat(&body.orientation);

            // Update goal block material if level won state changes
            if body.kind == BlockKind::Goal {
                mat_handle.0 = if game.engine.is_won {
                    assets.goal_won_mat.clone()
                } else {
                    assets.goal_mat.clone()
                };
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
            BlockKind::Player => (assets.player_mesh.clone(), assets.player_mat.clone()),
            BlockKind::Goal => {
                let mat = if game.engine.is_won {
                    assets.goal_won_mat.clone()
                } else {
                    assets.goal_mat.clone()
                };
                (assets.pyramid_mesh.clone(), mat)
            }
            BlockKind::Wall => (assets.cube_mesh.clone(), assets.fixed_wall_mat.clone()),
            BlockKind::Pushable => {
                let mat = if is_moveable {
                    assets.moveable_pushable_mat.clone()
                } else {
                    assets.fixed_pushable_mat.clone()
                };
                (assets.cube_mesh.clone(), mat)
            }
            BlockKind::Mirror => {
                let mat = if is_moveable {
                    assets.moveable_mirror_mat.clone()
                } else {
                    assets.fixed_mirror_mat.clone()
                };
                (assets.mirror_mesh.clone(), mat)
            }
            BlockKind::LaserSource => {
                let mat = if is_moveable {
                    assets.moveable_laser_mat.clone()
                } else {
                    assets.fixed_laser_mat.clone()
                };
                (assets.cube_mesh.clone(), mat)
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

/// Every frame: despawn all old beam entities and render continuous solid laser lines
/// with glowing cores, outer sheaths, hit flares, and dynamic point lights.
pub fn sync_lasers(
    mut commands: Commands,
    game: Res<GameState>,
    assets: Res<RenderAssets>,
    beams: Query<Entity, With<LaserBeamMarker>>,
) {
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
                    if hit_body.kind == BlockKind::Mirror {
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
// Coordinate Gizmo & Debug Displays
// ---------------------------------------------------------------------------

/// Toggle for the 3D coordinate frame gizmo in the bottom-left.
pub const SHOW_COORDINATE_GIZMO: bool = true;

/// Toggle for the 2D coordinate axes legend HUD overlay in the bottom-left.
pub const SHOW_COORDINATE_LEGEND: bool = true;

/// Draw 3D coordinate arrows in the bottom-left corner showing the game's Sim coordinate axes:
/// - Red arrow: +X (Right)
/// - Green arrow: +Y (Forward)
/// - Blue arrow: +Z (Up)
pub fn draw_coordinate_gizmo(mut gizmos: Gizmos) {
    if !SHOW_COORDINATE_GIZMO {
        return;
    }

    // Bottom-left origin in Bevy coordinates (-X, +Z)
    let origin = Vec3::new(-2.2, 0.2, 2.2);
    let len = 1.0;

    // +X (Right in Sim = +X in Bevy) - Red
    gizmos.arrow(origin, origin + Vec3::new(len, 0.0, 0.0), Color::srgb(1.0, 0.25, 0.25));
    // +Y (Forward in Sim = -Z in Bevy) - Green
    gizmos.arrow(origin, origin + Vec3::new(0.0, 0.0, -len), Color::srgb(0.25, 1.0, 0.25));
    // +Z (Up in Sim = +Y in Bevy) - Blue
    gizmos.arrow(origin, origin + Vec3::new(0.0, len, 0.0), Color::srgb(0.3, 0.6, 1.0));
}
