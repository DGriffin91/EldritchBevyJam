use std::time::Duration;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_egui::{egui, EguiContexts};

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
            (
                propagate::<FoxUnit>,
                init_fox_unit_animation,
                ui_example_system,
            )
                .chain()
                .run_if(in_state(GameLoading::Loaded)),
        )
        .add_systems(OnEnter(GameLoading::Loaded), create_fox_unit);
    }
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut fox: Query<(
        &mut AnimationTransitions,
        &AnimationIndices,
        &FoxUnit,
        &mut AnimationPlayer,
    )>,
) {
    let Ok((mut transitions, anim_indices, _fox_unit, mut anim_player)) = fox.get_single_mut()
    else {
        return;
    };

    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        for (name, anim_index) in anim_indices.iter() {
            if ui.button(name).clicked() {
                transitions
                    .play(&mut anim_player, *anim_index, Duration::from_secs_f32(0.2))
                    .repeat();
            }
        }
    });
}

#[derive(Component, Clone)]
struct FoxUnit;

#[derive(Component, Clone, Deref, DerefMut)]
// TODO use enum or Handle<AnimationClip> or something once this is more figured out
struct AnimationIndices(HashMap<String, AnimationNodeIndex>);

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
    for (entity, _player) in &mut players {
        dbg!(entity);
        let mut anim_indices = HashMap::new();
        let mut animation_graph = AnimationGraph::new();
        anim_indices.insert(
            String::from("idle"),
            animation_graph.add_clip(animation_assets.fox_0.clone(), 1.0, animation_graph.root),
        );
        anim_indices.insert(
            String::from("walk"),
            animation_graph.add_clip(animation_assets.fox_1.clone(), 1.0, animation_graph.root),
        );
        anim_indices.insert(
            String::from("run"),
            animation_graph.add_clip(animation_assets.fox_2.clone(), 1.0, animation_graph.root),
        );

        let graph_handle = animation_graphs.add(animation_graph);

        let transitions = AnimationTransitions::new();

        commands
            .entity(entity)
            .insert((transitions, graph_handle, AnimationIndices(anim_indices)));
    }
}
