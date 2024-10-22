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
                    propagate_to_name::<AddCuboidColliders>,
                    setup_cuboid_colliders,
                    propagate_to_name::<AddCuboidSensors>,
                    setup_cuboid_sensors,
                    display_events_example,
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

#[derive(Component, Clone, Copy)]
pub struct AddCuboidColliders;

pub fn setup_cuboid_colliders(
    mut commands: Commands,
    scene_entities: Query<
        Entity,
        (
            With<AddCuboidColliders>,
            Without<Handle<Mesh>>,
            Without<Handle<StandardMaterial>>,
        ),
    >,
) {
    for entity in scene_entities.iter() {
        commands
            .entity(entity)
            .insert(Collider::cuboid(1.0, 1.0, 1.0));
        commands.entity(entity).remove::<AddCuboidColliders>();
    }
}

#[derive(Component, Clone, Copy)]
pub struct AddCuboidSensors;

pub fn setup_cuboid_sensors(
    mut commands: Commands,
    scene_entities: Query<
        Entity,
        (
            With<AddCuboidSensors>,
            Without<Handle<Mesh>>,
            Without<Handle<StandardMaterial>>,
        ),
    >,
) {
    for entity in scene_entities.iter() {
        commands.entity(entity).insert((
            Collider::cuboid(1.0, 1.0, 1.0),
            Sensor,
            RigidBodyDisabled,
        ));
        commands.entity(entity).remove::<AddCuboidSensors>();
    }
}

pub fn display_events_example(
    mut collision_events: EventReader<CollisionEvent>,
    //mut contact_force_events: EventReader<ContactForceEvent>,
    names: Query<&Name>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity, entity1, _collision_event_flags) => {
                let (hit_entity, hit_name) =
                    get_sensor_entity_and_name(&names, entity, entity1, "SENSOR");
                println!(
                    "Started collision event: {collision_event:?} {hit_entity:?}, {hit_name:?}"
                );
            }
            CollisionEvent::Stopped(entity, entity1, _collision_event_flags) => {
                let (hit_entity, hit_name) =
                    get_sensor_entity_and_name(&names, entity, entity1, "SENSOR");
                println!(
                    "Stopped collision event: {collision_event:?} {hit_entity:?}, {hit_name:?}"
                );
            }
        };
    }

    //for contact_force_event in contact_force_events.read() {
    //    println!("Received contact force event: {contact_force_event:?}");
    //}
}

pub fn get_sensor_entity_and_name(
    names: &Query<&Name>,
    entity: &Entity,
    entity1: &Entity,
    contains: &str,
) -> (Option<Entity>, Option<Name>) {
    let mut hit_entity = None;
    let mut hit_name = None;
    if let Ok(name) = names.get(*entity) {
        if name.contains(contains) {
            hit_entity = Some(entity);
            hit_name = Some(name);
        }
    }
    if hit_entity.is_none() {
        if let Ok(name) = names.get(*entity1) {
            if name.contains(contains) {
                hit_entity = Some(entity1);
                hit_name = Some(name);
            }
        }
    }
    (hit_entity.copied(), hit_name.cloned())
}
