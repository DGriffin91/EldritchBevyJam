pub mod flac_loader;
pub mod mp3_loader;
pub mod ogg_loader;

use crate::minimal_kira_audio::{
    flac_loader::FlacLoader, mp3_loader::Mp3Loader, ogg_loader::OggLoader,
};
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use kira::manager::{AudioManager, AudioManagerSettings, DefaultBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::track::TrackHandle;

/// Controls audio from gameplay code.
#[derive(Resource, Deref, DerefMut)]
pub struct KiraAudioManager(AudioManager);

// A piece of audio loaded into memory all at once.
// These can be cheaply cloned, as the audio data is shared among all clones.
#[derive(Clone, Asset, TypePath, Deref, DerefMut)]
pub struct KiraSoundData(pub StaticSoundData);

/// Controls a static sound.
#[derive(Asset, bevy::reflect::TypePath)]
pub struct KiraSoundHandle(pub StaticSoundHandle);

/// Holds a handle to a kira track.
/// StaticSoundData instances can be assigned to a track.
/// A track can control processing/mixing.
/// If TrackHandle `drop`s any audio assigned to that track will stop.
#[derive(Asset, bevy::reflect::TypePath)]
pub struct KiraTrackHandle(pub TrackHandle);

pub struct MinimalKiraPlugin;
impl Plugin for MinimalKiraPlugin {
    fn build(&self, app: &mut App) {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
        app.insert_resource(KiraAudioManager(manager))
            .init_asset_loader::<FlacLoader>()
            .init_asset_loader::<OggLoader>()
            .init_asset_loader::<Mp3Loader>()
            .init_asset::<KiraSoundData>()
            .init_asset::<KiraSoundHandle>()
            .init_asset::<KiraTrackHandle>();
    }
}

pub fn db_to_lin(x: f32) -> f32 {
    10.0f32.powf(x * 0.05)
}

pub fn lin_to_db(x: f32) -> f32 {
    (x.max(0.0)).log10() * 20.0
}

pub fn sound_data(
    sounds: &Assets<KiraSoundData>,
    handle: &Handle<KiraSoundData>,
) -> StaticSoundData {
    sounds.get(handle).unwrap().0.clone()
}
