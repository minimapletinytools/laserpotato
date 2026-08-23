//! Sim state → Bevy mesh synchronization.
//!
//! This module never mutates the simulation. It reads the current
//! [`World`](crate::sim::World) and laser state each frame and keeps the
//! Bevy entity graph in sync.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

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
    pub mirror_mesh: Handle<Mesh>,
    pub indicator_mesh: Handle<Mesh>,
    pub laser_core_mesh: Handle<Mesh>,
    pub laser_glow_mesh: Handle<Mesh>,
    pub laser_impact_mesh: Handle<Mesh>,

    pub player_mat: Handle<StandardMaterial>,
    pub player_indicator_mat: Handle<StandardMaterial>,
    pub wall_mat: Handle<StandardMaterial>,
    pub pushable_mat: Handle<StandardMaterial>,
    pub mirror_mat: Handle<StandardMaterial>,
    pub laser_source_mat: Handle<StandardMaterial>,
    pub laser_indicator_mat: Handle<StandardMaterial>,
    pub laser_core_mat: Handle<StandardMaterial>,
    pub laser_glow_mat: Handle<StandardMaterial>,
    pub laser_impact_mat: Handle<StandardMaterial>,
}

/// Startup system — create shared meshes and materials.
pub fn setup_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
    let mirror = meshes.add(create_mirror_mesh());
    let indicator = meshes.add(Cuboid::new(0.3, 0.3, 0.15));

    // Continuous cylinder meshes for lasers:
    // Core is intense thin ray, glow is outer translucent beam
    let laser_core_mesh = meshes.add(Cylinder::new(0.04, 1.0));
    let laser_glow_mesh = meshes.add(Cylinder::new(0.12, 1.0));
    let laser_impact_mesh = meshes.add(Sphere::new(0.12));

    commands.insert_resource(RenderAssets {
        cube_mesh: cube,
        mirror_mesh: mirror,
        indicator_mesh: indicator,
        laser_core_mesh,
        laser_glow_mesh,
        laser_impact_mesh,

        // Player body — blue
        player_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.5, 1.0),
            ..default()
        }),
        // Player facing indicator — bright white
        player_indicator_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.95, 1.0),
            emissive: LinearRgba::new(0.6, 0.6, 1.0, 1.0),
            ..default()
        }),

        // Walls — dark gray
        wall_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        }),

        // Pushable — orange
        pushable_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.65, 0.15),
            ..default()
        }),

        // Mirror — silver metallic
        mirror_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.85, 0.95),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            double_sided: true,
            ..default()
        }),

        // Laser source body — dark red
        laser_source_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.1, 0.1),
            ..default()
        }),
        // Laser source emission indicator — bright red/orange glow
        laser_indicator_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.3, 0.1),
            emissive: LinearRgba::new(4.0, 0.8, 0.2, 1.0),
            ..default()
        }),

        // Solid continuous laser core — high emissive intense beam
        laser_core_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.9),
            emissive: LinearRgba::new(25.0, 10.0, 4.0, 1.0),
            ..default()
        }),

        // Laser outer glow sheath — translucent glowing orange
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
    });
}

// ---------------------------------------------------------------------------
// Custom meshes
// ---------------------------------------------------------------------------

/// Right-triangular prism for the mirror.
///
/// In sim coordinates, the "/" reflective surface runs along the diagonal
/// from `(-s, -s)` to `(+s, +s)` in the XY plane.
/// Rotated 180° in sim Z so that the reflective face normal points in (+X, -Y) in sim
/// (facing +X and +Z in Bevy).
fn create_mirror_mesh() -> Mesh {
    let s: f32 = 0.45;
    let n: f32 = std::f32::consts::FRAC_1_SQRT_2; // 1/√2

    // Vertices mapped from sim (x, y, z) -> Bevy (x, z, -y)
    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        // --- top cap (sim z = +s -> Bevy y = +s, normal +Y) ---
        [-s,  s,  s],  [-s,  s, -s],  [ s,  s, -s],   // 0 1 2
        // --- bottom cap (sim z = -s -> Bevy y = -s, normal -Y) ---
        [-s, -s,  s],  [ s, -s, -s],  [-s, -s, -s],   // 3 4 5
        // --- back wall 1 (sim y = +s -> Bevy z = -s, normal -Z) ---
        [-s, -s, -s],  [ s, -s, -s],  [ s,  s, -s],  [-s,  s, -s],  // 6 7 8 9
        // --- back wall 2 (sim x = -s -> Bevy x = -s, normal -X) ---
        [-s, -s,  s],  [-s, -s, -s],  [-s,  s, -s],  [-s,  s,  s],  // 10 11 12 13
        // --- hypotenuse "/" (normal facing +X, +Z in Bevy) ---
        [-s, -s,  s],  [ s, -s, -s],  [ s,  s, -s],  [-s,  s,  s],  // 14 15 16 17
    ];

    #[rustfmt::skip]
    let normals: Vec<[f32; 3]> = vec![
        // top
        [0., 1., 0.],  [0., 1., 0.],  [0., 1., 0.],
        // bottom
        [0.,-1., 0.],  [0.,-1., 0.],  [0.,-1., 0.],
        // back wall 1 (-Z)
        [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],  [0., 0.,-1.],
        // back wall 2 (-X)
        [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.],
        // hypotenuse (+X, +Z in Bevy)
        [n, 0., n],    [n, 0., n],    [n, 0., n],    [n, 0., n],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; positions.len()];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0,  1,  2,            // top
        3,  4,  5,            // bottom
        6,  7,  8,   6, 8, 9, // back wall 1
        10, 11, 12, 10,12,13, // back wall 2
        14, 15, 16, 14,16,17, // hypotenuse
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

/// Convert sim integer coordinates (Z-up, Y-forward, X-right) to Bevy coordinates (Y-up, -Z-forward, X-right).
pub fn sim_to_bevy(pos: glam::IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32)
}

/// Convert sim floating-point coordinates to Bevy coordinates.
pub fn sim_to_bevy_f32(pos: Vec3) -> Vec3 {
    Vec3::new(pos.x, pos.z, -pos.y)
}

/// Convert a sim [`CubeRot`] (Z-up frame) to a Bevy [`Quat`] (Y-up frame).
/// Uses similarity transform `R_bevy = R * M * R^T` where `R` is the 90° rotation around X.
fn cube_rot_to_quat(rot: &CubeRot) -> Quat {
    let m = rot.mat();
    let bevy_mat = Mat3::from_cols_array(&[
        m[0][0] as f32,  m[2][0] as f32, -m[1][0] as f32, // col 0
        m[0][2] as f32,  m[2][2] as f32, -m[1][2] as f32, // col 1
       -m[0][1] as f32, -m[2][1] as f32,  m[1][1] as f32, // col 2
    ]);
    Quat::from_mat3(&bevy_mat)
}

// ---------------------------------------------------------------------------
// Body sync
// ---------------------------------------------------------------------------

/// Every frame: update existing entity transforms, spawn new bodies, despawn
/// removed bodies.
pub fn sync_bodies(
    mut commands: Commands,
    game: Res<GameState>,
    assets: Res<RenderAssets>,
    mut query: Query<(Entity, &SimBodyLink, &mut Transform)>,
) {
    let world = &game.engine.world;
    let mut seen = std::collections::HashSet::new();
    let mut to_despawn = Vec::new();

    // Update existing entities (position + rotation).
    for (entity, link, mut transform) in &mut query {
        if let Some(body) = world.body(link.0) {
            transform.translation = sim_to_bevy(body.anchor);
            transform.rotation = cube_rot_to_quat(&body.orientation);
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

        let (mesh, material) = match body.kind {
            BlockKind::Player => (assets.cube_mesh.clone(), assets.player_mat.clone()),
            BlockKind::Wall => (assets.cube_mesh.clone(), assets.wall_mat.clone()),
            BlockKind::Pushable => (assets.cube_mesh.clone(), assets.pushable_mat.clone()),
            BlockKind::Mirror => (assets.mirror_mesh.clone(), assets.mirror_mat.clone()),
            BlockKind::LaserSource => (assets.cube_mesh.clone(), assets.laser_source_mat.clone()),
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
                    // "Nose" on the front face (-Z in Bevy local = +Y in sim).
                    parent.spawn((
                        Mesh3d(assets.indicator_mesh.clone()),
                        MeshMaterial3d(assets.player_indicator_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.45),
                    ));
                });
            }
            BlockKind::LaserSource => {
                entity_cmds.with_children(|parent| {
                    // Glowing dot on the face the laser emits from (-Z in Bevy local = +Y in sim).
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
    // Despawn previous beams & lights.
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

        // 2. Outer glowing translucent sheath (with animated pulse)
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
    let origin = Vec3::new(-2.2, 0.2, 1.8);
    let len = 1.0;

    // +X (Right in Sim = +X in Bevy) - Red
    gizmos.arrow(origin, origin + Vec3::new(len, 0.0, 0.0), Color::srgb(1.0, 0.25, 0.25));
    // +Y (Forward in Sim = -Z in Bevy) - Green
    gizmos.arrow(origin, origin + Vec3::new(0.0, 0.0, -len), Color::srgb(0.25, 1.0, 0.25));
    // +Z (Up in Sim = +Y in Bevy) - Blue
    gizmos.arrow(origin, origin + Vec3::new(0.0, len, 0.0), Color::srgb(0.3, 0.6, 1.0));
}
