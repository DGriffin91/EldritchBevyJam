// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod audio;
pub mod character_controller;
pub mod physics;
pub mod util;

use std::f32::consts::PI;
use std::path::PathBuf;

use audio::{AudioAssets, GameAudioEmitterParams, GameAudioPlugin};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::render::RenderPlugin;
use bevy::window::{PresentMode, WindowResolution};
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_asset_loader::loading_state::config::ConfigureLoadingState;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_kira_audio::{Audio, AudioControl};
use bevy_mod_mipmap_generator::{generate_mipmaps, MipmapGeneratorPlugin, MipmapGeneratorSettings};

use bs13::bs13_render::dyn_material_blender::AllDynMaterialImagesMaterial;

use bs13::bs13_render::taa::BS13TaaPlugin;
use bs13::bs13_render::BS13StandardMaterialPluginsSet;
use bs13_egui::BS13EguiPlugin;
use character_controller::CharacterController;
use iyes_progress::ProgressPlugin;
use physics::{AddTrimeshPhysics, PhysicsStuff};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameLoading {
    #[default]
    AssetLoading,
    Loaded,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Off)
        .insert_resource(AmbientLight::NONE)
        // TODO include compressed textures?
        .insert_resource(MipmapGeneratorSettings {
            anisotropic_filtering: 16,
            compression: Some(Default::default()),
            compressed_image_data_cache_path: Some(PathBuf::from("cache")),
            low_quality: true,
            ..default()
        })
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_plugins((
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        backends: None,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // TODO make present mode option?
                        // TODO full screen?
                        present_mode: PresentMode::Immediate,
                        resolution: WindowResolution::new(1920.0, 1080.0)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                }),
            BS13StandardMaterialPluginsSet,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            MipmapGeneratorPlugin,
            BS13EguiPlugin,
            BS13TaaPlugin,
            PhysicsStuff,
            CharacterController,
            GameAudioPlugin,
        ));

    app.init_state::<GameLoading>()
        .add_plugins(ProgressPlugin::new(GameLoading::AssetLoading))
        .add_loading_state(
            LoadingState::new(GameLoading::AssetLoading)
                .continue_to_state(GameLoading::Loaded)
                .load_collection::<AudioAssets>(),
        )
        .add_systems(OnEnter(GameLoading::Loaded), start_cooking);

    app.add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                generate_mipmaps::<AllDynMaterialImagesMaterial>,
                //ui_example_system,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("temp/init_test_scene.gltf#Scene0"),
            ..default()
        })
        .insert(AddTrimeshPhysics);

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            PI * -0.15,
            PI * 0.13,
            0.0,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 40.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn start_cooking(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    // Emitter Nr. 1
    let cooking = audio.play(audio_assets.cooking.clone()).looped().handle();
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("temp/panStew.glb#Scene0"),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        })
        .insert((
            bevy_kira_audio::prelude::AudioEmitter {
                instances: vec![cooking],
            },
            GameAudioEmitterParams {
                gain_db: 0.0,
                ..default()
            },
        ));
}
