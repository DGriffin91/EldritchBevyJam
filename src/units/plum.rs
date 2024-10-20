use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    animation::{init_animation_graph, AnimClips, AnimationIndices},
    mesh_assets::MeshAssets,
    util::{propagate, Propagate},
    GameLoading,
};

pub struct PlumUnitPlugin;
impl Plugin for PlumUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                propagate::<PlumUnit, AnimationPlayer>,
                init_animation_graph::<PlumUnit>,
                ui_example_system,
            )
                .chain()
                .run_if(in_state(GameLoading::Loaded)),
        );
    }
}

#[derive(Component, Clone)]
struct PlumUnit;

impl AnimClips for PlumUnit {
    fn get_gltf_id(&self, mesh_assets: &MeshAssets) -> Handle<Gltf> {
        mesh_assets.plum_gltf.clone_weak()
    }
}

fn ui_example_system(
    mut commands: Commands,
    mesh_assets: Res<MeshAssets>,
    mut contexts: EguiContexts,
    mut plum: Query<(
        &mut AnimationTransitions,
        &AnimationIndices,
        &PlumUnit,
        &mut AnimationPlayer,
    )>,
) {
    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        if ui.button("SPAWN").clicked() {
            commands.spawn((
                SceneBundle {
                    scene: mesh_assets.plum.clone(),
                    transform: Transform::from_xyz(5.0, 0.0, 0.0),
                    ..default()
                },
                Propagate(PlumUnit),
            ));
        }
        let mut selected_index = None;
        let mut iter = plum.iter_mut();
        if let Some((_transitions, anim_indices, _plum_unit, _anim_player)) = iter.next() {
            for (name, anim_index) in anim_indices.iter() {
                if ui.button(name).clicked() {
                    selected_index = Some(*anim_index);
                }
            }
        }
        if let Some(selected_index) = selected_index {
            for (i, (mut transitions, _anim_indices, _plum_unit, mut anim_player)) in
                plum.iter_mut().enumerate()
            {
                transitions
                    .play(
                        &mut anim_player,
                        selected_index,
                        Duration::from_secs_f32(i as f32 * 0.1 + 1.0),
                    )
                    .repeat()
                    .set_speed(i as f32 * 0.1 + 1.0);
            }
        }
    });
}
