use crate::{
    boid_simulation::{components::*, resources::*},
    states::SimulationState,
};
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

    egui::Window::new("Boids Config").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Configuración de los boids");
            ui_for_resource::<BoidConfiguration>(world, ui);
            let Ok((mut selected_boid, mut testing_boid)) = world
                .query::<(&mut Boid, &mut BoidTestingUnit)>()
                .single_mut(world)
            else {
                return;
            };
            ui.separator();
            ui.heading("Boid seleccionado");
            ui.add(egui::Slider::new(&mut selected_boid.speed, 0.0..=500.0));
            ui.drag_angle(&mut selected_boid.angle);
            ui.checkbox(&mut testing_boid.follow_boids, "Seguir demás boids");

            if ui.button("Reiniciar simulación").clicked() {
                world
                    .resource_mut::<NextState<SimulationState>>()
                    .set(SimulationState::Setup);
            }

            ui.checkbox(
                &mut world.resource_mut::<SimulationConfiguration>().should_draw,
                "Dibujar cosas para depurar",
            );
        });
    });

    egui::Window::new("Boids").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui_for_entities_filtered::<Filter<With<Boid>>>(world, ui, true, &Filter::all());
        });
    });
}
