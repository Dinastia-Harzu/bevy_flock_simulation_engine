use bevy::prelude::*;
use bevy_mod_imgui::prelude::*;

const BOUNDS: Vec2 = Vec2::new(1920.0, 1080.0);

#[derive(Resource)]
struct ImguiState {
    demo_window_open: bool,
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
}

#[derive(Component)]
struct SnapToPlayer;

#[derive(Component)]
struct RotateToPlayer {
    rotation_speed: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Motor de físicas en Rust"),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resolution: BOUNDS.into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(ImguiPlugin {
            ini_filename: Some("imgui.ini".into()),
            ..default()
        })
        .insert_resource(ClearColor(Color::srgba(0.0, 0.0, 0.0, 1.0)))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ImguiState {
            demo_window_open: true,
        })
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                player_movement_system,
                snap2player_system,
                rotate2player_system,
            ),
        )
        .add_systems(PostUpdate, imgui_example_ui)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle =
        asset_server.load("/home/agrg11/Documentos/uni/tfg/motor-fisicas/assets/textures/ship.png");
    let enemy_handle = asset_server
        .load("/home/agrg11/Documentos/uni/tfg/motor-fisicas/assets/textures/enemy.png");

    commands.spawn(Camera2d);

    let horizontal_margin = BOUNDS.x / 4.0;
    let vertical_margin = BOUNDS.y / 4.0;

    commands.spawn((
        Sprite::from_image(ship_handle),
        Player {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(360.0),
        },
    ));

    commands.spawn((
        Sprite::from_image(enemy_handle.clone()),
        Transform::from_xyz(0.0 - horizontal_margin, 0.0, 0.0),
        SnapToPlayer,
    ));
    commands.spawn((
        Sprite::from_image(enemy_handle.clone()),
        Transform::from_xyz(0.0, 0.0 - vertical_margin, 0.0),
        SnapToPlayer,
    ));
    commands.spawn((
        Sprite::from_image(enemy_handle.clone()),
        Transform::from_xyz(0.0 + horizontal_margin, 0.0, 0.0),
        RotateToPlayer {
            rotation_speed: f32::to_radians(45.0),
        },
    ));
    commands.spawn((
        Sprite::from_image(enemy_handle),
        Transform::from_xyz(0.0, 0.0 + vertical_margin, 0.0),
        RotateToPlayer {
            rotation_speed: f32::to_radians(90.0),
        },
    ));
}

fn imgui_example_ui(mut context: NonSendMut<ImguiContext>, mut state: ResMut<ImguiState>) {
    let ui = context.ui();
    let io = ui.io();
    ui.window("Hello world")
        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text(format!("{:.1} FPS | {:.2} ms per frame", io.framerate, 1.0 / io.framerate));
        });

    if state.demo_window_open {
        ui.show_demo_window(&mut state.demo_window_open);
    }
}

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.into_inner();

    let (mut movement_factor, mut rotation_factor) = (0.0, 0.0);

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        rotation_factor += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        rotation_factor -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        movement_factor += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        movement_factor -= 1.0;
    }

    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());

    let movement_direction = transform.rotation * Vec3::Y;
    let movement_distance = movement_factor * ship.movement_speed * time.delta_secs();
    transform.translation += movement_direction * movement_distance;

    let extents = Vec3::from((BOUNDS / 2.0 - 32.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
}

fn snap2player_system(
    mut query: Query<&mut Transform, (With<SnapToPlayer>, Without<Player>)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let player_translation = player_transform.translation.xy();

    for mut enemy_transform in &mut query {
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();
        let rotate_to_player = Quat::from_rotation_arc(Vec3::Y, to_player.extend(0.0));

        enemy_transform.rotation = rotate_to_player;
    }
}

fn rotate2player_system(
    time: Res<Time>,
    mut query: Query<(&RotateToPlayer, &mut Transform), Without<Player>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let player_translation = player_transform.translation.xy();

    for (config, mut enemy_transform) in &mut query {
        let enemy_forward = (enemy_transform.rotation * Vec3::Y).xy();
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();
        let forward_dot_player = enemy_forward.dot(to_player);

        if (forward_dot_player - 1.0).abs() < f32::EPSILON {
            continue;
        }

        let enemy_right = (enemy_transform.rotation * Vec3::X).xy();
        let right_dot_player = enemy_right.dot(to_player);

        let rotation_sign = -f32::copysign(1.0, right_dot_player);

        let max_angle = ops::acos(forward_dot_player.clamp(-1.0, 1.0));

        let rotation_angle =
            rotation_sign * (config.rotation_speed * time.delta_secs()).min(max_angle);

        enemy_transform.rotate_z(rotation_angle);
    }
}
