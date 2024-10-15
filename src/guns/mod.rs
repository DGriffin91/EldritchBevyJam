use bevy::{math::Vec3A, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_egui::EguiContexts;
use bevy_fps_controller::controller::RenderPlayer;

use crate::{character_controller::manage_cursor, GameLoading};

#[derive(AssetCollection, Resource)]
pub struct GunSceneAssets {
    #[asset(path = "models/guns/lmg.gltf#Scene0")]
    pub lmg: Handle<Scene>,
    #[asset(path = "models/guns/lmg_bullet.gltf#Scene0")]
    pub lmg_bullet: Handle<Scene>,
    #[asset(path = "models/guns/lmg_bullet_jacket.gltf#Scene0")]
    pub lmg_bullet_jacket: Handle<Scene>,
    #[asset(path = "models/guns/muzzle_flash.gltf#Scene0")]
    pub muzzle_flash: Handle<Scene>,
}

pub struct GunsPlugin;
impl Plugin for GunsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (position_lmg, mark_rotate_part, fire_gun)
                .run_if(in_state(GameLoading::Loaded))
                .after(manage_cursor),
        )
        .add_systems(Update, update_bullet.run_if(in_state(GameLoading::Loaded)))
        .add_systems(OnEnter(GameLoading::Loaded), spawn_gun);
    }
}

#[derive(Component)]
pub struct GunLMG;

#[derive(Component)]
pub struct LMGMuzzleFlashLight;
#[derive(Component)]
pub struct LMGMuzzleFlashMesh;

#[derive(Component, Default)]
pub struct LMGRotateyBoi {
    rotate_speed: f32,
}

fn mark_rotate_part(
    mut commands: Commands,
    entities: Query<(Entity, &Transform, &Name)>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    for (entity, _trans, name) in &entities {
        if name.contains("BARREL") {
            commands.entity(entity).insert(LMGRotateyBoi::default());
            *done = true;
            dbg!("gun_trans");
            return;
        }
    }
}

fn spawn_gun(mut commands: Commands, gun_assets: Res<GunSceneAssets>) {
    commands
        .spawn((
            SceneBundle {
                scene: gun_assets.lmg.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            GunLMG,
        ))
        .with_children(|cmd| {
            cmd.spawn((
                PointLightBundle {
                    point_light: PointLight {
                        shadows_enabled: false,
                        intensity: 100000.0,
                        //inner_angle: 0.1,
                        //outer_angle: PI / 2.0,
                        range: 100.0,
                        radius: 0.0,
                        color: Color::srgb(1.0, 0.7, 0.5),
                        ..default()
                    },
                    transform: Transform::from_xyz(-0.17, 0.17, -0.5)
                        .looking_at(-Vec3::Z * 10.0, Vec3::Y),
                    ..default()
                },
                LMGMuzzleFlashLight,
            ));
            cmd.spawn((
                SceneBundle {
                    scene: gun_assets.muzzle_flash.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, -1.2).with_scale(Vec3::splat(2.0)),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                LMGMuzzleFlashMesh,
            ));
        });
}

fn position_lmg(
    mut gun: Query<&mut Transform, With<GunLMG>>,
    player_camera: Query<&Transform, (With<RenderPlayer>, Without<GunLMG>)>,
) {
    let Ok(mut gun_trans) = gun.get_single_mut() else {
        return;
    };
    let Ok(player_cam_trans) = player_camera.get_single() else {
        return;
    };
    let player_mat = player_cam_trans.compute_matrix();

    gun_trans.rotation = player_cam_trans.rotation;
    gun_trans.translation = player_mat
        .transform_point3a(Vec3A::new(0.4, -0.2, -1.6))
        .into();
}

pub fn fire_gun(
    mut commands: Commands,
    btn: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut gun_rot: Query<(&mut Transform, &mut LMGRotateyBoi)>,
    mut gun_muzzle: Query<
        (&mut PointLight, &mut LMGMuzzleFlashLight),
        (
            Without<LMGRotateyBoi>,
            Without<GunLMG>,
            Without<LMGMuzzleFlashMesh>,
        ),
    >,
    mut gun: Query<
        &GlobalTransform,
        (
            With<GunLMG>,
            Without<LMGRotateyBoi>,
            Without<LMGMuzzleFlashLight>,
            Without<LMGMuzzleFlashMesh>,
        ),
    >,
    mut muzzle_flash_mesh: Query<
        &mut Visibility,
        (
            With<LMGMuzzleFlashMesh>,
            Without<LMGRotateyBoi>,
            Without<LMGMuzzleFlashLight>,
            Without<GunLMG>,
        ),
    >,
    time: Res<Time>,
    mut fire_ready: Local<bool>,
    gun_assets: Res<GunSceneAssets>,
) {
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }
    let Ok((mut gun_rot_trans, mut props)) = gun_rot.get_single_mut() else {
        return;
    };
    let Ok((mut gun_muzzle_light, mut _muzzle_props)) = gun_muzzle.get_single_mut() else {
        return;
    };
    let Ok(gun_global_trans) = gun.get_single_mut() else {
        return;
    };
    let Ok(mut muzzle_flash_mesh_vis) = muzzle_flash_mesh.get_single_mut() else {
        return;
    };

    let dt = time.delta_seconds();
    let t = time.elapsed_seconds();
    let max_rotate_speed = 20.0;
    let ramp_up_speed = 1.0;
    let ramp_down_speed = 0.6;
    let flash_rate = 1.0;
    gun_muzzle_light.intensity = 0.0;

    let fire_flip =
        ((t * props.rotate_speed * max_rotate_speed * flash_rate) as i32).rem_euclid(2) == 0;

    if !fire_flip {
        *fire_ready = true;
    }

    let fire_bullet_period =
        fire_flip && btn.pressed(MouseButton::Left) && props.rotate_speed >= 1.0;

    let mut fire_this_frame = false;
    if *fire_ready && fire_bullet_period {
        fire_this_frame = true;
        *fire_ready = false;
    }

    if fire_bullet_period {
        gun_muzzle_light.intensity = 500000.0;
        *muzzle_flash_mesh_vis = Visibility::Visible;
    } else {
        *muzzle_flash_mesh_vis = Visibility::Hidden;
    }

    if fire_this_frame {
        let gun_global_mat = gun_global_trans.compute_matrix(); // TODO reuse camera mat or something

        commands.spawn((
            SceneBundle {
                scene: gun_assets.lmg_bullet_jacket.clone(),
                transform: Transform::from_translation(
                    gun_global_mat
                        .transform_point3a(Vec3A::new(0.8, 0.2, -1.2))
                        .into(),
                )
                .looking_at(
                    gun_global_mat
                        .transform_point3a(Vec3A::new(0.0, 0.0, -100.0))
                        .into(),
                    Vec3::Y,
                )
                .with_scale(Vec3::splat(0.7)),
                ..default()
            },
            LMGBullet {
                velocity: gun_global_mat
                    .transform_vector3a(Vec3A::new(3.0, 6.0, 1.0))
                    .into(),
            },
        ));
    }

    // TODO make input configurable
    if btn.pressed(MouseButton::Left) {
        props.rotate_speed += dt * ramp_up_speed;
        if props.rotate_speed >= 1.0 {}
    } else {
        props.rotate_speed -= dt * ramp_down_speed;
    }
    gun_rot_trans.rotate_local_z(-dt * props.rotate_speed * max_rotate_speed);
    props.rotate_speed = props.rotate_speed.clamp(0.0, 1.0);
}

#[derive(Component)]
pub struct LMGBullet {
    velocity: Vec3,
}
pub fn update_bullet(
    //mut commands: Commands,
    mut bullets: Query<(Entity, &mut LMGBullet, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (_entity, mut bullet, mut trans) in &mut bullets {
        if trans.translation.y < 0.1 {
            bullet.velocity.y = 0.0;
            bullet.velocity.x *= 0.993;
            bullet.velocity.z *= 0.993;
            if bullet.velocity.y == 0.0 {
                trans.rotate_y(3.0 * dt * bullet.velocity.x * bullet.velocity.z);
            } else {
                trans.rotation = Default::default();
            }
            //commands.entity(entity).despawn_recursive(); // Despawn after a while
        } else {
            bullet.velocity -= Vec3::Y * 13.0 * dt; // Gravity or whatever
        }
        trans.translation += bullet.velocity * dt;
        if bullet.velocity.y > 0.0 {
            trans.rotate_local_y(-10.0 * dt);
            trans.rotate_local_x(-5.0 * dt);
        }
    }
}
