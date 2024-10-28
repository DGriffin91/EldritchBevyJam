use bevy::math::vec3;
use bevy::window::CursorGrabMode;
use bevy::{prelude::*, window::WindowMode};
use bevy_egui::{egui, EguiContexts};

use bs13_render::{else_return, BS13ViewTargetSettings};
use fps_controller::FpsController;

use crate::character_controller::Player;
use crate::fps_controller::{self, LogicalPlayer};
use crate::guns::LMGBullet;
use crate::units::plum::PlumUnit;
use crate::units::spider::SpiderUnit;
use crate::{GameLoading, StartLevel};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserSettings>()
            .add_systems(PostUpdate, menu_ui.run_if(in_state(GameLoading::Loaded)));
    }
}

#[derive(Resource, Default)]
pub struct UserSettings {
    pub disable_muzzle_flash: bool,
}

pub fn menu_ui(
    mut commands: Commands,
    mut fps_controller: Query<&mut FpsController>,
    mut windows: Query<&mut Window>,
    mut contexts: EguiContexts,
    mut settings: ResMut<UserSettings>,
    mut view_target_settings: Query<&mut BS13ViewTargetSettings>,
    stuff_to_despawn: Query<Entity, Or<(With<PlumUnit>, With<SpiderUnit>, With<LMGBullet>)>>,
    mut player: Query<&mut Player>,
    mut logical_player: Query<(&mut Transform, &LogicalPlayer)>,
    mut start_level_items: Query<(Entity, &mut Visibility), With<StartLevel>>,
) {
    let Ok(mut player_stats) = player.get_single_mut() else {
        return;
    };
    let Ok((mut player_trans, _)) = logical_player.get_single_mut() else {
        return;
    };
    let mut window = windows.single_mut();
    let mut fps_controller = fps_controller.single_mut();
    let mut view_target_settings = else_return!(view_target_settings.get_single_mut().ok());
    let cursor_locked = window.cursor.grab_mode == CursorGrabMode::Locked;
    if cursor_locked {
        return;
    }
    let height = window.height();
    let width = 300.0;

    egui::Window::new("SETTINGS")
        .fixed_pos(egui::Pos2::ZERO)
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(width, height))
        .show(contexts.ctx_mut(), |ui| {
            ui.allocate_space(egui::vec2(width, 40.0));
            ui.spacing_mut().slider_width = ui.available_width();

            if ui.button("RESTART GAME").clicked() {
                for entity in &stuff_to_despawn {
                    commands.entity(entity).despawn_recursive();
                }
                player_trans.translation = vec3(0.0, 2.0, -200.0);
                *player_stats = Default::default();

                for (_entity, mut vis) in &mut start_level_items {
                    *vis = Visibility::Visible;
                }
            }

            ui.label("GAME SETTINGS");
            let mut sens = fps_controller.sensitivity * 1000.0;
            if ui
                .add(egui::Slider::new(&mut sens, 0.1..=10.0).text("MOUSE SENSITIVITY"))
                .changed()
            {
                fps_controller.sensitivity = sens / 1000.0;
            }

            ui.allocate_space(egui::vec2(width, 40.0));
            ui.label("RENDER SETTINGS");
            ui.checkbox(&mut settings.disable_muzzle_flash, "DISABLE MUZZLE FLASH");
            ui.add(
                egui::Slider::new(&mut view_target_settings.render_scale, 0.25..=2.0)
                    .text("RENDER SCALE"),
            );

            ui.label("WINDOW MODE");
            if ui
                .radio(
                    window.mode == WindowMode::BorderlessFullscreen,
                    "BORDERLESS FULLSCREEN",
                )
                .clicked()
            {
                window.mode = WindowMode::BorderlessFullscreen;
            }
            if ui
                .radio(window.mode == WindowMode::Fullscreen, "FULLSCREEN")
                .clicked()
            {
                window.mode = WindowMode::Fullscreen;
            }
            if ui
                .radio(window.mode == WindowMode::Windowed, "WINDOWED")
                .clicked()
            {
                window.mode = WindowMode::Windowed;
            }

            ui.allocate_space(egui::vec2(width, height));
        });
}
