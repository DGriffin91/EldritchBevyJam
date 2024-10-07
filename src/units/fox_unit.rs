use std::time::Duration;

use bevy::prelude::*;

use crate::{
    mesh_assets::{AnimationAssets, MeshAssets},
    util::{propagate, Propagate},
    GameLoading,
};

pub struct FoxUnitPlugin;
impl Plugin for FoxUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (propagate::<FoxUnit>, init_fox_unit_animation)
                .chain()
                .run_if(in_state(GameLoading::Loaded)),
        )
        .add_systems(OnEnter(GameLoading::Loaded), create_fox_unit);
    }
}

#[derive(Component, Clone)]
struct FoxUnit;

fn create_fox_unit(mut commands: Commands, mesh_assets: Res<MeshAssets>) {
    commands.spawn((
        SceneBundle {
            scene: mesh_assets.fox.clone(),
            transform: Transform::from_xyz(5.0, 0.0, 0.0).with_scale(Vec3::splat(0.03)),
            ..default()
        },
        FoxUnit,
        Propagate(FoxUnit),
    ));
}

fn init_fox_unit_animation(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), (Added<AnimationPlayer>, With<FoxUnit>)>,
    animation_assets: Res<AnimationAssets>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (entity, mut player) in &mut players {
        dbg!(entity);
        let mut animation_graph = AnimationGraph::new();
        let blend_node = animation_graph.add_blend(0.5, animation_graph.root);
        animation_graph.add_clip(animation_assets.fox_0.clone(), 1.0, animation_graph.root);
        animation_graph.add_clip(animation_assets.fox_1.clone(), 1.0, blend_node);
        animation_graph.add_clip(animation_assets.fox_2.clone(), 1.0, blend_node);
        let graph_handle = animation_graphs.add(animation_graph);

        dbg!("!!!");
        let mut transitions = AnimationTransitions::new();

        for i in 0..4 {
            // Make sure to start the animation via the `AnimationTransitions`
            // component. The `AnimationTransitions` component wants to manage all
            // the animations and will get confused if the animations are started
            // directly via the `AnimationPlayer`.
            transitions
                .play(&mut player, i.into(), Duration::ZERO)
                .repeat();
        }

        commands
            .entity(entity)
            .insert(transitions)
            .insert(graph_handle);
    }
}
