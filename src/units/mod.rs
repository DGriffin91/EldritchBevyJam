use bevy::prelude::*;
use plum::PlumUnitPlugin;

pub mod fox_unit;
pub mod plum;

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlumUnitPlugin);
    }
}
