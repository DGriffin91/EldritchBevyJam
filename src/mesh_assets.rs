use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::AssetFileName};

#[derive(AssetCollection, Resource)]
pub struct MeshAssets {
    #[asset(path = "temp/animated", collection(mapped, typed))]
    pub gltfs: HashMap<AssetFileName, Handle<Gltf>>,

    #[asset(path = "temp/animated/Fox.glb")]
    pub fox_gltf: Handle<Gltf>,
    #[asset(path = "temp/animated/Fox.glb#Scene0")]
    pub fox: Handle<Scene>,
    #[asset(path = "temp/panStew.glb#Scene0")]
    pub pan_stew: Handle<Scene>,
}
