use anyhow::Result;
use bevy::asset::io::Reader;
use bevy::asset::Asset;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use kira::manager::{AudioManager, AudioManagerSettings, DefaultBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::sound::FromFileError;
use kira::track::TrackHandle;
use std::io::Cursor;
use thiserror::Error;

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

/// Possible errors that can be produced by [`OggLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OggLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// An Error loading sound from a file. See [`FromFileError`]
    #[error("Error while loading a sound: {0}")]
    FileError(#[from] FromFileError),
}

/// Asset loader for OGG files.
#[derive(Default)]
pub struct OggLoader;

impl AssetLoader for OggLoader {
    type Asset = KiraSoundData;
    type Settings = ();
    type Error = OggLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut sound_bytes = vec![];
        reader.read_to_end(&mut sound_bytes).await?;
        let sound = StaticSoundData::from_cursor(Cursor::new(sound_bytes))?;
        Ok(KiraSoundData(sound))
    }

    fn extensions(&self) -> &[&str] {
        &["ogg", "oga", "spx"]
    }
}

// ----------------------------------------------------

/// Possible errors that can be produced by [`FlacLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FlacLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// An Error loading sound from a file. See [`FromFileError`]
    #[error("Error while loading a sound: {0}")]
    FileError(#[from] FromFileError),
}

/// Asset loader for FLAC files.
#[derive(Default)]
pub struct FlacLoader;

impl AssetLoader for FlacLoader {
    type Asset = KiraSoundData;
    type Settings = ();
    type Error = FlacLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut sound_bytes = vec![];
        reader.read_to_end(&mut sound_bytes).await?;
        let sound = StaticSoundData::from_cursor(Cursor::new(sound_bytes))?;
        Ok(KiraSoundData(sound))
    }

    fn extensions(&self) -> &[&str] {
        &["flac"]
    }
}
