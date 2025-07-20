use bevy::prelude::*;
use bevy_flock_simulation_engine::{constants::*, BevyFlockSimulationEnginePlugins};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Motor de físicas para simulación de boids".into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resolution: SCREEN_SIZE.into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BevyFlockSimulationEnginePlugins)
        .run();
}
