use core::f32;

use bevy::{math::NormedVectorSpace, prelude::*};
use bevy_mod_imgui::prelude::*;
use rand::Rng;

const BOUNDS: Vec2 = Vec2::new(1920.0, 1080.0);
const MAX_BOIDS: u32 = 100;

#[derive(Resource)]
struct ImguiState {
    common_window: bool,
}

#[derive(Component)]
struct Boid {
    velocity: Vec2,
}

#[derive(Component)]
struct BoidTestingUnit {
    speed: f32,
    angle: f32,
}

impl BoidTestingUnit {
    const MAX_VEL: f32 = 600.0;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Motor de físicas en Rust".into(),
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
            common_window: true,
        })
        .add_systems(Startup, (setup).chain())
        .add_systems(FixedUpdate, update_boids)
        .add_systems(PostUpdate, imgui_ui)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let boid_handle = asset_server.load("textures/wave_46.png");

    commands.spawn(Camera2d);

    spawn_boids(&mut commands, &boid_handle);
}

fn spawn_boids(commands: &mut Commands, boids_handle: &Handle<Image>) {
    let mut rng = rand::rng();
    let pi = f32::consts::PI;
    for _ in 0..MAX_BOIDS {
        // let vel = rng.random_range(0.0_f32..=60.0_f32);
        let vel = 60.0;
        let rotation = rng.random_range(-pi..=pi);
        commands.spawn((
            Sprite::from_image(boids_handle.clone()),
            Boid {
                velocity: vel * Vec2::new(rotation.cos(), rotation.sin()),
            },
            Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, rotation)),
        ));
    }
    commands.spawn((
        Sprite::from_image(boids_handle.clone()),
        Boid {
            velocity: Vec2::ZERO,
        },
        BoidTestingUnit {
            speed: 0.0,
            angle: 0.0,
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}

fn imgui_ui(
    mut commands: Commands,
    mut context: NonSendMut<ImguiContext>,
    state: Res<ImguiState>,
    mut window_query: Query<Entity, With<Window>>,
    mut boid_query: Single<(&BoidTestingUnit, &mut Boid, &mut Transform)>,
) {
    if state.common_window {
        let ui = context.ui();
        let io = ui.io();
        ui.window("Hello world").build(|| {
            ui.text(format!(
                "{:.1} FPS | {:.2} ms per frame",
                1.0 / io.delta_time,
                1000.0 * io.delta_time
            ));
            if ui.button("Terminar programa") {
                let window = window_query.single_mut();
                commands.entity(window).despawn();
            }
        });

        let (_, mut boid, mut transform) = boid_query.into_inner();
        let pi = f32::consts::PI;
        let mut vel = boid.velocity.norm();
        let mut angle = if vel == 0.0 {
            0.0
        } else {
            boid.velocity.to_angle()
        };
        ui.window("Parámetros del boid").build(|| {
            ui.slider("Velocidad", 0.0, BoidTestingUnit::MAX_VEL, &mut vel);
            ui.new_line();
            ui.slider("Ángulo", -pi, pi, &mut angle);
        });
        boid.velocity = vel * Vec2::new(angle.cos(), angle.sin());
        transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
    }
}

fn update_boids(mut query: Query<(&mut Boid, &mut Transform)>, time: Res<Time>) {
    for (mut boid, mut transform) in &mut query {
        // boid.velocity += accel * time.delta_secs();
        transform.translation += boid.velocity.extend(0.0) * time.delta_secs();
    }
}
