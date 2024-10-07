use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct MeshAssets {
    #[asset(path = "temp/animated/Fox.glb#Scene0")]
    pub fox: Handle<Scene>,
    #[asset(path = "temp/panStew.glb#Scene0")]
    pub pan_stew: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct AnimationAssets {
    #[asset(path = "temp/animated/Fox.glb#Animation0")]
    pub fox_0: Handle<AnimationClip>,
    #[asset(path = "temp/animated/Fox.glb#Animation1")]
    pub fox_1: Handle<AnimationClip>,
    #[asset(path = "temp/animated/Fox.glb#Animation2")]
    pub fox_2: Handle<AnimationClip>,
}
