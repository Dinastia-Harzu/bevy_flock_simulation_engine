use super::{
    bundles::{BoidBundle, ForceFieldBundle, WindCurrentBundle},
    components::*,
    resources::*,
};
use crate::{asset_related::resources::*, helpers::*, states::*};
use bevy::{color::palettes::css::*, math::FloatPow, prelude::*};
use core::f32;
use itertools::Itertools;
use rand::Rng;

pub fn setup_boid_parametres(mut config: ResMut<BoidConfiguration>) {
    config
        .add_scalar_parametre("Radio de separación", 50.0, 1.0..=100.0)
        .add_scalar_parametre("Radio de visión", 100.0, 1.0..=200.0)
        .add_scalar_parametre("Peso de cohesión", 0.25, 0.0..=1.0)
        .add_scalar_parametre("Peso de separación", 1.0, 0.0..=5.0)
        .add_scalar_parametre("Peso de alineamiento", 0.125, 0.0..=1.0)
        .add_scalar_parametre("Ángulo del viento", -120.0, -180.0..=180.0)
        .add_scalar_parametre(
            "Velocidad del viento",
            100.0,
            0.0..=(BoidConfiguration::highest_speed() * 2.0),
        )
        .add_scalar_parametre("Peso de huida", 0.5, 0.0..=1.0);
}

pub fn clear_simulation(
    mut commands: Commands,
    simulation_entities: Query<Entity, Or<(With<Boid>, With<WindCurrent>, With<ForceField>)>>,
) {
    for entity in simulation_entities {
        commands.entity(entity).despawn();
    }
}

pub fn setup_simulation(
    mut commands: Commands,
    boid_configuration: Res<BoidConfiguration>,
    simulation_configuration: Res<SimulationConfiguration>,
    spatial_grid: Res<SpatialGrid>,
    image_assets: Res<ImageAssets>,
    mut app_next_state: ResMut<NextState<SimulationState>>,
) {
    let mut rng = rand::rng();
    let pi = f32::consts::PI;
    let bounds = spatial_grid.grid_size() / 2.0;
    let scale = Vec3::ONE;
    for _ in 0..simulation_configuration.normal_boids {
        let angle = rng.random_range(-pi..=pi);
        let boid = Boid::new(boid_configuration.average_speed(), angle);
        commands.spawn(
            BoidBundle::start()
                .name("Boid")
                .boid(boid_configuration.average_speed(), angle)
                .sprite(image_assets.boid_sprite.clone(), Color::srgb(0.1, 1.0, 0.2))
                .transform(
                    angle,
                    Vec2::new(
                        rng.random_range(-bounds.x..=bounds.x),
                        rng.random_range(-bounds.y..=bounds.y),
                    ),
                )
                .build(),
        );
    }
    commands.spawn((
        BoidBundle::start()
            .name("Boid de pruebas")
            .sprite(image_assets.boid_sprite.clone(), Color::srgb(0.1, 0.1, 1.0))
            .build(),
        BoidTestingUnit::default(),
    ));

    // Predators
    for _ in 0..simulation_configuration.predators {
        let angle = rng.random_range(-pi..=pi);
        commands.spawn((
            BoidBundle::start()
                .name("Boid depredador")
                .sprite(image_assets.boid_sprite.clone(), Color::srgb(1.0, 0.2, 0.2))
                .transform(
                    angle,
                    Vec2::new(
                        rng.random_range(-bounds.x..=bounds.x),
                        rng.random_range(-bounds.y..=bounds.y),
                    ),
                )
                .build(),
            BoidPredator,
        ));
    }

    // Wind currents
    commands.spawn(WindCurrentBundle::new(
        100.0,
        100.0,
        [
            vec2(-10.0, -200.0),
            vec2(30.0, 20.0),
            vec2(350.0, 30.0),
            vec2(390.0, 80.0),
        ],
    ));

    // Force fields
    commands.spawn(ForceFieldBundle::new(120.0, Vec2::new(-550.0, 200.0)));
    commands.spawn(ForceFieldBundle::new(80.0, Vec2::new(-400.0, 220.0)));
    commands.spawn(ForceFieldBundle::new(-120.0, Vec2::new(550.0, -200.0)));

    // Switch to next state
    app_next_state.set(SimulationState::Running);
}

pub fn update_spatial_grid(
    boids: Query<(Entity, &Transform, &Boid)>,
    mut spatial_grid: ResMut<SpatialGrid>,
) {
    spatial_grid.clear();
    for (entity, transform, boid) in boids {
        let position = transform.translation.xy();
        spatial_grid
            .at_world_position_mut(position)
            .push(SpatialGridBoid::new(entity, position, boid.velocity()));
    }
}

pub fn update_boids(
    mut boids: Query<
        (Entity, &mut Boid, &mut Transform, Option<&BoidTestingUnit>),
        Without<BoidPredator>,
    >,
    mut boid_predators: Query<
        (Entity, &mut Boid, &mut Transform),
        (With<BoidPredator>, Without<BoidTestingUnit>),
    >,
    wind_currents: Query<&WindCurrent>,
    force_fields: Query<(&Transform, &ForceField), Without<Boid>>,
    boid_configuration: Res<BoidConfiguration>,
    simulation_configuration: Res<SimulationConfiguration>,
    spatial_grid: Res<SpatialGrid>,
    time: Res<Time>,
) {
    boids
        .par_iter_mut()
        .for_each(|(entity, mut boid, mut transform, testing_unit)| {
            let Transform {
                translation,
                rotation,
                scale,
            } = &mut *transform;
            let position = translation.xy();
            let mut velocity = Vec2::ZERO;
            let mut offset_velocity = Vec2::ZERO;

            if testing_unit.is_none()
                || testing_unit.is_some_and(|testing_unit| testing_unit.follow_boids)
            {
                // Common
                let mut perceived_centre = OVec2::default();
                let mut perceived_velocity = OVec2::default();
                let mut neighbours_to_follow = 0;
                let mut push_force = Vec2::ZERO;
                let view_radius = boid_configuration.scalar_parametre("Radio de visión");
                let view_radius_squared = view_radius.squared();
                let avoidance_radius = boid_configuration.scalar_parametre("Radio de separación");
                let avoidance_radius_squared = avoidance_radius.squared();
                for cell in spatial_grid.iter_radius(position, view_radius) {
                    for other_boid in cell
                        .cell_boids()
                        .iter()
                        .filter(|cell_boid| cell_boid.entity != entity)
                    {
                        let distance_squared = position.distance_squared(other_boid.position);
                        let r = other_boid.position - position;
                        if boid_predators.contains(other_boid.entity) {
                            if distance_squared < view_radius_squared {
                                push_force -= boid_configuration.scalar_parametre("Peso de huida")
                                    * r.normalize_or_zero()
                                    * boid.speed;
                            }
                        } else {
                            if distance_squared < avoidance_radius_squared {
                                push_force -= (boid_configuration
                                    .scalar_parametre("Peso de separación")
                                    * avoidance_radius_squared
                                    * r.normalize_or(boid.velocity())
                                    / if distance_squared < 0.1 {
                                        1.0
                                    } else {
                                        distance_squared
                                    })
                                .clamp_length_max(avoidance_radius_squared);
                            } else if distance_squared < view_radius_squared {
                                perceived_centre += other_boid.position;
                                perceived_velocity += other_boid.velocity;
                                neighbours_to_follow += 1;
                            }
                        }
                    }
                }
                if neighbours_to_follow > 1 {
                    let neighbours_to_follow = neighbours_to_follow as f32;
                    perceived_centre /= neighbours_to_follow;
                    perceived_velocity /= neighbours_to_follow;
                }

                // Force fields
                for (ff_point, ff) in force_fields {
                    let point = ff_point.translation.xy();
                    let distance = point.distance(position);
                    let charge = ff.charge.abs();
                    push_force += if distance <= charge {
                        ff.charge.signum()
                            * (position - point).normalize_or(boid.velocity().normalize())
                            * boid.speed
                            * (1.0 - (1.0 - (distance / charge - 1.0).squared()).sqrt())
                    } else {
                        Vec2::ZERO
                    };
                }

                // Cohesion
                velocity += (perceived_centre.get().unwrap_or(position) - position)
                    * boid_configuration.scalar_parametre("Peso de cohesión");

                // Separation
                velocity += push_force;

                // Alignment
                velocity += (perceived_velocity.get().unwrap_or(velocity) - velocity)
                    * boid_configuration.scalar_parametre("Peso de alineamiento");

                // Strong wind
                offset_velocity += Vec2::from_angle(
                    boid_configuration
                        .scalar_parametre("Ángulo del viento")
                        .to_radians(),
                ) * boid_configuration.scalar_parametre("Velocidad del viento");

                // Wind currents
                for wind_current in wind_currents {
                    if let Some((t, distance, _)) = wind_current.closest(position) {
                        let curve = wind_current.curve();
                        offset_velocity +=
                            curve.velocity(t).normalize_or_zero() * wind_current.wind_speed;
                    }
                }
            }

            boid.add_velocity(velocity, &boid_configuration);
            *translation += (boid.velocity() + offset_velocity).extend(0.0) * time.delta_secs();
            *rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
            *scale = Vec2::splat(boid_configuration.scale).extend(1.0);
        });
    boid_predators
        .par_iter_mut()
        .for_each(|(entity, mut boid, mut transform)| {
            let Transform {
                translation,
                rotation,
                scale,
            } = &mut *transform;
            let position = translation.xy();
            let mut velocity = Vec2::ZERO;
            let mut offset_velocity = Vec2::ZERO;

            let mut push_force = Vec2::ZERO;
            let mut closest = None;
            let view_radius = boid_configuration.scalar_parametre("Radio de visión");
            for cell in spatial_grid.iter_radius(position, view_radius) {
                for other_boid in cell
                    .cell_boids()
                    .iter()
                    .filter(|cell_boid| cell_boid.entity != entity)
                {
                    let distance = position.distance(other_boid.position);
                    if boids.contains(other_boid.entity) {
                        if distance
                            < view_radius.min(position.distance(closest.unwrap_or(Vec2::MAX)))
                        {
                            closest = Some(other_boid.position);
                        }
                    } else {
                        let avoidance_radius_squared = boid_configuration
                            .scalar_parametre("Radio de separación")
                            .squared();
                        let distance_squared = distance.squared();
                        let r = other_boid.position - position;
                        if distance_squared < avoidance_radius_squared {
                            push_force -= (boid_configuration
                                .scalar_parametre("Peso de separación")
                                * avoidance_radius_squared
                                * r.normalize_or(boid.velocity())
                                / if distance_squared < 0.1 {
                                    1.0
                                } else {
                                    distance_squared
                                })
                            .clamp_length_max(avoidance_radius_squared);
                        }
                    }
                }
            }
            // Force fields
            for (ff_point, ff) in force_fields {
                let point = ff_point.translation.xy();
                let distance = point.distance(position);
                let charge = ff.charge.abs();
                push_force += if distance <= charge {
                    ff.charge.signum()
                        * (position - point).normalize_or(boid.velocity().normalize())
                        * boid.speed
                        * (1.0 - (1.0 - (distance / charge - 1.0).squared()).sqrt())
                } else {
                    Vec2::ZERO
                };
            }

            // Separation
            velocity += push_force;

            // Hunt
            velocity += {
                let current_velocity = boid.velocity();
                simulation_configuration.predator_hunt_weight
                    * if let Some(closest) = closest {
                        (closest - position).normalize_or(current_velocity) * boid.speed
                    } else {
                        current_velocity
                    }
            };

            // Strong wind
            offset_velocity += Vec2::from_angle(
                boid_configuration
                    .scalar_parametre("Ángulo del viento")
                    .to_radians(),
            ) * boid_configuration.scalar_parametre("Velocidad del viento");

            // Wind currents
            for wind_current in wind_currents {
                if let Some((t, distance, _)) = wind_current.closest(position) {
                    let curve = wind_current.curve();
                    offset_velocity +=
                        curve.velocity(t).normalize_or_zero() * wind_current.wind_speed;
                }
            }

            boid.add_velocity(velocity, &boid_configuration);
            *translation += (boid.velocity() + offset_velocity).extend(0.0) * time.delta_secs();
            *rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
            *scale = Vec2::splat(boid_configuration.scale).extend(1.0);
        });
}

pub fn wrap_edges(boids: Query<&mut Transform, With<Boid>>, spatial_grid: Res<SpatialGrid>) {
    for mut transform in boids {
        let safe_offset = Vec2::splat(0.1f32);
        let bounds = spatial_grid.grid_size() / 2.0;
        let Vec2 { x: bx, y: by } = bounds - safe_offset;
        let Vec3 { x, y, .. } = &mut transform.translation;
        x.toroidal_clamp(-bx, bx);
        y.toroidal_clamp(-by, by);
    }
}

pub fn draw_debug(
    wind_currents: Query<&WindCurrent>,
    testing_unit_boid: Option<Single<(&Transform, &mut Sprite), With<BoidTestingUnit>>>,
    force_fields: Query<(&Transform, &ForceField)>,
    spatial_grid: Res<SpatialGrid>,
    boid_configuration: Res<BoidConfiguration>,
    simulation_configuration: Res<SimulationConfiguration>,
    mut gizmos: Gizmos,
) {
    if !simulation_configuration.should_draw {
        return;
    }

    // Boid testing unit
    let Some(boid) = testing_unit_boid else {
        return;
    };
    let (transform, mut sprite) = boid.into_inner();
    let position = transform.translation.xy();
    sprite.color = Color::srgb(0.3, 0.3, 1.0);
    gizmos
        .circle_2d(
            position,
            boid_configuration.scalar_parametre("Radio de separación"),
            RED,
        )
        .resolution(64);
    gizmos
        .circle_2d(
            position,
            boid_configuration.scalar_parametre("Radio de visión"),
            GREEN,
        )
        .resolution(64);

    // Spatial grid
    for cell in spatial_grid.cells() {
        gizmos.rect_2d(
            cell.location(),
            Vec2::splat(spatial_grid.cell_size()),
            WHITE,
        );
    }
    for cell in spatial_grid.iter_radius(
        position,
        boid_configuration.scalar_parametre("Radio de visión"),
    ) {
        for r in 1..=4 {
            gizmos.circle_2d(cell.location(), (r * 20) as f32, ORANGE);
        }
    }

    // Wind currents
    for wind_current in wind_currents {
        let curve = wind_current.curve();
        for (i, &point) in wind_current.control_points().iter().enumerate() {
            gizmos.circle_2d(point, 10.0, YELLOW);
            if i == 0 {
                gizmos.circle_2d(point, wind_current.radius, PURPLE);
            }
        }
        for (start, end) in curve
            .iter_positions(wind_current.arrow_resolution())
            .tuple_windows::<(_, _)>()
        {
            gizmos.arrow_2d(start, end, WHITE);
        }
    }

    // Force fields
    {
        let pieces = 12;
        let angle_step = 360.0 / pieces as f32;
        for (ff_point, ff) in force_fields {
            let point = ff_point.translation.xy();
            for i in 0..pieces {
                let end =
                    point + ff.charge * Vec2::from_angle(((i as f32) * angle_step).to_radians());
                if ff.charge.is_sign_positive() {
                    gizmos.arrow_2d(point, end, RED);
                } else {
                    gizmos.arrow_2d(end, point, BLUE);
                }
            }
        }
    }
}
