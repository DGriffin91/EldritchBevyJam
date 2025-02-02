use bevy::prelude::*;
use minimal_kira_audio::KiraTrackHandle;

pub mod animation;
pub mod audio;
pub mod character_controller;
pub mod fps_controller;
pub mod guns;
pub mod menu;
pub mod mesh_assets;
pub mod minimal_kira_audio;
pub mod physics;
pub mod units;
pub mod util;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameLoading {
    #[default]
    AssetLoading,
    AssetLoading2,
    Loaded,
}
pub const LEVEL_TRANSITION_HEIGHT: f32 = -200.0;
pub const LEVEL_MAIN_FLOOR: f32 = -220.0;

#[inline(always)]
pub fn uhash(a: u32, b: u32) -> u32 {
    let mut x = (a.overflowing_mul(1597334673).0) ^ (b.overflowing_mul(3812015801).0);
    // from https://nullprogram.com/blog/2018/07/31/
    x = x ^ (x >> 16);
    x = x.overflowing_mul(0x7feb352d).0;
    x = x ^ (x >> 15);
    x = x.overflowing_mul(0x846ca68b).0;
    x = x ^ (x >> 16);
    x
}

#[inline(always)]
pub fn unormf(n: u32) -> f32 {
    n as f32 * (1.0 / 0xffffffffu32 as f32)
}

#[inline(always)]
pub fn hash_noise(x: u32, y: u32, z: u32) -> f32 {
    let urnd = uhash(x, (y << 11) + z);
    unormf(urnd)
}

#[derive(Component, Clone, Copy)]
pub struct ShaderCompSpawn;
#[derive(Component)]
pub struct StartLevel;

#[derive(Component, Clone)]
pub struct PlayerStart;

#[derive(Resource)]
pub struct MusicTrack {
    pub handle: Handle<KiraTrackHandle>,
    pub volume: f32,
}
#[derive(Resource)]
pub struct SfxTrack {
    pub handle: Handle<KiraTrackHandle>,
    pub volume: f32,
}
