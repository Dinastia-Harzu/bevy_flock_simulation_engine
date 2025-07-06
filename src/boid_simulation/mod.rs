pub(crate) mod components;
pub(crate) mod resources;
pub(crate) mod rules;
pub(crate) mod systems;

use self::{components::*, resources::*, rules::*, systems::*};
use crate::states::*;
use bevy::prelude::*;

pub struct BoidSimulationPlugin;

impl Plugin for BoidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>()
            .init_resource::<BoidRules>()
            .insert_resource(BoidConfiguration::default())
            .insert_resource(SpatialGrid::new(5, 7, 200.0))
            .insert_resource(SimulationConfiguration::default())
            .register_type::<Boid>()
            .add_systems(Startup, setup_rules)
            .add_systems(
                PreUpdate,
                (clear_simulation, spawn_boids)
                    .chain()
                    .run_if(in_state(SimulationState::Setup).and(in_state(AppState::Running))),
            )
            .add_systems(
                FixedUpdate,
                (update_spatial_grid, update_boids, wrap_edges).chain(),
            )
            .add_systems(PostUpdate, (update_debug_boid, draw_spatial_grid));
    }
}
