use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use kira::tween::Tween;

use crate::minimal_kira_audio::{KiraSoundData, KiraSoundHandle, MinimalKiraPlugin};

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
        app.add_plugins(MinimalKiraPlugin)
            .add_systems(PostUpdate, run_spatial_audio);
    }
}

#[derive(Component, Clone, Copy)]
pub struct GameAudioReceiver;

#[derive(Component, Clone)]
pub struct AudioEmitter {
    pub handle: Handle<KiraSoundHandle>,
    /// Gain offset (post clamp)
    pub gain_db: f32,
    /// Above this distance the volume won't decrease any more
    pub max_distance: f32,
    /// Below this distance the volume won't increase any more
    pub min_distance: f32,
    /// Faster more realistic falloff
    pub inv_square_falloff: bool,
    /// Within this radius the effect of the panning is reduced
    pub size: f32,
}

#[derive(Component, Clone)]
pub struct AudioEmitterSet(pub Vec<AudioEmitter>);

impl Default for AudioEmitter {
    fn default() -> Self {
        Self {
            gain_db: 0.0,
            max_distance: 1000.0,
            min_distance: 1.0,
            inv_square_falloff: false,
            size: 0.2,
            handle: Default::default(),
        }
    }
}

pub fn db_to_lin(x: f32) -> f32 {
    10.0f32.powf(x * 0.05)
}

pub fn lin_to_db(x: f32) -> f32 {
    (x.max(0.0)).log10() * 20.0
}

pub fn run_spatial_audio(
    receiver: Query<&GlobalTransform, With<GameAudioReceiver>>,
    mut emitters: Query<(
        &GlobalTransform,
        Option<&AudioEmitter>,
        Option<&AudioEmitterSet>,
    )>,
    mut audio_instances: ResMut<Assets<KiraSoundHandle>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        for (emitter_transform, single_emit, emit_set) in &mut emitters {
            if let Some(emit) = single_emit {
                process(
                    emitter_transform,
                    receiver_transform,
                    emit,
                    &mut audio_instances,
                );
            }
            if let Some(set) = &emit_set {
                for emit in &set.0 {
                    process(
                        emitter_transform,
                        receiver_transform,
                        emit,
                        &mut audio_instances,
                    );
                }
            }
        }
    }
}

fn process(
    emitter_transform: &GlobalTransform,
    receiver_transform: &GlobalTransform,
    emit_params: &AudioEmitter,
    audio_instances: &mut Assets<KiraSoundHandle>,
) {
    let rx_to_emit = emitter_transform.translation() - receiver_transform.translation();
    let distance = rx_to_emit
        .length()
        .clamp(emit_params.min_distance, emit_params.max_distance);
    let falloff = if emit_params.inv_square_falloff {
        distance.powi(2)
    } else {
        distance
    };
    let volume = (1.0 / (1.0 + falloff)).clamp(0., 1.) * db_to_lin(emit_params.gain_db);

    let mut panning = receiver_transform
        .right()
        .dot(rx_to_emit.normalize_or_zero());

    let damp_pan = if emit_params.size != 0.0 {
        1.0 - ((emit_params.size - distance) / emit_params.size).clamp(0.0, 1.0)
    } else {
        1.0
    };

    panning *= damp_pan;

    if let Some(instance) = audio_instances.get_mut(&emit_params.handle) {
        instance.0.set_volume(volume as f64, Tween::default());
        instance
            .0
            .set_panning((panning * 0.5 + 0.5) as f64, Tween::default());
    }
}
