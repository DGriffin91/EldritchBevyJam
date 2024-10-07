use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use spatial::SpatialAudioPlugin;

use crate::minimal_kira_audio::{KiraSoundData, MinimalKiraPlugin};

pub mod spatial;

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "temp/audio/cooking.ogg")]
    pub cooking: Handle<KiraSoundData>,
    #[asset(path = "temp/audio/loop.ogg")]
    pub music: Handle<KiraSoundData>,
}

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MinimalKiraPlugin, SpatialAudioPlugin));
    }
}
