use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct MeshAssets {
    #[asset(path = "temp/animated/Fox.glb")]
    pub fox_gltf: Handle<Gltf>,
    #[asset(path = "temp/animated/Fox.glb#Scene0")]
    pub fox: Handle<Scene>,
    #[asset(path = "temp/panStew.glb#Scene0")]
    pub pan_stew: Handle<Scene>,

    #[asset(path = "temp/animated/Stabby_Enemy.gltf")]
    pub plum_gltf: Handle<Gltf>,
    #[asset(path = "temp/animated/Stabby_Enemy.gltf#Scene0")]
    pub plum: Handle<Scene>,

    #[asset(path = "temp/animated/Stinger_Enemy.gltf")]
    pub spider_gltf: Handle<Gltf>,
    #[asset(path = "temp/animated/Stinger_Enemy.gltf#Scene0")]
    pub spider: Handle<Scene>,

    #[asset(path = "temp/level_start.gltf#Scene0")]
    pub level_start: Handle<Scene>,
    #[asset(path = "temp/level_c.gltf#Scene0")]
    pub level_c: Handle<Scene>,
    #[asset(path = "temp/starting_level.gltf#Scene0")]
    pub starting_level: Handle<Scene>,
}
