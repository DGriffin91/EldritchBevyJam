use std::{
    f32::consts::{PI, TAU},
    time::Duration,
};

use crate::{
    animation::{
        init_animation_graph, ramp_up_down_anim, AnimClips, AnimPlayerController, AnimationIndices,
    },
    character_controller::Player,
    fps_controller::RenderPlayer,
    hash_noise,
    menu::menu_ui,
    mesh_assets::MeshAssets,
    util::{pfract, propagate, Propagate, PropagateDefault, FRAC_1_TAU},
    GameLoading, ShaderCompSpawn, LEVEL_MAIN_FLOOR,
};

use bevy::{core::FrameCount, math::vec3, prelude::*, render::view::NoFrustumCulling};
use bevy_egui::{egui, EguiContexts};

use super::spider::Explosion;

pub struct PlumUnitPlugin;
impl Plugin for PlumUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                propagate::<PlumUnitAnim, AnimationPlayer>,
                init_animation_graph::<PlumUnitAnim>,
                //ui_example_system,
                plum_spawner,
                put_self_on_parent,
                move_to_player,
                despawn_dead_plum,
            )
                .chain()
                .run_if(in_state(GameLoading::Loaded))
                .before(menu_ui),
        )
        .add_systems(OnEnter(GameLoading::Loaded), shadercomp_plum);
    }
}

const MAX_PLUM_COUNT: usize = 30;
const PLUM_ATTACK_DMG: f32 = 20.0; // Per boom
const PLUM_ATTACK_RADIUS: f32 = 20.0;

#[derive(Component, Clone, Debug)]
pub struct PlumUnit {
    pub action: PlumAction,
    pub health: f32,
}

impl Default for PlumUnit {
    fn default() -> Self {
        Self {
            action: Default::default(),
            health: 100.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PlumAction {
    #[default]
    Idle,
    Attack,
    Rotate,
    Walk,
}

#[derive(Component, Clone)]
pub struct PlumUnitAnim {
    pub main_entity: Entity,
    pub added_ref_to_self_on_parent: bool,
}

#[derive(Component, Clone)]
pub struct PlumUnitAnimChildRef(pub Entity);

impl AnimClips for PlumUnitAnim {
    fn get_gltf_id(&self, mesh_assets: &MeshAssets) -> Handle<Gltf> {
        mesh_assets.plum_gltf.clone_weak()
    }
}

fn plum_spawner(
    mut commands: Commands,
    player: Query<(&Transform, &Player), (With<Camera3d>, Without<PlumUnit>)>,
    plums: Query<&PlumUnit>,
    time: Res<Time>,
    mut last_spawn: Local<f32>,
    mesh_assets: Res<MeshAssets>,
    frame: Res<FrameCount>,
) {
    let Ok((_player_trans, player)) = player.get_single() else {
        return;
    };
    let spiders_count = plums.iter().len();
    if spiders_count > MAX_PLUM_COUNT {
        return;
    }
    let t = time.elapsed_seconds();
    let rng_x = (hash_noise(frame.0, 0, 1) * 2.0 - 1.0) * 500.0;
    let rng_z = (hash_noise(frame.0, 1, 1) * 2.0 - 1.0) * 150.0 - 700.0;
    if let Some(activity_start_time) = player.activity_start_time {
        let mut spawn_interval = 10.0 / activity_start_time.powf(0.2);
        spawn_interval = spawn_interval.clamp(0.2, 2.0);
        if t > *last_spawn + spawn_interval {
            *last_spawn = t;
            let mut ecmds = commands.spawn((
                SceneBundle {
                    scene: mesh_assets.plum.clone(),
                    transform: Transform::from_xyz(rng_x, LEVEL_MAIN_FLOOR, rng_z),
                    ..default()
                },
                PlumUnit::default(),
                NoFrustumCulling,
                PropagateDefault(NoFrustumCulling),
            ));
            ecmds.insert(Propagate(PlumUnitAnim {
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
    mut plum: Query<(
        &mut AnimationTransitions,
        &AnimationIndices,
        &PlumUnitAnim,
        &mut AnimationPlayer,
    )>,
) {
    egui::Window::new("Plum").show(contexts.ctx_mut(), |ui| {
        if ui.button("SPAWN Plum").clicked() {
            let mut ecmds = commands.spawn((
                SceneBundle {
                    scene: mesh_assets.plum.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, -100.0),
                    ..default()
                },
                PlumUnit::default(),
                NoFrustumCulling,
                PropagateDefault(NoFrustumCulling),
            ));
            ecmds.insert(Propagate(PlumUnitAnim {
                main_entity: ecmds.id(),
                added_ref_to_self_on_parent: false,
            }));
        }
        let mut selected_index = None;
        let mut iter = plum.iter_mut();
        if let Some((_transitions, anim_indices, _plum_unit, _anim_player)) = iter.next() {
            for (name, anim_index) in anim_indices.iter() {
                if ui.button(name).clicked() {
                    selected_index = Some(*anim_index);
                }
            }
        }
        if let Some(selected_index) = selected_index {
            for (i, (mut transitions, _anim_indices, _plum_unit, mut anim_player)) in
                plum.iter_mut().enumerate()
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
    units: Query<Entity, With<PlumUnit>>,
    mut unit_anims: Query<(Entity, &mut PlumUnitAnim)>,
) {
    for (unit_anim_entity, mut unit_anim) in &mut unit_anims {
        if !unit_anim.added_ref_to_self_on_parent {
            if let Ok(parent) = units.get(unit_anim.main_entity) {
                commands
                    .entity(parent)
                    .insert(PlumUnitAnimChildRef(unit_anim_entity));
                unit_anim.added_ref_to_self_on_parent = false;
            }
        }
    }
}

fn move_to_player(
    mut commands: Commands,
    mut units: Query<(Entity, &mut Transform, &PlumUnitAnimChildRef, &mut PlumUnit)>,
    time: Res<Time>,
    mut plum_anim: Query<(
        &mut AnimationTransitions,
        &AnimationIndices,
        &PlumUnitAnim,
        &mut AnimationPlayer,
    )>,
    mut player: Query<(&Transform, &mut Player), (With<Camera3d>, Without<PlumUnit>)>,
    mesh_assets: Res<MeshAssets>,
) {
    let Ok((player_trans, mut player_stats)) = player.get_single_mut() else {
        return;
    };
    let dt = time.delta_seconds();
    let dead = player_stats.health < 0.0;

    let dest = if dead {
        vec3(0.0, LEVEL_MAIN_FLOOR, -1200.0)
    } else {
        player_trans.translation
    };

    let attack_dist = 15.0;

    for (unit_entity, mut unit_trans, anim_child, mut unit) in &mut units {
        if let Ok((mut transitions, anim, _plum_unit, mut player)) = plum_anim.get_mut(anim_child.0)
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
                "Fast_Turning_Left"
            } else {
                "Fast_Turning_Right"
            };

            let need_to_turn = to_dest.dot(*unit_trans.forward()) < 0.93;

            let buffer = 2.0;
            let should_attack = to_dist - buffer < attack_dist && !dead;
            let should_pursue = !need_to_turn && to_dist > attack_dist;

            let attacking = player.playing("Attack");

            if !attacking && should_attack {
                player.play("Attack", 0.1, 2.0, false);
            } else if !attacking && !player.playing(dir_anim_index) && need_to_turn {
                player.play(dir_anim_index, 0.1, 2.0, true);
            } else if !attacking && !player.playing("Fast_Walk_Cycle") && should_pursue {
                player.play("Fast_Walk_Cycle", 0.1, if dead { 1.0 } else { 3.0 }, true);
            }

            if player.playing("Attack") {
                unit.action = PlumAction::Attack;

                let active_anim = player.animation("Attack").unwrap();
                let anim_speed = active_anim.speed();

                if active_anim.is_finished() {
                    if dest.distance(unit_trans.translation) < PLUM_ATTACK_RADIUS {
                        player_stats.health -= PLUM_ATTACK_DMG;
                    }
                    commands.entity(unit_entity).despawn_recursive();
                    commands.spawn((
                        SceneBundle {
                            scene: mesh_assets.exp.clone(),
                            transform: Transform::from_translation(unit_trans.translation.into()),
                            ..default()
                        },
                        Explosion(0.0),
                    ));
                } else {
                    let dest_rot = unit_trans
                        .looking_at(vec3(dest.x, unit_trans.translation.y, dest.z), Vec3::Y);
                    unit_trans.rotation = unit_trans
                        .rotation
                        .lerp(dest_rot.rotation, (0.15 * anim_speed).clamp(0.0, 1.0));
                }
            } else if player.playing("Fast_Walk_Cycle") {
                unit.action = PlumAction::Walk;

                let base_walk_speed = 14.0;

                let active_anim = player.animation("Fast_Walk_Cycle").unwrap();
                let seek_f = active_anim.seek_time() * 24.0 + 10.0; // TODO what offset by 10?

                let current_y = unit_trans.translation.y;
                let move_start = 20.0;
                let move_end = 27.0;
                let move_length = move_end - move_start;

                if seek_f > move_start && seek_f < move_end {
                    let anim_speed = ramp_up_down_anim(seek_f, move_start, move_length, 1.5)
                        * active_anim.speed();
                    unit_trans.translation += to_dest * dt * base_walk_speed * anim_speed;
                    let dest_rot = unit_trans.looking_at(vec3(dest.x, current_y, dest.z), Vec3::Y);

                    unit_trans.rotation = unit_trans
                        .rotation
                        .lerp(dest_rot.rotation, (0.15 * anim_speed).clamp(0.0, 1.0));
                }
            } else if player.playing(dir_anim_index) {
                unit.action = PlumAction::Rotate;

                let base_turn_speed = 1.0 * need_to_rotate_dir.signum();

                let active_anim = player.animation(dir_anim_index).unwrap();
                let seek_f = active_anim.seek_time() * 24.0 + 0.0;

                let move_start = 16.0;
                let move_end = 20.0;
                let move_length = move_end - move_start;

                if seek_f > move_start && seek_f < move_end {
                    let anim_speed = ramp_up_down_anim(seek_f, move_start, move_length, 1.5)
                        * active_anim.speed();

                    unit_trans.rotate_local_y(dt * base_turn_speed * anim_speed * TAU);
                }
            }
        }
    }
}

fn despawn_dead_plum(
    mut commands: Commands,
    units: Query<(Entity, &Transform, &PlumUnit)>,
    mesh_assets: Res<MeshAssets>,
    mut player_camera: Query<(&Transform, &mut Player), (With<RenderPlayer>, Without<PlumUnit>)>,
) {
    let Ok((_player_cam_trans, mut player)) = player_camera.get_single_mut() else {
        return;
    };
    for (entity, trans, unit) in &units {
        if unit.health < 0.0 {
            commands.entity(entity).despawn_recursive();
            commands.spawn((
                SceneBundle {
                    scene: mesh_assets.exp.clone(),
                    transform: Transform::from_translation(trans.translation),
                    ..default()
                },
                Explosion(0.2),
            ));
            commands.spawn((
                SceneBundle {
                    scene: mesh_assets.exp.clone(),
                    transform: Transform::from_translation(trans.translation + Vec3::Y * 1.5),
                    ..default()
                },
                Explosion(0.0),
            ));
            player.kills += 1;
        }
    }
}

fn shadercomp_plum(mut commands: Commands, mesh_assets: Res<MeshAssets>) {
    commands.spawn((
        SceneBundle {
            scene: mesh_assets.plum.clone(),
            transform: Transform::from_xyz(0.0, -5100.0, 0.0),
            ..default()
        },
        NoFrustumCulling,
        PropagateDefault(NoFrustumCulling),
        ShaderCompSpawn,
    ));
}
