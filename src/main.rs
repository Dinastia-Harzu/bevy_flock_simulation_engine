use core::f32;

use bevy::{
    color::palettes::css::RED,
    ecs::query::QueryIter,
    math::{FloatPow, NormedVectorSpace},
    prelude::*,
};
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
struct BoidConfig {
    speed: f32,
    angle: f32,
    perception_radius: f32,
}

impl BoidConfig {
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
        .add_systems(PostUpdate, (update_debug_boid, imgui_ui).chain())
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
        let rotation = rng.random_range(-pi..=pi) - f32::consts::FRAC_PI_2;
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
        BoidConfig {
            speed: 0.0,
            angle: 0.0,
            perception_radius: 100.0,
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
}

fn imgui_ui(
    mut commands: Commands,
    mut context: NonSendMut<ImguiContext>,
    state: Res<ImguiState>,
    mut window_query: Query<Entity, With<Window>>,
    boid_query: Single<(&mut BoidConfig, &mut Boid, &mut Transform)>,
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

        let (mut boid_config, mut boid, mut transform) = boid_query.into_inner();
        let pi = f32::consts::PI;
        let mut vel = boid_config.speed;
        let mut angle = boid_config.angle;
        ui.window("Parámetros del boid").build(|| {
            ui.slider("Velocidad", 0.0, BoidConfig::MAX_VEL, &mut vel);
            ui.slider("Ángulo", -pi, pi, &mut angle);
            ui.slider(
                "Percepción",
                0.0,
                1000.0,
                &mut boid_config.perception_radius,
            );
        });
        boid.velocity = vel * Vec2::new(angle.cos(), angle.sin());
        transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
        boid_config.speed = vel;
        boid_config.angle = angle;
    }
}

fn update_boids(
    mut query: Query<(Entity, &mut Boid, &mut Transform)>,
    boid_config_query: Single<&BoidConfig>,
    time: Res<Time>,
) {
    let boid_config = boid_config_query.into_inner();
    let mut align_vel;
    let mut iter1 = query.iter_mut();
    while let Some((entity, mut boid, mut transform)) = iter1.next() {
        align_vel = align(
            entity,
            &boid.velocity,
            &transform.translation.xy(),
            boid_config.perception_radius,
            boid_config.speed,
            iter1.remaining_mut(),
        );
        boid.velocity += align_vel * time.delta_secs();
        transform.translation += boid.velocity.extend(0.0) * time.delta_secs();
        transform.rotation = Quat::from_rotation_z(boid.velocity.to_angle());
    }
}

fn align(
    current_entity: Entity,
    current_velocity: &Vec2,
    current_position: &Vec2,
    perception_radius: f32,
    max_speed: f32,
    mut remaining_iter: QueryIter<'_, '_, (Entity, &mut Boid, &mut Transform), ()>,
) -> Vec2 {
    let mut steer = Vec2::ZERO;
    let mut total = 0;
    while let Some((entity, boid, transform)) = remaining_iter.next() {
        if current_entity.index() != entity.index()
            && current_position.distance_squared(transform.translation.xy())
                <= perception_radius.squared()
        {
            steer += boid.velocity;
            total += 1;
        }
    }
    if total > 0 {
        steer /= total as f32;
        steer = steer.normalize_or_zero() * max_speed;
        steer -= current_velocity;
        steer = if steer.norm_squared() > max_speed.squared() {
            steer.normalize() * max_speed
        } else {
            steer
        };
    }
    steer
}

fn update_debug_boid(
    boid_query: Single<(&BoidConfig, &Transform), With<Boid>>,
    mut gizmos: Gizmos,
) {
    let (boid_config, transform) = boid_query.into_inner();
    gizmos
        .circle_2d(
            transform.translation.xy(),
            boid_config.perception_radius,
            RED,
        )
        .resolution(64);
}
