// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::borrow::Cow;
use std::f32::consts::PI;

use audio::{AudioAssets, GameAudioPlugin};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::system::EntityCommands;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::render::RenderPlugin;
use bevy::window::{PresentMode, WindowResolution};
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_asset_loader::loading_state::config::ConfigureLoadingState;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
// use bevy_mod_mipmap_generator::{generate_mipmaps, MipmapGeneratorPlugin, MipmapGeneratorSettings};

use bevy_egui::{egui, EguiContexts};

use bs13::bs13_render::taa::BS13TaaPlugin;
use bs13::bs13_render::BS13StandardMaterialPluginsSet;
use bs13_egui::BS13EguiPlugin;
use character_controller::CharacterController;
use eldritch_game::audio::spatial::{AudioEmitter, AudioEmitterSet};
use eldritch_game::fps_controller::LogicalPlayer;
use eldritch_game::guns::{GunSceneAssets, GunsPlugin};
use eldritch_game::mesh_assets::MeshAssets;
use eldritch_game::physics::{AddCuboidColliders, AddCuboidSensors};
use eldritch_game::units::UnitsPlugin;
use eldritch_game::util::{propagate_to_name, PropagateToName};
use eldritch_game::{
    audio, character_controller, minimal_kira_audio, physics, GameLoading, LEVEL_TRANSITION_HEIGHT,
};
use iyes_progress::ProgressPlugin;
use kira::effect::reverb::ReverbBuilder;
use kira::track::TrackBuilder;
use kira::tween::Tween;
use minimal_kira_audio::{
    sound_data, KiraAudioManager, KiraSoundData, KiraSoundHandle, KiraTrackHandle,
};
use physics::{AddTrimeshPhysics, PhysicsStuff};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Off)
        //.insert_resource(ClearColor(Color::srgb(0.1, 0.03, 0.03)))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight::NONE)
        // TODO include compressed textures?
        //.insert_resource(MipmapGeneratorSettings {
        //    anisotropic_filtering: 16,
        //    compression: Some(Default::default()),
        //    compressed_image_data_cache_path: Some(PathBuf::from("cache")),
        //    low_quality: true,
        //    ..default()
        //})
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
                        present_mode: PresentMode::AutoVsync,
                        resolution: WindowResolution::new(1920.0, 1080.0)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                }),
            BS13StandardMaterialPluginsSet,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            //MipmapGeneratorPlugin,
            BS13EguiPlugin,
            BS13TaaPlugin,
            PhysicsStuff,
            CharacterController,
        ));

    app.add_plugins((GameAudioPlugin, UnitsPlugin, GunsPlugin));

    app.init_state::<GameLoading>()
        .add_plugins(ProgressPlugin::new(GameLoading::AssetLoading))
        .add_loading_state(
            LoadingState::new(GameLoading::AssetLoading)
                .continue_to_state(GameLoading::Loaded)
                .load_collection::<AudioAssets>()
                .load_collection::<MeshAssets>()
                .load_collection::<GunSceneAssets>(),
        );

    app.add_systems(Startup, setup)
        .add_systems(OnEnter(GameLoading::Loaded), level_c)
        .add_systems(
            Update,
            (
                propagate_to_name::<PlayerStart>,
                hide_start_level,
                crosshair,
                move_player_to_start,
            )
                .run_if(in_state(GameLoading::Loaded)),
        )
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    //commands
    //    .spawn(SceneBundle {
    //        scene: asset_server.load("temp/init_test_scene.gltf#Scene0"),
    //        ..default()
    //    })
    //    .insert(PropagateToName(
    //        AddTrimeshPhysics,
    //        Cow::Borrowed("COLLIDER"),
    //    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            PI * -0.6,
            PI * 0.3,
            0.0,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 8500.0,
            shadow_depth_bias: 0.8,
            shadow_normal_bias: 0.8,
            color: Color::srgb(1.0, 0.4, 0.0),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 700.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn level_c(mut commands: Commands, mesh_assets: Res<MeshAssets>) {
    add_level_props(commands.spawn((SceneBundle {
        scene: mesh_assets.level_c.clone(),
        ..default()
    },)));
    add_level_props(commands.spawn((
        StartLevel,
        SceneBundle {
            scene: mesh_assets.starting_level.clone(),
            ..default()
        },
    )));
    add_level_props(commands.spawn((
        StartLevel,
        SceneBundle {
            scene: mesh_assets.level_start.clone(),
            ..default()
        },
    )));
}

fn add_level_props(mut ecmds: EntityCommands) {
    ecmds.insert((
        PropagateToName(AddTrimeshPhysics, Cow::Borrowed("COLLIDER")),
        PropagateToName(AddCuboidColliders, Cow::Borrowed("COLLIDER")),
        PropagateToName(AddCuboidSensors, Cow::Borrowed("SENSOR")),
        PropagateToName(PlayerStart, Cow::Borrowed("PLAYER_START")),
    ));
}

fn move_player_to_start(
    mut player: Query<&mut Transform, With<LogicalPlayer>>,
    start: Query<(&mut Transform, &PlayerStart), Without<LogicalPlayer>>,
    mut has_run: Local<bool>,
) {
    if *has_run {
        return;
    }
    let Ok(mut player_trans) = player.get_single_mut() else {
        return;
    };
    for (start_trans, _start) in start.iter() {
        player_trans.translation = start_trans.translation;
        player_trans.look_to(-start_trans.forward(), Vec3::Y);
        *has_run = true;
    }
}

#[derive(Component, Clone)]
pub struct PlayerStart;

fn hide_start_level(
    //mut commands: Commands,
    player: Query<&Transform, With<Camera3d>>,
    mut start_level_items: Query<(Entity, &mut Visibility), With<StartLevel>>,
    mut has_run: Local<bool>,
) {
    if *has_run {
        return;
    }
    let Ok(player) = player.get_single() else {
        return;
    };

    if player.translation.y < LEVEL_TRANSITION_HEIGHT {
        for (_entity, mut vis) in &mut start_level_items {
            *vis = Visibility::Hidden;
        }
        *has_run = true;
    }
}

#[derive(Component)]
struct StartLevel;

#[derive(Resource)]
pub struct CookingTrack {
    pub handle: Handle<KiraTrackHandle>,
}

fn start_cooking(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mesh_assets: Res<MeshAssets>,
    sounds: Res<Assets<KiraSoundData>>,
    mut tracks: ResMut<Assets<KiraTrackHandle>>,
    mut manager: ResMut<KiraAudioManager>,
    mut audio_instances: ResMut<Assets<KiraSoundHandle>>,
) {
    let mut track = manager
        .add_sub_track({
            let mut builder = TrackBuilder::new();
            builder.add_effect(ReverbBuilder::new());
            builder
        })
        .unwrap();

    let mut cooking = manager
        .play(sound_data(&sounds, &audio_assets.cooking).output_destination(&track))
        .unwrap();
    cooking.set_loop_region(..);

    let mut music = manager
        .play(sound_data(&sounds, &audio_assets.music).output_destination(&track))
        .unwrap();
    music.set_loop_region(..);

    track.set_volume(1.0, Tween::default());

    commands.insert_resource(CookingTrack {
        handle: tracks.add(KiraTrackHandle(track)),
    });

    commands
        .spawn(SceneBundle {
            scene: mesh_assets.pan_stew.clone(),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        })
        .insert(AudioEmitterSet(vec![
            AudioEmitter {
                handle: audio_instances.add(KiraSoundHandle(cooking)),
                gain_db: 0.0,
                ..default()
            },
            AudioEmitter {
                handle: audio_instances.add(KiraSoundHandle(music)),
                gain_db: -10.0,
                ..default()
            },
        ]));
}

fn crosshair(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let size = ctx.available_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    let crosshair = 1.0;
    let crosshair_border = 2.0;
    let mid_x = size.width() * 0.5;
    let mid_y = size.height() * 0.5;
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::Pos2::new(mid_x - crosshair_border, mid_y - crosshair_border),
            egui::Pos2::new(mid_x + crosshair_border, mid_y + crosshair_border),
        ),
        egui::Rounding::ZERO,
        egui::Color32::BLACK,
    );
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::Pos2::new(mid_x - crosshair, mid_y - crosshair),
            egui::Pos2::new(mid_x + crosshair, mid_y + crosshair),
        ),
        egui::Rounding::ZERO,
        egui::Color32::WHITE,
    );
}
