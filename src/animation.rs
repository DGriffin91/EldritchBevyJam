use std::time::Duration;

use bevy::{animation::ActiveAnimation, prelude::*, utils::hashbrown::HashMap};

use crate::mesh_assets::MeshAssets;

#[derive(Component, Clone, Deref, DerefMut)]
// TODO use enum or Handle<AnimationClip> or something once this is more figured out
pub struct AnimationIndices(pub HashMap<String, AnimationNodeIndex>);

pub trait AnimClips {
    fn get_gltf_id(&self, mesh_assets: &MeshAssets) -> Handle<Gltf>
    where
        Self: Sized;
}

pub fn init_animation_graph<T: AnimClips + Component>(
    mut commands: Commands,
    mut players: Query<(Entity, &T), Added<AnimationPlayer>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    gltf_assets: ResMut<Assets<Gltf>>,
    mesh_assets: Res<MeshAssets>,
) {
    for (entity, anim) in &mut players {
        let mut anim_indices = HashMap::new();
        let mut animation_graph = AnimationGraph::new();
        let gltf = gltf_assets.get(&anim.get_gltf_id(&mesh_assets)).unwrap();
        for (name, clip_handle) in gltf.named_animations.iter() {
            anim_indices.insert(
                name.to_string(),
                animation_graph.add_clip(clip_handle.clone(), 1.0, animation_graph.root),
            );
        }

        let graph_handle = animation_graphs.add(animation_graph);
        let transitions = AnimationTransitions::new();
        commands
            .entity(entity)
            .insert((transitions, graph_handle, AnimationIndices(anim_indices)));
    }
}

/// Wraps the relevant animation components into a simpler interface
pub struct AnimPlayerController<'a> {
    pub transitions: &'a mut AnimationTransitions,
    pub player: &'a mut AnimationPlayer,
    pub anim: &'a AnimationIndices,
}

impl<'a> AnimPlayerController<'a> {
    pub fn new(
        transitions: &'a mut AnimationTransitions,
        player: &'a mut AnimationPlayer,
        anim: &'a AnimationIndices,
    ) -> Self {
        AnimPlayerController {
            transitions,
            player,
            anim,
        }
    }

    pub fn play(&mut self, name: &str, transition: f32, speed: f32, repeat: bool) {
        self.play_idx(
            *self
                .anim
                .get(name)
                .unwrap_or_else(|| panic!("{}", self.no_name_msg(name))),
            transition,
            speed,
            repeat,
        )
    }

    pub fn play_idx(&mut self, idx: AnimationNodeIndex, transition: f32, speed: f32, repeat: bool) {
        let a = self
            .transitions
            .play(self.player, idx, Duration::from_secs_f32(transition))
            .set_speed(speed);
        if repeat {
            a.repeat();
        }
    }

    pub fn playing(&mut self, name: &str) -> bool {
        self.player.is_playing_animation(
            *self
                .anim
                .get(name)
                .unwrap_or_else(|| panic!("{}", self.no_name_msg(name))),
        )
    }

    fn no_name_msg(&self, name: &str) -> String {
        format!(
            "Couldn't find animation with name: {}. Options: {:?}",
            name,
            self.anim.iter().map(|(_, v)| v)
        )
    }

    pub fn playing_idx(&mut self, idx: AnimationNodeIndex) -> bool {
        self.player.is_playing_animation(idx)
    }

    pub fn animation(&mut self, name: &str) -> std::option::Option<ActiveAnimation> {
        self.player
            .animation(
                *self
                    .anim
                    .get(name)
                    .unwrap_or_else(|| panic!("{}", self.no_name_msg(name))),
            )
            .copied()
    }

    pub fn animation_idx(
        &mut self,
        idx: AnimationNodeIndex,
    ) -> std::option::Option<ActiveAnimation> {
        self.player.animation(idx).copied()
    }
}

pub fn ramp_up_down_anim(seek_f: f32, move_start: f32, move_length: f32, pow_curve: f32) -> f32 {
    let mut mspeed = (seek_f - move_start) / move_length;
    mspeed = 1.0 - (mspeed * 2.0 - 1.0).abs();
    mspeed = mspeed.powf(pow_curve);
    mspeed
}
