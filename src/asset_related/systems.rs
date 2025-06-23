use super::resources::*;
use crate::states::*;
use bevy::prelude::*;

pub(super) fn use_asset_handles(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    app_next_state.set(AppState::Running);
}
