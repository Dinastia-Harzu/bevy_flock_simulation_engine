use core::f32;
use std::{collections::HashMap, usize};

use bevy::{
    color::palettes::css::*,
    math::{FloatPow, NormedVectorSpace},
    prelude::*,
};
use bevy_mod_imgui::prelude::*;
use rand::Rng;

const SCREEN_SIZE: Vec2 = Vec2::new(1920.0, 1080.0);

#[derive(Resource)]
struct ImguiState {
    common_window: bool,
}

#[derive(Component, Clone, Copy)]
struct Boid {
    speed: f32,
    angle: f32,
}

impl Boid {
    fn velocity(&self) -> Vec2 {
        Vec2::from_angle(self.angle) * self.speed
    }
}

#[derive(Resource)]
struct BoidConfiguration {
    speed: f32,
    inner_perception_radius: f32,
    outer_perception_radius: f32,
    separation_factor: f32,
    alignment_factor: f32,
    cohesion_factor: f32,
}

impl BoidConfiguration {
    const MAX_VEL: f32 = 600.0;
    const MAX_BOIDS: u32 = 100;
    const MAX_INNER_PERCEPTION_RADIUS: f32 = 500.0;
    const MAX_OUTER_PERCEPTION_RADIUS: f32 = 2000.0;
    const MAX_SEPARATION_FACTOR: f32 = 10.0;
    const MAX_ALIGNMENT_FACTOR: f32 = 10.0;
    const MAX_COHESION_FACTOR: f32 = 10.0;
}

#[derive(Resource)]
struct BoidSprite {
    fireball_handle: Handle<Image>,
    galaga_ship_handle: Handle<Image>,
    size: Vec2,
}

#[derive(Component, Clone, Copy)]
struct BoidTestingUnit;

#[derive(Resource)]
struct BoidEntities {
    entities: Vec<Entity>,
    current_id: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Motor de físicas en Rust".into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resolution: SCREEN_SIZE.into(),
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
        .add_systems(Startup, (setup, spawn_boids).chain())
        .add_systems(FixedUpdate, (update_boids, wrap_edges).chain())
        .add_systems(PostUpdate, (update_debug_boid, imgui_ui).chain())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BoidSprite {
        fireball_handle: asset_server.load("textures/wave-fireball.png"),
        galaga_ship_handle: asset_server.load("textures/wave-blue-fireball.png"),
        size: (63.0, 35.0).into(),
    });
    commands.insert_resource(BoidConfiguration {
        speed: 100.0,
        inner_perception_radius: 100.0,
        outer_perception_radius: 500.0,
        separation_factor: 1.0,
        alignment_factor: 1.0,
        cohesion_factor: 1.0,
    });
    let max_boids = BoidConfiguration::MAX_BOIDS as usize;
    commands.insert_resource(BoidEntities {
        entities: Vec::with_capacity(max_boids + 1),
        current_id: max_boids,
    });

    commands.spawn(Camera2d);
}

fn spawn_boids(
    mut commands: Commands,
    boid_configuration: Res<BoidConfiguration>,
    boid_sprite: Res<BoidSprite>,
    mut boid_entities: ResMut<BoidEntities>,
) {
    let mut rng = rand::rng();
    let pi = f32::consts::PI;
    let bounds = SCREEN_SIZE / 2.0;
    for _ in 0..BoidConfiguration::MAX_BOIDS {
        let angle = rng.random_range(-pi..=pi) - f32::consts::FRAC_PI_2;
        let entity = commands
            .spawn((
                Boid {
                    speed: boid_configuration.speed,
                    angle,
                },
                Sprite {
                    image: boid_sprite.fireball_handle.clone(),
                    custom_size: Some(boid_sprite.size),
                    ..Default::default()
                },
                Transform {
                    translation: (
                        rng.random_range(-bounds.x..=bounds.x),
                        rng.random_range(-bounds.y..=bounds.y),
                        0.0,
                    )
                        .into(),
                    rotation: Quat::from_axis_angle(Vec3::Z, angle),
                    ..Default::default()
                },
            ))
            .id();
        boid_entities.entities.push(entity);
    }
    let selected_entity = commands
        .spawn((
            Boid {
                speed: 0.0,
                angle: 0.0,
            },
            Sprite {
                image: boid_sprite.galaga_ship_handle.clone(),
                custom_size: Some(boid_sprite.size),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            BoidTestingUnit,
        ))
        .id();
    boid_entities.entities.push(selected_entity);
}

fn imgui_ui(
    mut commands: Commands,
    mut context: NonSendMut<ImguiContext>,
    state: Res<ImguiState>,
    mut boid_configuration: ResMut<BoidConfiguration>,
    boid_sprite: Res<BoidSprite>,
    mut window_query: Query<Entity, With<Window>>,
    boid_query_opt: Option<Single<(Entity, &mut Boid), With<BoidTestingUnit>>>,
    mut boid_entities: ResMut<BoidEntities>,
    time: Res<Time>,
) {
    if state.common_window {
        let ui = context.ui();
        let dt = time.delta_secs();
        ui.window("Hello world").build(|| {
            ui.text(format!(
                "{:.1} FPS | {:.2} ms per frame",
                1.0 / dt,
                1000.0 * dt
            ));
            if ui.button("Terminar programa") {
                let window = window_query.single_mut();
                commands.entity(window).despawn();
            }
        });

        if let Some(single_boid) = boid_query_opt {
            let (entity, mut boid) = single_boid.into_inner();
            let pi = f32::consts::PI;
            let mut change_selected_boid = false;
            ui.window("Boid controlado").build(|| {
                if ui.arrow_button("##previous", Direction::Left) {
                    boid_entities.current_id = if boid_entities.current_id == 0 {
                        boid_entities.entities.len() - 1
                    } else {
                        boid_entities.current_id - 1
                    };
                    change_selected_boid = true;
                }
                ui.same_line();
                if ui.arrow_button("##next", Direction::Right) {
                    boid_entities.current_id =
                        if boid_entities.current_id == boid_entities.entities.len() - 1 {
                            0
                        } else {
                            boid_entities.current_id + 1
                        };
                    change_selected_boid = true;
                }
                ui.same_line();
                ui.text(format!("Entidad actual: {}", entity.index()));
                ui.slider(
                    "Velocidad",
                    0.0,
                    BoidConfiguration::MAX_VEL,
                    &mut boid_configuration.speed,
                );
                ui.slider("Ángulo", -pi, pi, &mut boid.angle);
                ui.slider(
                    "Percepción exterior",
                    boid_configuration.inner_perception_radius,
                    BoidConfiguration::MAX_OUTER_PERCEPTION_RADIUS,
                    &mut boid_configuration.outer_perception_radius,
                );
                ui.slider(
                    "Percepción interior",
                    0.0,
                    BoidConfiguration::MAX_INNER_PERCEPTION_RADIUS,
                    &mut boid_configuration.inner_perception_radius,
                );
                ui.slider(
                    "Factor de separación",
                    0.0,
                    BoidConfiguration::MAX_SEPARATION_FACTOR,
                    &mut boid_configuration.separation_factor,
                );
                ui.slider(
                    "Factor de alineamiento",
                    0.0,
                    BoidConfiguration::MAX_ALIGNMENT_FACTOR,
                    &mut boid_configuration.alignment_factor,
                );
                ui.slider(
                    "Factor de cohesión",
                    0.0,
                    BoidConfiguration::MAX_COHESION_FACTOR,
                    &mut boid_configuration.cohesion_factor,
                );
            });
            if change_selected_boid {
                commands
                    .entity(entity)
                    .remove::<BoidTestingUnit>()
                    .remove::<Sprite>()
                    .insert(Sprite {
                        image: boid_sprite.fireball_handle.clone(),
                        custom_size: Some(boid_sprite.size),
                        ..Default::default()
                    });
                commands
                    .entity(boid_entities.entities[boid_entities.current_id])
                    .insert(BoidTestingUnit)
                    .remove::<Sprite>()
                    .insert(Sprite {
                        image: boid_sprite.galaga_ship_handle.clone(),
                        custom_size: Some(boid_sprite.size),
                        ..Default::default()
                    });
            }
        }
    }
}

fn update_boids(
    mut query: Query<(Entity, &mut Boid, &mut Transform)>,
    boid_configuration: Res<BoidConfiguration>,
    time: Res<Time>,
) {
    let mut steerings = HashMap::new();
    let mut combinations = query.iter_combinations();
    while let Some([(entity1, boid1, transform1), (entity2, boid2, transform2)]) =
        combinations.next()
    {
        let pos1 = transform1.translation.xy();
        let pos2 = transform2.translation.xy();
        let initial_values = (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0f32, 0.0f32);
        let pos1_to_pos2_squared = pos1.distance_squared(pos2);
        let pos1_to_pos2 = pos1.distance(pos2);
        let in_inner_radius =
            pos1_to_pos2_squared <= boid_configuration.inner_perception_radius.squared();
        let in_outer_radius =
            pos1_to_pos2_squared <= boid_configuration.outer_perception_radius.squared();

        let (cohesion1, separation1, alignment1, total_outer1, total_inner1) =
            steerings.entry(entity1).or_insert(initial_values);
        if in_outer_radius {
            *cohesion1 += pos2;
            *alignment1 += boid2.velocity();
            *total_outer1 += 1.0;
        }
        if in_inner_radius {
            let other_to_current = pos1 - pos2;
            *separation1 += other_to_current / pos1_to_pos2;
            *total_inner1 += 1.0;
        }

        let (cohesion2, separation2, alignment2, total_outer2, total_inner2) =
            steerings.entry(entity2).or_insert(initial_values);
        if in_outer_radius {
            *cohesion2 += pos1;
            *alignment2 += boid1.velocity();
            *total_outer2 += 1.0;
        }
        if in_inner_radius {
            let other_to_current = pos2 - pos1;
            *separation2 += other_to_current / pos1_to_pos2;
            *total_inner2 += 1.0;
        }
    }
    for (entity, mut boid, mut transform) in &mut query {
        if let Some((cohesion, separation, alignment, total_outer, total_inner)) =
            steerings.get_mut(&entity)
        {
            *cohesion /= *total_outer;
            *cohesion = (*cohesion - transform.translation.xy()) / 100.0;
            *cohesion *= boid_configuration.cohesion_factor;

            *separation /= *total_inner;
            *separation *= boid_configuration.separation_factor;

            *alignment /= *total_outer;
            *alignment = (*alignment - boid.velocity()) / 8.0;
            *alignment *= boid_configuration.alignment_factor;

            let mut velocity = boid.velocity();
            velocity += *cohesion + *alignment + *separation;
            // velocity += *cohesion + *alignment;
            // velocity += *separation;
            velocity =
                velocity.normalize_or(Vec2::from_angle(boid.angle)) * boid_configuration.speed;

            boid.speed = velocity.norm();
            boid.angle = velocity.to_angle();
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
            transform.rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
        }
    }
}

fn wrap_edges(mut query: Query<&mut Transform, With<Boid>>) {
    for mut transform in &mut query {
        let position = &mut transform.translation;
        let Vec2 { x: bx, y: by } = SCREEN_SIZE / 2.0;
        if position.x > bx {
            position.x = -bx;
        } else if position.x < -bx {
            position.x = bx;
        }
        if position.y > by {
            position.y = -by;
        } else if position.y < -by {
            position.y = by;
        }
    }
}

fn update_debug_boid(
    boid_query: Option<Single<&Transform, With<BoidTestingUnit>>>,
    boid_configuration: Res<BoidConfiguration>,
    mut gizmos: Gizmos,
) {
    if let Some(boid) = boid_query {
        let transform = boid.into_inner();
        gizmos
            .circle_2d(
                transform.translation.xy(),
                boid_configuration.inner_perception_radius,
                RED,
            )
            .resolution(64);
        gizmos
            .circle_2d(
                transform.translation.xy(),
                boid_configuration.outer_perception_radius,
                GREEN,
            )
            .resolution(64);
    }
}
