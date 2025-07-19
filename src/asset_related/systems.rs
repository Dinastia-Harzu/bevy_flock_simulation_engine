use crate::states::*;
use bevy::prelude::*;

pub(super) fn start_running(
    mut _commands: Commands,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    app_next_state.set(AppState::Running);
}
