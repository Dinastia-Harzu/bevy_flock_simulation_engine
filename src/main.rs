mod asset_related;
mod boid_simulation;
mod components;
mod constants;
mod resources;
mod states;
mod systems;

use self::{boid_simulation::BoidSimulationPlugin, constants::*, states::*, systems::*};
use asset_related::AssetsPlugin;
use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Motor de físicas en Rust".into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resolution: SCREEN_SIZE.into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins((AssetsPlugin, BoidSimulationPlugin))
        .init_state::<AppState>()
        .insert_resource(ClearColor(Color::srgba(0.0, 0.0, 0.0, 1.0)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .add_systems(Update, common_input)
        .add_systems(EguiContextPass, inspector_ui)
        .add_systems(OnEnter(AppState::Finished), exit)
        .run();
}
