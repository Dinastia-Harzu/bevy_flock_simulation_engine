use crate::{
    boid_simulation::{components::*, resources::*},
    states::*,
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
            let mut boid_config = world.resource_mut::<BoidConfiguration>();
            ui.heading("Configuración de los boids");
            let min_speed = boid_config.min_speed;
            let max_speed = boid_config.max_speed;
            ui.label("Parámetros comunes:");
            ui.add(
                egui::Slider::new(
                    &mut boid_config.min_speed,
                    BoidConfiguration::lowest_speed()..=max_speed,
                )
                .text("Velocidad mínima"),
            );
            ui.add(
                egui::Slider::new(
                    &mut boid_config.max_speed,
                    min_speed..=BoidConfiguration::highest_speed(),
                )
                .text("Velocidad máxima"),
            );
            ui.add(
                egui::Slider::new(&mut boid_config.scale, BoidConfiguration::SCALE_RANGE)
                    .text("Tamaño de los boids"),
            );
            ui.label("Parámetros personalizados:");
            for (name, (value, range)) in &mut boid_config {
                ui.add(egui::Slider::new(value, range.clone()).text(format!(
                    "{name} [{}..{}]",
                    *range.start(),
                    *range.end()
                )));
            }

            if let Ok((mut selected_boid, mut testing_boid)) = world
                .query::<(&mut Boid, &mut BoidTestingUnit)>()
                .single_mut(world)
            {
                let follow_boids = &mut testing_boid.follow_boids;
                ui.separator();
                ui.heading("Boid seleccionado");
                ui.checkbox(follow_boids, "Seguir demás boids");
                if !*follow_boids {
                    ui.add(
                        egui::Slider::new(&mut selected_boid.speed, min_speed..=max_speed)
                            .text("Velocidad"),
                    );
                    ui.label("Ángulo: ");
                    ui.drag_angle(&mut selected_boid.angle);
                }
            }

            let there_are_predators = !world
                .query_filtered::<(), With<BoidPredator>>()
                .query(world)
                .is_empty();
            let mut simulation_config = world.resource_mut::<SimulationConfiguration>();
            ui.separator();
            ui.heading("Simulación");
            ui.add(
                egui::Slider::new(
                    &mut simulation_config.normal_boids,
                    SimulationConfiguration::BOIDS_RANGE,
                )
                .text("Número de boids"),
            );
            ui.checkbox(
                &mut simulation_config.should_draw,
                "Dibujar cosas para depurar",
            );
            ui.add(
                egui::Slider::new(&mut simulation_config.predators, 0..=10).text("Depredadores"),
            );
            if there_are_predators {
                ui.add(
                    egui::Slider::new(&mut simulation_config.predator_hunt_weight, 0.0..=1.0)
                        .text("Peso de atosigamiento"),
                );
            }
            if ui.button("Reiniciar simulación").clicked() {
                world
                    .resource_mut::<NextState<SimulationState>>()
                    .set(SimulationState::Setup);
            }
        });
    });

    egui::Window::new("Boids").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui_for_world(world, ui);
        });
    });
}
