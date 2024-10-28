use bevy::window::CursorGrabMode;
use bevy::{prelude::*, window::WindowMode};
use bevy_egui::{egui, EguiContexts};

use bs13_render::{else_return, BS13ViewTargetSettings};
use fps_controller::FpsController;

use crate::fps_controller;

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserSettings>()
            .add_systems(Update, menu_ui);
    }
}

#[derive(Resource, Default)]
pub struct UserSettings {
    pub disable_muzzle_flash: bool,
}

pub fn menu_ui(
    //mut commands: Commands,
    mut fps_controller: Query<&mut FpsController>,
    mut windows: Query<&mut Window>,
    mut contexts: EguiContexts,
    mut settings: ResMut<UserSettings>,
    mut view_target_settings: Query<&mut BS13ViewTargetSettings>,
) {
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
