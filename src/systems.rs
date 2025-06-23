use crate::{states::*};
use bevy::{
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_entities(world, ui);
        });
    });
}

pub fn common_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        app_next_state.set(AppState::Finished);
    }
}

pub fn exit(mut commands: Commands, window: Single<Entity, With<Window>>) {
    commands.entity(window.entity()).despawn();
}
