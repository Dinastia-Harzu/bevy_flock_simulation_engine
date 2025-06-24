use crate::boid_simulation::{components::*, resources::*};
use bevy::prelude::*;
use bevy_egui::*;
use bevy_inspector_egui::{bevy_inspector::*, DefaultInspectorConfigPlugin};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .register_type::<BoidConfiguration>()
        .add_systems(EguiContextPass, inspector_ui);
    }
}

pub fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<Window>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("Configuración de los Boids").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui_for_resource::<BoidConfiguration>(world, ui);
            let Ok(mut testing_unit_boid_transform) = world
                .query_filtered::<&mut Transform, With<BoidTestingUnit>>()
                .single_mut(world)
            else {
                return;
            };
            let mut new_rotation = testing_unit_boid_transform.rotation.to_axis_angle().1;
            ui.drag_angle(&mut new_rotation);
            testing_unit_boid_transform.rotation = Quat::from_axis_angle(Vec3::Z, new_rotation);
        });
    });
    egui::Window::new("Boids").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui_for_entities_filtered::<Filter<With<Boid>>>(world, ui, true, &Filter::all());
        });
    });
}
