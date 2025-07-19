pub mod asset_related;
pub mod boid_simulation;
pub mod constants;
pub mod helpers;
pub mod inspector;
pub mod states;
pub mod systems;

use self::{asset_related::*, boid_simulation::*, inspector::*, states::*, systems::*};
use bevy::{app::plugin_group, prelude::*};

plugin_group! {
    pub struct BevyFlockSimulationEnginePlugins {
        :InspectorPlugin,
        :AssetsPlugin,
        :BoidSimulationPlugin,
        :BogStandardPlugin,
    }
}

#[derive(Default)]
pub struct BogStandardPlugin;

impl Plugin for BogStandardPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .insert_resource(ClearColor(Color::srgba(0.0, 0.0, 0.0, 1.0)))
            .add_systems(Startup, setup)
            .add_systems(Update, common_input)
            .add_systems(OnEnter(AppState::Finished), exit);
    }
}
