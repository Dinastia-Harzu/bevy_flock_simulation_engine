mod components;
mod resources;
mod systems;

use self::{resources::*, systems::*};
use crate::states::*;
use bevy::prelude::*;

pub struct BoidSimulationPlugin;

impl Plugin for BoidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoidConfiguration {
            speed: 100.0,
            inner_perception_radius: 100.0,
            outer_perception_radius: 500.0,
            separation_factor: 1.0,
            alignment_factor: 1.0,
            cohesion_factor: 1.0,
        })
        .add_systems(OnEnter(AppState::Next), spawn_boids)
        .add_systems(FixedUpdate, (update_boids, wrap_edges).chain())
        .add_systems(PostUpdate, update_debug_boid);
    }
}
