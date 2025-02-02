use bevy::core_pipeline::prepass::{DeferredPrepass, DepthPrepass};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_egui::EguiContexts;
use bevy_rapier3d::prelude::*;
use bs13::bs13_render::cmaa::Cmaa;
use bs13::bs13_render::ssao::Ssao;
use bs13::bs13_render::taa::TaaBundle;
use bs13::bs13_render::GpuCull;
use bs13_render::frame_pyramid::FramePyramid;
use bs13_render::ssr::Ssr;
use bs13_render::{BS13ViewTargetSettings, DepthPrepassForDeferred};
use fps_controller::{
    CameraConfig, FpsController, FpsControllerInput, FpsControllerPlugin, LogicalPlayer,
    RenderPlayer,
};
use std::f32::consts::TAU;

use crate::audio::spatial::GameAudioReceiver;
use crate::fps_controller;

pub struct CharacterController;
impl Plugin for CharacterController {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsControllerPlugin)
            .add_systems(Startup, spawn_player)
            .add_systems(Update, manage_cursor);
    }
}

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.625, -100.0);

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Note that we have two entities for the player
    // One is a "logical" player that handles the physics computation and collision
    // The other is a "render" player that is what is displayed to the user
    // This distinction is useful for later on if you want to add multiplayer,
    // where often time these two ideas are not exactly synced up
    let height = 1.7;
    let logical_entity = commands
        .spawn((
            Collider::cylinder(height / 2.0, 0.5),
            // A capsule can be used but is NOT recommended
            // If you use it, you have to make sure each segment point is
            // equidistant from the translation of the player transform
            //Collider::capsule_y(height / 2.0, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            LogicalPlayer,
            FpsControllerInput {
                pitch: TAU * 0.0,
                yaw: TAU * 0.0,
                ..default()
            },
            FpsController {
                max_air_speed: 60.0,
                air_acceleration: 150.0,
                jump_speed: 15.0,
                run_speed: 50.0,
                walk_speed: 8.0,
                upright_height: height,
                crouch_height: height * 0.5,
                //gravity: 0.0,
                ..default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.1,
        })
        .id();

    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.7, 0.7, 1.0)
                    .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: TAU / 6.0,
                    ..default()
                }),
                ..default()
            },
            EnvironmentMapLight {
                diffuse_map: asset_server
                    .load("environment_maps/kloofendal_28d_misty_puresky_2k_diffuse.ktx2"),
                specular_map: asset_server
                    .load("environment_maps/kloofendal_28d_misty_puresky_2k_specular.ktx2"),
                intensity: 50.0,
            },
            Cmaa::default(),
            TaaBundle::sample4(),
            DepthPrepass,
            DeferredPrepass,
            Ssao,
            Ssr,
            GpuCull {
                frustum: false, // CPU is culling and we use no frustum culling on some things
                occlusion: true,
            },
            DepthPrepassForDeferred {
                screen_ratio_threshold: None,
                include_alpha_mask: false,
            },
            RenderPlayer { logical_entity },
            FramePyramid,
            Player::default(),
            BS13ViewTargetSettings { render_scale: 1.0 },
        ))
        .insert(GameAudioReceiver);
}

#[derive(Component, Clone, Copy)]
pub struct Player {
    pub activity_start_time: Option<f32>,
    pub health: f32,
    pub kills: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            activity_start_time: None,
            health: 100.0,
            kills: 0,
        }
    }
}

pub fn manage_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut fps_controller: Query<&mut FpsController>,
    btn: Res<ButtonInput<MouseButton>>,
    //#[cfg(debug_assertions)] editor_state: Res<EditorState>,
    mut windows: Query<&mut Window>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }
    let mut window = windows.single_mut();
    let mut fps_controller = fps_controller.single_mut();
    let cursor_locked = window.cursor.grab_mode == CursorGrabMode::Locked;
    let mut lock = None;
    if keys.just_pressed(KeyCode::Tab) {
        lock = Some(!cursor_locked);
    }
    if keys.just_pressed(KeyCode::Escape) || (!cursor_locked && fps_controller.enable_input) {
        // Unlock
        lock = Some(false);
    }

    #[allow(unused_assignments, unused_mut)]
    let mut editor_active = false;

    //#[cfg(debug_assertions)]
    //{
    //    editor_active = editor_state.active;
    //}

    if btn.just_pressed(MouseButton::Left)
        && (!fps_controller.enable_input || window.cursor.visible || !cursor_locked)
        && !editor_active
    {
        // Lock
        lock = Some(true);
    }

    if let Some(lock) = lock {
        if lock {
            // Lock
            fps_controller.enable_input = true;
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        } else {
            // Unlock
            fps_controller.enable_input = false;
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
    #[cfg(not(target_os = "macos"))]
    if cursor_locked {
        let (w, h) = (window.width(), window.height());
        window.set_cursor_position(Some(vec2(w / 2.0, h / 2.0)));
    }
}
