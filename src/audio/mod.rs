use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use spatial::SpatialAudioPlugin;

use crate::minimal_kira_audio::{KiraSoundData, MinimalKiraPlugin};

pub mod spatial;

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/theme3.flac")]
    pub game_music: Handle<KiraSoundData>,
    #[asset(path = "audio/gun.flac")]
    pub gun: Handle<KiraSoundData>,
}

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MinimalKiraPlugin, SpatialAudioPlugin));
    }
}
