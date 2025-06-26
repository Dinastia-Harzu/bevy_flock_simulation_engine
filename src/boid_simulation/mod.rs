pub(crate) mod components;
pub(crate) mod resources;
pub(crate) mod systems;

use self::{components::*, resources::*, systems::*};
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
        .insert_resource(SpatialGrid::new(5, 8, 200.0))
        .register_type::<Boid>()
        .add_systems(OnEnter(AppState::Next), spawn_boids)
        .add_systems(FixedUpdate, (update_boids, wrap_edges).chain())
        .add_systems(PostUpdate, (update_debug_boid, draw_spatial_grid));
    }
}
