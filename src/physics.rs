use crate::util::{all_children, propagate_to_name};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsStuff;
impl Plugin for PhysicsStuff {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            //.add_plugin(RapierDebugRenderPlugin::default())
            .add_systems(
                Update,
                (
                    propagate_to_name::<AddTrimeshPhysics>,
                    setup_trimesh_colliders,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Clone, Copy)]
pub struct AddTrimeshPhysics;

pub fn setup_trimesh_colliders(
    mut commands: Commands,
    scene_entities: Query<Entity, With<AddTrimeshPhysics>>,
    children_query: Query<&Children>,
    mesh_handles: Query<&Handle<Mesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    for entity in scene_entities.iter() {
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                if let Ok(mesh_h) = mesh_handles.get(entity) {
                    let mesh = meshes.get(mesh_h).unwrap();
                    // TODO seems inefficient if there are multiple instances of the same trimesh collider
                    commands.entity(entity).insert((
                        Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
                        //RigidBody::Fixed, // Seemed to be moving some objects or something, but collision works without
                    ));
                }
            });
            commands.entity(entity).remove::<AddTrimeshPhysics>();
        }
    }
}
