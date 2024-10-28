use std::{
    borrow::Cow,
    f32::consts::{PI, TAU},
};

use bevy::{core::FrameCount, math::*, prelude::*, render::view::NoFrustumCulling};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_egui::EguiContexts;

use crate::{
    character_controller::manage_cursor,
    fps_controller::RenderPlayer,
    hash_noise,
    menu::UserSettings,
    mesh_assets::MeshAssets,
    units::{plum::PlumUnit, spider::SpiderUnit},
    util::{propagate_to_name, PropagateDefault, PropagateToName},
    GameLoading, ShaderCompSpawn, LEVEL_TRANSITION_HEIGHT,
};

#[derive(AssetCollection, Resource)]
pub struct GunSceneAssets {
    #[asset(path = "models/guns/lmg.gltf#Scene0")]
    pub lmg: Handle<Scene>,
    #[asset(path = "models/guns/lmg_bullet.gltf#Scene0")]
    pub lmg_bullet: Handle<Scene>,
    #[asset(path = "models/guns/lmg_bullet_jacket.gltf#Scene0")]
    pub lmg_bullet_jacket: Handle<Scene>,
}

pub struct GunsPlugin;
impl Plugin for GunsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                position_lmg,
                mark_rotate_part,
                fire_gun,
                update_blood_splatter,
            )
                .run_if(in_state(GameLoading::Loaded))
                .after(manage_cursor),
        )
        .add_systems(Update, update_bullet.run_if(in_state(GameLoading::Loaded)))
        .add_systems(Update, propagate_to_name::<LMGMuzzleFlashMesh>)
        .add_systems(
            OnEnter(GameLoading::Loaded),
            (shadercomp_gun_misc, spawn_gun),
        );
    }
}

#[derive(Component, Default)]
pub struct GunLMG {
    offset: Vec3,
}

#[derive(Component)]
pub struct LMGMuzzleFlashLight;
#[derive(Component, Clone)]
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
            GunLMG::default(),
            PropagateToName(LMGMuzzleFlashMesh, Cow::Borrowed("MUZZLE_FLASH")),
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
        });
}

fn position_lmg(
    mut gun: Query<(&mut Transform, &mut GunLMG)>,
    player_camera: Query<&Transform, (With<RenderPlayer>, Without<GunLMG>)>,
    time: Res<Time>,
) {
    let Ok((mut gun_trans, mut gun)) = gun.get_single_mut() else {
        return;
    };
    let Ok(player_cam_trans) = player_camera.get_single() else {
        return;
    };
    let player_mat = player_cam_trans.compute_matrix();
    gun.offset = gun
        .offset
        .lerp(Vec3::ZERO, (time.delta_seconds() * 10.0).clamp(0.0, 1.0));
    gun_trans.rotation = player_cam_trans.rotation;

    gun_trans.translation = player_mat
        .transform_point3a(Vec3A::new(0.4, -0.2, -1.6) + Vec3A::from(gun.offset))
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
        (&mut GunLMG, &GlobalTransform),
        (
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
    mut vis_started: Local<f32>,
    mut spiders: Query<(&GlobalTransform, &mut SpiderUnit)>,
    mut plums: Query<(&GlobalTransform, &mut PlumUnit)>,
    player_camera: Query<
        &Transform,
        (
            With<RenderPlayer>,
            Without<LMGMuzzleFlashMesh>,
            Without<LMGRotateyBoi>,
            Without<LMGMuzzleFlashLight>,
            Without<GunLMG>,
        ),
    >,
    mesh_assets: Res<MeshAssets>,
    misc: (Res<FrameCount>, Res<UserSettings>),
) {
    let (frame, settings) = misc;
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }
    let Ok(player_cam_trans) = player_camera.get_single() else {
        return;
    };
    let Ok((mut gun_rot_trans, mut props)) = gun_rot.get_single_mut() else {
        return;
    };
    let Ok((mut gun_muzzle_light, mut _muzzle_props)) = gun_muzzle.get_single_mut() else {
        return;
    };
    let Ok((mut gun, gun_global_trans)) = gun.get_single_mut() else {
        return;
    };
    let Ok(mut muzzle_flash_mesh_vis) = muzzle_flash_mesh.get_single_mut() else {
        return;
    };

    let frame = frame.0;
    let t = time.elapsed_seconds();
    let dt = time.delta_seconds();
    let max_rotate_speed = 12.0;
    let min_fire_ratio = 0.0;
    let ramp_up_speed = 0.1;
    let ramp_down_speed = 0.7;
    let rotate_offset = 0.1;
    gun_muzzle_light.intensity = 0.0;

    // TODO make input configurable
    let trigger_pressed = btn.pressed(MouseButton::Left);

    if trigger_pressed {
        props.rotate_speed += dt * ramp_up_speed;
    } else {
        props.rotate_speed -= dt * ramp_down_speed;
    }
    gun_rot_trans.rotate_local_z(-dt * props.rotate_speed * max_rotate_speed);
    props.rotate_speed = props.rotate_speed.clamp(0.0, 1.0);

    let barrel_rot = gun_rot_trans.local_y().xy();
    let fac = (barrel_rot.y.atan2(barrel_rot.x) + PI) / TAU + rotate_offset;
    let b_fac = (fac * 8.0).fract();
    let fire_flip_vis = b_fac > 0.8 && b_fac < 1.0;
    let fire_flip_logic = b_fac > 0.4; // TODO make sure it fires even at low frame rates

    if !fire_flip_logic {
        *fire_ready = true;
    }

    let can_fire = trigger_pressed && props.rotate_speed >= min_fire_ratio;

    let mut fire_this_frame = false;
    if *fire_ready && fire_flip_logic && can_fire {
        fire_this_frame = true;
        *fire_ready = false;
    }

    let max_vis_time = 0.04;
    if fire_flip_vis && can_fire && t - *vis_started < max_vis_time {
        if !settings.disable_muzzle_flash {
            gun_muzzle_light.intensity = 400000.0;
            *muzzle_flash_mesh_vis = Visibility::Visible;
        }
        if *vis_started == f32::MAX {
            *vis_started = t;
        }
    } else {
        *muzzle_flash_mesh_vis = Visibility::Hidden;
    }
    if !(fire_flip_vis && can_fire) {
        *vis_started = f32::MAX;
    }

    if fire_this_frame {
        let gun_global_mat = gun_global_trans.compute_matrix();
        let rng_vel = 2.0;

        let offset_strength = 1.0 - props.rotate_speed.clamp(0.0, 1.0);
        gun.offset += vec3(
            (hash_noise(frame, 0, 0) * 2.0 - 1.0) * 0.01,
            (hash_noise(frame, 1, 0) * 2.0 - 1.0) * 0.01,
            (hash_noise(frame, 1, 0) * 2.0 - 1.0) * 0.01 + 0.2,
        ) * (offset_strength * 0.9 + 0.1);

        commands.spawn((
            SceneBundle {
                scene: gun_assets.lmg_bullet_jacket.clone(),
                transform: Transform::from_translation(
                    gun_global_mat
                        .transform_point3a(Vec3A::new(0.8, 0.2, -1.2) + Vec3A::from(gun.offset))
                        .into(),
                )
                .looking_at(
                    gun_global_mat
                        .transform_point3a(Vec3A::new(0.0, 0.0, -100.0))
                        .into(),
                    Vec3::Y,
                )
                .with_scale(Vec3::splat(0.6)),
                ..default()
            },
            LMGBullet {
                velocity: gun_global_mat
                    .transform_vector3a(Vec3A::new(
                        2.0 + hash_noise(frame, 0, 0) * rng_vel,
                        5.0 + hash_noise(frame, 1, 0) * rng_vel,
                        0.6 + hash_noise(frame, 2, 0) * rng_vel,
                    ))
                    .into(),
                floor_y: player_cam_trans.translation.y - 1.65,
            },
        ));

        let ray = obvhs::ray::Ray::new_inf(
            player_cam_trans.translation.into(),
            (*player_cam_trans.forward()).into(),
        );
        let mut hit_count = 0;
        for (unit_transform, mut unit) in &mut spiders {
            let unit_ws_trans = unit_transform.translation_vec3a();
            let aabb = obvhs::aabb::Aabb {
                min: unit_ws_trans - 1.8,
                max: unit_ws_trans + 1.8,
            };
            let t = aabb.intersect_ray(&ray);
            if t != f32::INFINITY {
                let hitp = ray.origin + ray.direction * t;
                commands.spawn((
                    SceneBundle {
                        scene: mesh_assets.blood.clone(),
                        transform: Transform::from_translation(hitp.into())
                            .looking_at(player_cam_trans.translation, Vec3::Y),
                        ..default()
                    },
                    BloodSplatter(0.0),
                ));
                hit_count += 1;
                unit.health -= 40.0;
            }
            if hit_count > 3 {
                // Only damage 3 max units
                break;
            }
        }
        let mut hit_count = 0;
        for (unit_transform, mut unit) in &mut plums {
            let unit_ws_trans = unit_transform.translation_vec3a();
            let aabb = obvhs::aabb::Aabb {
                min: unit_ws_trans - 2.1,
                max: unit_ws_trans + 2.1,
            };
            let t = aabb.intersect_ray(&ray);
            if t != f32::INFINITY {
                let hitp = ray.origin + ray.direction * t;
                commands.spawn((
                    SceneBundle {
                        scene: mesh_assets.blood.clone(),
                        transform: Transform::from_translation(hitp.into())
                            .looking_at(player_cam_trans.translation, Vec3::Y),
                        ..default()
                    },
                    BloodSplatter(0.0),
                ));
                hit_count += 1;
                unit.health -= 20.0;
            }
            if hit_count > 3 {
                // Only damage 3 max units
                break;
            }
        }
    }
}

#[derive(Component)]
pub struct BloodSplatter(pub f32);

fn update_blood_splatter(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BloodSplatter)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (entity, mut trans, mut splatter) in &mut query {
        trans.translation.y += dt * 15.0;
        let local_z = trans.local_z();
        trans.translation -= 0.4 * dt * *local_z;
        trans.scale += dt * Vec3::ONE * 15.0;

        if splatter.0 > 0.5 {
            commands.entity(entity).despawn_recursive();
        }

        splatter.0 += dt;
    }
}

#[derive(Component)]
pub struct LMGBullet {
    velocity: Vec3,
    floor_y: f32,
}

pub fn update_bullet(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut LMGBullet, &mut Transform)>,
    time: Res<Time>,
    player_camera: Query<&Transform, (With<RenderPlayer>, Without<LMGBullet>)>,
) {
    let Ok(player_cam_trans) = player_camera.get_single() else {
        return;
    };
    let dt = time.delta_seconds();
    let iter = bullets.iter_mut();
    let mut delete_one = iter.len() > 20000;
    for (entity, mut bullet, mut trans) in iter {
        if delete_one {
            commands.entity(entity).despawn_recursive();
            delete_one = false;
            continue;
        }
        if player_cam_trans.translation.y < LEVEL_TRANSITION_HEIGHT {
            bullet.floor_y = -220.0;
        }
        if trans.translation.y < bullet.floor_y + 0.1 {
            bullet.velocity.y = 0.0;
            bullet.velocity.x *= 0.993;
            bullet.velocity.z *= 0.993;
            if bullet.velocity.y == 0.0 {
                trans.rotate_local_y(3.0 * dt * bullet.velocity.x * bullet.velocity.z);
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

fn shadercomp_gun_misc(
    mut commands: Commands,
    assets: Res<GunSceneAssets>,
    mesh_assets: Res<MeshAssets>,
) {
    for scene in [assets.lmg_bullet.clone(), assets.lmg_bullet_jacket.clone()] {
        commands.spawn((
            SceneBundle {
                scene,
                transform: Transform::from_xyz(0.0, -5000.0, 0.0),
                ..default()
            },
            NoFrustumCulling,
            PropagateDefault(NoFrustumCulling),
            ShaderCompSpawn,
        ));
    }
    for scene in [mesh_assets.blood.clone(), mesh_assets.exp.clone()] {
        commands.spawn((
            SceneBundle {
                scene,
                transform: Transform::from_xyz(0.0, -5200.0, 0.0),
                ..default()
            },
            NoFrustumCulling,
            PropagateDefault(NoFrustumCulling),
        ));
    }
}
