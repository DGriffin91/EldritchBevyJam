use bevy::prelude::*;

pub mod animation;
pub mod audio;
pub mod character_controller;
pub mod fps_controller;
pub mod guns;
pub mod mesh_assets;
pub mod minimal_kira_audio;
pub mod physics;
pub mod units;
pub mod util;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameLoading {
    #[default]
    AssetLoading,
    Loaded,
}
pub const LEVEL_TRANSITION_HEIGHT: f32 = -200.0;
