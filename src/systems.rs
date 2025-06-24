use crate::states::*;
use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn common_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        app_next_state.set(AppState::Finished);
    }
}

pub fn exit(mut commands: Commands, window: Single<Entity, With<Window>>) {
    commands.entity(window.entity()).despawn();
}
