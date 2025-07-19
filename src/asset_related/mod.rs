pub(crate) mod resources;
pub(crate) mod systems;

use self::{resources::*, systems::*};
use crate::states::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::Next)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("assets.ron")
                .load_collection::<ImageAssets>(),
        )
        .add_systems(Update, use_asset_handles.run_if(in_state(AppState::Next)));
    }
}
