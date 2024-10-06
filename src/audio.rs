use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_kira_audio::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "temp/audio/cooking.ogg")]
    pub cooking: Handle<bevy_kira_audio::prelude::AudioSource>,
}

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .insert_resource(GameSpatialAudio { max_distance: 25. })
            .add_systems(
                PostUpdate,
                run_spatial_audio.run_if(resource_exists::<GameSpatialAudio>),
            );
    }
}

#[derive(Component, Clone, Copy)]
pub struct GameAudioReceiver;

#[derive(Component, Clone, Copy)]
pub struct GameAudioEmitterParams {
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

impl Default for GameAudioEmitterParams {
    fn default() -> Self {
        Self {
            gain_db: 0.0,
            max_distance: 1000.0,
            min_distance: 1.0,
            inv_square_falloff: false,
            size: 0.2,
        }
    }
}

pub fn db_to_lin(x: f32) -> f32 {
    10.0f32.powf(x * 0.05)
}

pub fn lin_to_db(x: f32) -> f32 {
    (x.max(0.0)).log10() * 20.0
}

#[derive(Resource)]
pub struct GameSpatialAudio {
    /// The volume will change from `1` at distance `0` to `0` at distance `max_distance`
    pub max_distance: f32,
}

pub fn run_spatial_audio(
    _spatial_audio: Res<GameSpatialAudio>,
    receiver: Query<&GlobalTransform, With<GameAudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitter, &GameAudioEmitterParams)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Ok(receiver_transform) = receiver.get_single() {
        for (emitter_transform, emitter, emit_params) in &emitters {
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

            for instance in emitter.instances.iter() {
                if let Some(instance) = audio_instances.get_mut(instance) {
                    instance.set_volume(volume as f64, AudioTween::default());
                    instance.set_panning((panning * 0.5 + 0.5) as f64, AudioTween::default());
                }
            }
        }
    }
}
