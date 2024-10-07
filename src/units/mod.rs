use bevy::prelude::*;
use fox_unit::FoxUnitPlugin;

pub mod fox_unit;

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FoxUnitPlugin);
    }
}
