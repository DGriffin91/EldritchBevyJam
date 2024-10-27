use std::{
    f32::consts::{PI, TAU},
    time::Duration,
};

use crate::{
    animation::{init_animation_graph, AnimClips, AnimPlayerController, AnimationIndices},
    character_controller::Player,
    fps_controller::RenderPlayer,
    hash_noise,
    mesh_assets::MeshAssets,
    util::{pfract, propagate, Propagate, PropagateDefault, FRAC_1_TAU},
    GameLoading, ShaderCompSpawn, LEVEL_MAIN_FLOOR,
};

use bevy::{core::FrameCount, math::vec3, prelude::*, render::view::NoFrustumCulling};
use bevy_egui::{egui, EguiContexts};

pub struct SpiderUnitPlugin;
impl Plugin for SpiderUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                propagate::<SpiderUnitAnim, AnimationPlayer>,
                init_animation_graph::<SpiderUnitAnim>,
                //ui_example_system,
                put_self_on_parent,
                spider_spawner,
                move_to_player,
                despawn_dead_spider,
                update_explosion,
            )
                .chain()
                .run_if(in_state(GameLoading::Loaded)),
        )
        .add_systems(OnEnter(GameLoading::Loaded), shadercomp_spider);
    }
}

const SPIDER_SCALE: f32 = 0.5;
const SPIDER_ATTACK_DMG: f32 = 4.0; // Per dt
const MAX_SPIDER_COUNT: usize = 1000;

#[derive(Component, Clone, Debug)]
pub struct SpiderUnit {
    pub action: SpiderAction,
    pub health: f32,
}

impl Default for SpiderUnit {
    fn default() -> Self {
        Self {
            action: Default::default(),
            health: 100.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SpiderAction {
    #[default]
    Idle,
    Attack,
    Rotate,
    Walk,
}

#[derive(Component, Clone)]
pub struct SpiderUnitAnim {
    pub main_entity: Entity,
    pub added_ref_to_self_on_parent: bool,
}

#[derive(Component, Clone)]
pub struct SpiderUnitAnimChildRef(pub Entity);

impl AnimClips for SpiderUnitAnim {
    fn get_gltf_id(&self, mesh_assets: &MeshAssets) -> Handle<Gltf> {
        mesh_assets.spider_gltf.clone_weak()
    }
}

fn spider_spawner(
    mut commands: Commands,
    player: Query<(&Transform, &Player), (With<Camera3d>, Without<SpiderUnit>)>,
    spiders: Query<&SpiderUnit>,
    time: Res<Time>,
    mut last_spawn: Local<f32>,
    mesh_assets: Res<MeshAssets>,
    frame: Res<FrameCount>,
) {
    let Ok((_player_trans, player)) = player.get_single() else {
        return;
    };
    let spiders_count = spiders.iter().len();
    if spiders_count > MAX_SPIDER_COUNT {
        return;
    }
    let t = time.elapsed_seconds();
    let rng_x = (hash_noise(frame.0, 0, 0) * 2.0 - 1.0) * 500.0;
    let rng_z = (hash_noise(frame.0, 1, 0) * 2.0 - 1.0) * 250.0 - 800.0;
    if let Some(activity_start_time) = player.activity_start_time {
        let mut spawn_interval = 3.0 / activity_start_time.powf(0.5);
        spawn_interval = spawn_interval.clamp(0.2, 2.0);
        if t > *last_spawn + spawn_interval {
            *last_spawn = t;
            let mut ecmds = commands.spawn((
                SceneBundle {
                    scene: mesh_assets.spider.clone(),
                    transform: Transform::from_xyz(rng_x, LEVEL_MAIN_FLOOR, rng_z)
                        .with_scale(Vec3::splat(SPIDER_SCALE)),
                    ..default()
                },
                SpiderUnit::default(),
                NoFrustumCulling,
                PropagateDefault(NoFrustumCulling),
            ));
            ecmds.insert(Propagate(SpiderUnitAnim {
                main_entity: ecmds.id(),
                added_ref_to_self_on_parent: false,
            }));
        }
    }
}

#[allow(unused)]
fn ui_example_system(
    mut commands: Commands,
    mesh_assets: Res<MeshAssets>,
    mut contexts: EguiContexts,
    mut spider: Query<(
        &mut AnimationTransitions,
        &AnimationIndices,
        &SpiderUnitAnim,
        &mut AnimationPlayer,
    )>,
) {
    egui::Window::new("Spider").show(contexts.ctx_mut(), |ui| {
        if ui.button("SPAWN Spider").clicked() {
            let mut ecmds = commands.spawn((
                SceneBundle {
                    scene: mesh_assets.spider.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, -180.0)
                        .with_scale(Vec3::splat(SPIDER_SCALE)),
                    ..default()
                },
                SpiderUnit::default(),
                NoFrustumCulling,
                PropagateDefault(NoFrustumCulling),
            ));
            ecmds.insert(Propagate(SpiderUnitAnim {
                main_entity: ecmds.id(),
                added_ref_to_self_on_parent: false,
            }));
        }
        let mut selected_index = None;
        let mut iter = spider.iter_mut();
        if let Some((_transitions, anim_indices, _spider_unit, _anim_player)) = iter.next() {
            for (name, anim_index) in anim_indices.iter() {
                if ui.button(name).clicked() {
                    selected_index = Some(*anim_index);
                }
            }
        }
        if let Some(selected_index) = selected_index {
            for (i, (mut transitions, _anim_indices, _spider_unit, mut anim_player)) in
                spider.iter_mut().enumerate()
            {
                transitions
                    .play(
                        &mut anim_player,
                        selected_index,
                        Duration::from_secs_f32(i as f32 * 0.1 + 1.0),
                    )
                    .repeat()
                    .set_speed(i as f32 * 0.1 + 1.0);
            }
        }
    });
}

fn put_self_on_parent(
    mut commands: Commands,
    units: Query<Entity, With<SpiderUnit>>,
    mut unit_anims: Query<(Entity, &mut SpiderUnitAnim)>,
) {
    for (unit_anim_entity, mut unit_anim) in &mut unit_anims {
        if !unit_anim.added_ref_to_self_on_parent {
            if let Ok(parent) = units.get(unit_anim.main_entity) {
                commands
                    .entity(parent)
                    .insert(SpiderUnitAnimChildRef(unit_anim_entity));
                unit_anim.added_ref_to_self_on_parent = false;
            }
        }
    }
}

fn move_to_player(
    mut units: Query<(&mut Transform, &SpiderUnitAnimChildRef, &mut SpiderUnit)>,
    mut player: Query<(&Transform, &mut Player), (With<Camera3d>, Without<SpiderUnit>)>,
    time: Res<Time>,
    mut spider_anim: Query<(
        &mut AnimationTransitions,
        &AnimationIndices,
        &SpiderUnitAnim,
        &mut AnimationPlayer,
    )>,
) {
    let Ok((player_trans, mut player_stats)) = player.get_single_mut() else {
        return;
    };
    let dt = time.delta_seconds();

    let dest = player_trans.translation;
    let attack_dist = 3.0;
    let base_walk_speed = 10.0;
    let base_turn_speed = 3.0;

    for (mut unit_trans, anim_child, mut unit) in &mut units {
        if let Ok((mut transitions, anim, _spider_unit, mut player)) =
            spider_anim.get_mut(anim_child.0)
        {
            let mut player = AnimPlayerController::new(&mut transitions, &mut player, anim);

            let forward = *unit_trans.forward();
            let mut to_dest =
                (dest - unit_trans.translation).normalize_or(unit_trans.translation + forward);
            to_dest.y = forward.y;
            to_dest = to_dest.normalize_or_zero();

            let to_dist = (dest - unit_trans.translation).length();

            let to_dest_dir = (to_dest.x.atan2(to_dest.z) + PI) * FRAC_1_TAU;
            let forward_dir = (forward.x.atan2(forward.z) + PI) * FRAC_1_TAU;
            let need_to_rotate_dir = pfract(forward_dir - to_dest_dir) - 0.5;
            let dir_anim_index = if need_to_rotate_dir > 0.0 {
                "Wandering_Turn_Left"
            } else {
                "Wandering_Turn_Right"
            };

            let need_to_turn = to_dest.dot(*unit_trans.forward()) < 0.93;

            let buffer = 2.0;
            let should_pursue = !need_to_turn && to_dist > attack_dist;
            let should_attack = !need_to_turn && to_dist - buffer < attack_dist;

            let attacking = player.playing("Attack");

            if !attacking && should_attack {
                player.play("Attack", 0.1, 1.0, true);
            } else if !should_attack && !player.playing(dir_anim_index) && need_to_turn {
                player.play(dir_anim_index, 0.1, 1.0, true);
            } else if !should_attack && !player.playing("Wandering_Walk_Cycle") && should_pursue {
                player.play("Wandering_Walk_Cycle", 0.1, 6.0, true);
            }

            if player.playing("Attack") {
                unit.action = SpiderAction::Attack;
                player_stats.health -= dt * SPIDER_ATTACK_DMG;

                //let active_anim = player.animation("Attack").unwrap();
                //let anim_speed = active_anim.speed();

                //let dest_rot =
                //    unit_trans.looking_at(vec3(dest.x, unit_trans.translation.y, dest.z), Vec3::Y);

                //unit_trans.rotation = unit_trans
                //    .rotation
                //    .lerp(dest_rot.rotation, (0.1 * anim_speed).clamp(0.0, 1.0));
            } else if player.playing("Wandering_Walk_Cycle") {
                unit.action = SpiderAction::Walk;

                let active_anim = player.animation("Wandering_Walk_Cycle").unwrap();

                let current_y = unit_trans.translation.y;

                let anim_speed = active_anim.speed();
                unit_trans.translation +=
                    to_dest * SPIDER_SCALE * dt * base_walk_speed * anim_speed;
                let dest_rot = unit_trans.looking_at(vec3(dest.x, current_y, dest.z), Vec3::Y);

                unit_trans.rotation = unit_trans
                    .rotation
                    .lerp(dest_rot.rotation, (dt * anim_speed).clamp(0.0, 1.0));
            } else if player.playing(dir_anim_index) {
                unit.action = SpiderAction::Rotate;

                let turn_sign = need_to_rotate_dir.signum();

                let active_anim = player.animation(dir_anim_index).unwrap();

                let anim_speed = active_anim.speed();

                //SPIDER_SCALE * // Small things don't turn slower
                unit_trans.rotate_local_y(dt * base_turn_speed * turn_sign * anim_speed * TAU);
            }
        }
    }
}

fn despawn_dead_spider(
    mut commands: Commands,
    units: Query<(Entity, &Transform, &SpiderUnit)>,
    mesh_assets: Res<MeshAssets>,
    mut player_camera: Query<(&Transform, &mut Player), (With<RenderPlayer>, Without<SpiderUnit>)>,
) {
    let Ok((player_cam_trans, mut player)) = player_camera.get_single_mut() else {
        return;
    };
    for (entity, trans, unit) in &units {
        if unit.health < 0.0 {
            commands.entity(entity).despawn_recursive();
            commands.spawn((
                SceneBundle {
                    scene: mesh_assets.exp.clone(),
                    transform: Transform::from_translation(trans.translation.into())
                        .looking_at(player_cam_trans.translation, Vec3::Y),
                    ..default()
                },
                Explosion(0.0),
            ));
            player.kills += 1;
        }
    }
}

#[derive(Component)]
pub struct Explosion(pub f32);

fn update_explosion(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Explosion)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (entity, mut trans, mut exp) in &mut query {
        trans.translation.y += dt * 10.0;
        trans.scale += dt * Vec3::ONE * 30.0;

        if exp.0 > 1.0 {
            commands.entity(entity).despawn_recursive();
        }

        exp.0 += dt;
    }
}

fn shadercomp_spider(mut commands: Commands, mesh_assets: Res<MeshAssets>) {
    commands.spawn((
        SceneBundle {
            scene: mesh_assets.spider.clone(),
            transform: Transform::from_xyz(0.0, -5000.0, 0.0),
            ..default()
        },
        NoFrustumCulling,
        PropagateDefault(NoFrustumCulling),
        ShaderCompSpawn,
    ));
}
