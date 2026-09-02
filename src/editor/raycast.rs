//! 3D Raycasting and geometry utilities for grid and block intersections in the editor.

use bevy::prelude::*;
use crate::sim::{BodyId, World};

/// Intersect a ray with an Axis-Aligned Bounding Box (AABB).
/// Returns the distance `t` along the ray if hit, or `None`.
pub fn ray_intersect_aabb(
    ray_origin: Vec3,
    ray_dir: Vec3,
    min: Vec3,
    max: Vec3,
) -> Option<f32> {
    let inv_d = Vec3::new(
        if ray_dir.x.abs() > 1e-6 { 1.0 / ray_dir.x } else { f32::INFINITY },
        if ray_dir.y.abs() > 1e-6 { 1.0 / ray_dir.y } else { f32::INFINITY },
        if ray_dir.z.abs() > 1e-6 { 1.0 / ray_dir.z } else { f32::INFINITY },
    );

    let t0 = (min - ray_origin) * inv_d;
    let t1 = (max - ray_origin) * inv_d;

    let tmin_v = t0.min(t1);
    let tmax_v = t0.max(t1);

    let tmin = tmin_v.x.max(tmin_v.y).max(tmin_v.z);
    let tmax = tmax_v.x.min(tmax_v.y).min(tmax_v.z);

    if tmin <= tmax && tmax >= 0.0 {
        Some(if tmin >= 0.0 { tmin } else { tmax })
    } else {
        None
    }
}

/// Raycasts against a flat horizontal plane at sim Z layer `z_level`.
/// Returns the grid coordinate `IVec3(gx, gy, z_level)`.
pub fn raycast_plane_at_z(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
    z_level: i32,
) -> Option<IVec3> {
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    let ray_dir: Vec3 = ray.direction.into();
    if ray_dir.y.abs() < 1e-5 {
        return None;
    }
    // In Bevy coordinates, Sim Z is along the Y axis: Y_bevy = z_level as f32
    let target_y = z_level as f32;
    let t = (target_y - ray.origin.y) / ray_dir.y;
    if t < 0.0 {
        return None;
    }
    let world_pt = ray.origin + ray_dir * t;
    let gx = (world_pt.x + 0.5).floor() as i32;
    let gy = (-world_pt.z + 0.5).floor() as i32;
    Some(IVec3::new(gx, gy, z_level))
}

/// Raycasts into the 3D scene to find the highest block column or ground plane at `ground_z`.
/// Returns the cell `(IVec3, Option<BodyId>)` indicating the target placement cell on top.
pub fn raycast_stack_on_top(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
    world: &World,
    ground_z: i32,
    ignore_body: Option<BodyId>,
) -> Option<(IVec3, Option<BodyId>)> {
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    let ray_dir: Vec3 = ray.direction.into();

    let mut closest: Option<(f32, IVec3, BodyId)> = None;

    for body in world.bodies() {
        if Some(body.id) == ignore_body {
            continue;
        }
        for world_cell in body.world_cells() {
            // Bevy coordinate conversion: (world_cell.x, world_cell.z, -world_cell.y)
            let center = Vec3::new(
                world_cell.x as f32,
                world_cell.z as f32,
                -world_cell.y as f32,
            );
            let half = Vec3::splat(0.5);
            let min = center - half;
            let max = center + half;

            if let Some(t) = ray_intersect_aabb(ray.origin, ray_dir, min, max) {
                if closest.is_none() || t < closest.unwrap().0 {
                    closest = Some((t, world_cell, body.id));
                }
            }
        }
    }

    if let Some((_t, hit_cell, hit_body_id)) = closest {
        // Find highest occupied Z coordinate among all bodies at column (hit_cell.x, hit_cell.y)
        let mut max_z = hit_cell.z;
        for body in world.bodies() {
            if Some(body.id) == ignore_body {
                continue;
            }
            for cell in body.world_cells() {
                if cell.x == hit_cell.x && cell.y == hit_cell.y && cell.z > max_z {
                    max_z = cell.z;
                }
            }
        }
        let target_cell = IVec3::new(hit_cell.x, hit_cell.y, max_z + 1);
        Some((target_cell, Some(hit_body_id)))
    } else {
        // No block hit directly -> raycast ground plane at ground_z and check if column has blocks
        let ground_cell = raycast_plane_at_z(camera, camera_transform, cursor_pos, ground_z)?;
        let mut max_z = ground_z - 1;
        let mut found_body = None;
        for body in world.bodies() {
            if Some(body.id) == ignore_body {
                continue;
            }
            for cell in body.world_cells() {
                if cell.x == ground_cell.x && cell.y == ground_cell.y && cell.z > max_z {
                    max_z = cell.z;
                    found_body = Some(body.id);
                }
            }
        }
        if max_z >= ground_z {
            Some((IVec3::new(ground_cell.x, ground_cell.y, max_z + 1), found_body))
        } else {
            Some((IVec3::new(ground_cell.x, ground_cell.y, ground_z), None))
        }
    }
}
