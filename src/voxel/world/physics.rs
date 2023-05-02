//! Voxel collision detection and resolution.
//!
//! Based on Based on https://www.gamedev.net/tutorials/programming/general-and-gameplay-programming/swept-aabb-collision-detection-and-response-r3084/ .

use bevy::math::Vec3A;
use bevy::prelude::{
    Component, CoreSet, Deref, DerefMut, Entity, IVec3, IntoSystemConfig, Plugin, Query, Res,
    SystemSet, Transform, Vec3,
};
use bevy::render::primitives::Aabb;
use itertools::{iproduct, Itertools};
use std::cmp::Ordering;
use std::f32::{INFINITY, NEG_INFINITY};

use crate::voxel::storage::ChunkMap;
use crate::voxel::Voxel;

use super::ChunkShape;

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec3A);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Acceleration(pub Vec3A);

#[derive(Component, Deref, DerefMut)]
pub struct Drag(pub f32);

/// Threshold to consider floats equal.
const EPSILON: f32 = 1e-5;
const SIM_TIME: f32 = 1.0 / 20.0;

#[derive(Component)]
/// Marker component for entities that can collide with voxels.
pub struct Collider {
    pub aabb: Aabb,
}

#[derive(Debug)]
struct Collision {
    normal: Vec3A,
    time: f32,
}

#[cfg(test)]
impl PartialEq for Collision {
    fn eq(&self, other: &Self) -> bool {
        (self.time - other.time).abs() <= EPSILON
            && (self.normal - other.normal)
                .abs()
                .cmple(Vec3A::splat(EPSILON))
                .all()
    }
}

// Returns the time of collision (in [0, 1)) and normal if the given [Aabb] collides with a voxel at the given position.
fn swept_voxel_collision(b1: Aabb, displacement: Vec3A, b2: Aabb) -> Option<Collision> {
    // find the distance between the objects on the near and far sides for each axis
    let is_displacement_positive = displacement.cmpgt(Vec3A::ZERO);

    let inv_entry = Vec3A::select(
        is_displacement_positive,
        b2.min() - b1.max(),
        b2.max() - b1.min(),
    );
    let inv_exit = Vec3A::select(
        is_displacement_positive,
        b2.max() - b1.min(),
        b2.min() - b1.max(),
    );

    let is_displacement_nonzero: [bool; 3] = displacement.abs().cmpgt(Vec3A::splat(EPSILON)).into();
    let (mut entry, mut exit) = (Vec3A::splat(NEG_INFINITY), Vec3A::splat(INFINITY));
    for axis in 0..3 {
        if is_displacement_nonzero[axis] {
            entry[axis] = inv_entry[axis] / displacement[axis];
            exit[axis] = inv_exit[axis] / displacement[axis];
        }
    }

    let entry_time = entry.max_element(); // @todo: why is this not min/max (flipped)??
    let exit_time = exit.min_element();

    if entry_time > exit_time || entry.cmplt(Vec3A::ZERO).all() || entry.cmpge(Vec3A::ONE).any() {
        // no collision in [0, 1)
        None
    } else {
        // we collided! need to determine the normal vector based on which axis collided first.
        let comp: [bool; 3] = entry.cmpeq(Vec3A::splat(entry_time)).into();
        let axis_idx = comp.into_iter().position(|x| x).unwrap();
        let axis = Vec3A::AXES[axis_idx];
        if entry[axis_idx] > 0.0 {
            Some(Collision {
                time: entry_time,
                normal: -axis,
            })
        } else {
            Some(Collision {
                time: entry_time,
                normal: axis,
            })
        }
    }
}

fn voxel_aabb(voxel: IVec3) -> Aabb {
    let center = voxel.as_vec3a() + Vec3A::splat(0.5);
    Aabb {
        center,
        half_extents: Vec3A::splat(0.5),
    }
}

fn step(
    mut colliders: Query<(
        Entity,
        &mut Transform,
        &mut Velocity,
        Option<&Acceleration>,
        Option<&Collider>,
        Option<&Drag>,
    )>,
    voxels: Res<ChunkMap<Voxel, ChunkShape>>,
) {
    for (_entity, mut transform, mut velocity, acceleration, collider, drag) in &mut colliders {
        if let Some(acceleration) = acceleration {
            **velocity += **acceleration * SIM_TIME;
        }

        let mut displacement = **velocity * SIM_TIME;
        if let Some(&Collider { aabb }) = collider {
            let aabb = Aabb {
                center: Vec3A::from(transform.translation) + aabb.center,
                ..aabb
            };
            let start = (Vec3A::min(aabb.min(), aabb.min() + displacement).floor()).as_ivec3()
                + IVec3::NEG_Y;
            let end = Vec3A::max(aabb.max(), aabb.max() + displacement)
                .floor()
                .as_ivec3();

            println!(
                "pos: {}, start: {}, end: {}",
                aabb.center, start, end
            );

            assert!(start.cmple(end).all());

            let all_voxels = iproduct!(start.x..end.x, start.y..end.y, start.z..end.z)
                .map(|(x, y, z)| IVec3::new(x, y, z))
                .collect_vec();

            let interesting_voxels = all_voxels
                .iter()
                .filter(|voxel| voxels.voxel_at(**voxel).is_some_and(Voxel::collidable))
                .collect_vec();

            // println!(
            //     "pos: {}, voxels: {:?}, interesting voxels: {:?}",
            //     transform.translation, all_voxels, interesting_voxels
            // );

            if let Some(collision) = interesting_voxels
                .into_iter()
                .map(|voxel| swept_voxel_collision(aabb, displacement, voxel_aabb(*voxel)))
                .flatten()
                .min_by(|a, b| {
                    if a.time < b.time {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
            {
                println!("resolving collision!");

                // clip the displacement to avoid overlap
                displacement = **velocity * collision.time * SIM_TIME;

                let previous_velocity = **velocity;
                // cancel velocity in the normal direction
                **velocity -= Vec3A::dot(previous_velocity, collision.normal) * collision.normal;
            }
        }
        transform.translation += Vec3::from(displacement);
        if let Some(drag) = drag {
            **velocity *= **drag;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct PhysicsSet;

pub struct VoxelWorldPhysicsPlugin;

impl Plugin for VoxelWorldPhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(step.in_set(PhysicsSet).in_base_set(CoreSet::Update));
    }
}

#[cfg(test)]
mod test {
    use super::{swept_voxel_collision, voxel_aabb, Collision};
    use bevy::{math::Vec3A, prelude::IVec3, render::primitives::Aabb};

    #[test]
    fn test_swept_collisions() {
        let player = Aabb {
            center: Vec3A::new(0., 0.9, 0.0), // player is 1.8m tall, standing at the origin with feet touching the xz plane.
            half_extents: Vec3A::new(0.25, 0.9, 0.25), // player is 0.5m wide and 1.8m tall
        };

        // no collision with a voxel centered at (1.5, 0.5, 1.5)
        assert!(
            swept_voxel_collision(player, Vec3A::ZERO, voxel_aabb(IVec3::new(1, 0, 1))).is_none()
        );
        // no collision with a voxel centered at (1.5, 0.5, 0.5)
        assert!(
            swept_voxel_collision(player, Vec3A::ZERO, voxel_aabb(IVec3::new(1, 0, 0))).is_none()
        );
        // player is now moving in the +x direction, and should collide:
        assert_eq!(
            swept_voxel_collision(player, Vec3A::X, voxel_aabb(IVec3::new(1, 0, 0))).unwrap(),
            Collision {
                time: 0.75,
                normal: Vec3A::NEG_X
            }
        );
        assert!(
            swept_voxel_collision(player, Vec3A::Y, voxel_aabb(IVec3::new(0, -1, 0))).is_none()
        );
        assert!(swept_voxel_collision(player, Vec3A::Y, voxel_aabb(IVec3::new(1, 0, 0))).is_none());
        assert_eq!(
            swept_voxel_collision(player, Vec3A::Y, voxel_aabb(IVec3::new(0, 2, 0))).unwrap(),
            Collision {
                time: 0.2,
                normal: Vec3A::NEG_Y
            }
        );
    }
}
