use bevy::{prelude::*, render::view::NoFrustumCulling};
use plum::PlumUnitPlugin;
use spider::SpiderUnitPlugin;

use crate::util::propagate_default;

pub mod fox_unit;
pub mod plum;
pub mod spider;

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PlumUnitPlugin, SpiderUnitPlugin))
            .add_systems(Update, propagate_default::<NoFrustumCulling, Handle<Mesh>>);
    }
}
