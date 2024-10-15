use bevy::{prelude::*, utils::hashbrown::HashMap};

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
