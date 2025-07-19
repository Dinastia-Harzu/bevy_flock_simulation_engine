pub(crate) mod bundles;
pub(crate) mod components;
pub(crate) mod resources;
pub(crate) mod systems;

use self::{components::*, resources::*, systems::*};
use crate::states::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct BoidSimulationPlugin;

impl Plugin for BoidSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>()
            .init_resource::<BoidRules>()
            .init_resource::<BoidConfiguration>()
            .init_resource::<SimulationConfiguration>()
            .insert_resource(SpatialGrid::with_cell_size(200.0))
            .register_type::<Boid>()
            .register_type::<WindCurrent>()
            .register_type::<ForceField>()
            .add_systems(Startup, setup_boid_parametres)
            .add_systems(
                PreUpdate,
                (clear_simulation, setup_simulation)
                    .chain()
                    .run_if(in_state(SimulationState::Setup).and(in_state(AppState::Running))),
            )
            .add_systems(
                FixedUpdate,
                (update_spatial_grid, update_boids, wrap_edges).chain(),
            )
            .add_systems(PostUpdate, draw_debug);
    }
}
