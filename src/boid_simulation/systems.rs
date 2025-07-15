use super::{components::*, resources::*};
use crate::{asset_related::resources::*, helpers::*, miscellaneous::*, states::*};
use bevy::{color::palettes::css::*, math::FloatPow, prelude::*};
use core::f32;
use itertools::Itertools;
use rand::Rng;

pub fn clear_simulation(
    mut commands: Commands,
    simulation_entities: Query<Entity, Or<(With<Boid>, With<WindCurrent>)>>,
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
        commands.spawn((
            Name::from("Boid"),
            boid,
            Sprite {
                image: image_assets.boid_sprite.clone(),
                color: Color::srgb(0.1, 1.0, 0.2),
                ..default()
            },
            Transform::from_scale(scale)
                .with_rotation(Quat::from_axis_angle(Vec3::Z, angle))
                .with_translation(Vec3::new(
                    rng.random_range(-bounds.x..=bounds.x),
                    rng.random_range(-bounds.y..=bounds.y),
                    0.0,
                )),
        ));
    }
    commands.spawn((
        Name::from("Boid objetivo"),
        Boid::default(),
        Sprite {
            image: image_assets.boid_sprite.clone(),
            color: Color::srgb(0.1, 0.1, 1.0),
            ..default()
        },
        Transform::from_scale(scale).with_translation(Vec3::Z),
        BoidTestingUnit::default(),
    ));

    // Predator(s)
    for _ in 0..simulation_configuration.predators {
        let angle = rng.random_range(-pi..=pi);
        commands.spawn((
            Name::from("Boid depredador"),
            Boid::default().with_angle(angle),
            Sprite {
                image: image_assets.boid_sprite.clone(),
                color: Color::srgb(1.0, 0.2, 0.2),
                ..default()
            },
            Transform::from_scale(scale)
                .with_rotation(Quat::from_axis_angle(Vec3::Z, angle))
                .with_translation(Vec3::new(
                    rng.random_range(-bounds.x..=bounds.x),
                    rng.random_range(-bounds.y..=bounds.y),
                    0.0,
                )),
            BoidPredator,
        ));
    }

    // Wind currents
    for _ in 0..1 {
        commands.spawn((
            Name::from("Wind current"),
            WindCurrent::new(
                100.0,
                100.0,
                [
                    vec2(-10.0, -200.0),
                    vec2(30.0, 20.0),
                    vec2(350.0, 30.0),
                    vec2(390.0, 80.0),
                ],
            ),
        ));
    }

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
                let view_radius = boid_configuration.scalar_parametre("view_radius");
                let view_radius_squared = view_radius.squared();
                let avoidance_radius = boid_configuration.scalar_parametre("avoidance_radius");
                let avoidance_radius_squared = avoidance_radius.squared();
                // let cell = spatial_grid.at_world_position(position);
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
                                push_force -= boid_configuration.scalar_parametre("Flee weight")
                                    * r.normalize_or_zero()
                                    * boid.speed;
                            }
                        } else {
                            if distance_squared < avoidance_radius_squared {
                                push_force -= (boid_configuration
                                    .scalar_parametre("separation_weight")
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

                // Cohesion
                velocity += (perceived_centre.get().unwrap_or(position) - position)
                    * boid_configuration.scalar_parametre("cohesion_weight");

                // Separation
                velocity += push_force;

                // Alignment
                velocity += (perceived_velocity.get().unwrap_or(velocity) - velocity)
                    * boid_configuration.scalar_parametre("alignment_weight");

                // Strong wind
                offset_velocity += Vec2::from_angle(
                    boid_configuration
                        .scalar_parametre("wind_angle")
                        .to_radians(),
                ) * boid_configuration.scalar_parametre("wind_speed");

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
            let cell = spatial_grid.at_world_position(position);
            let mut velocity = Vec2::ZERO;
            let mut offset_velocity = Vec2::ZERO;

            let mut push_force = Vec2::ZERO;
            let mut closest = None;
            for other_boid in cell
                .cell_boids()
                .iter()
                .filter(|cell_boid| cell_boid.entity != entity)
            {
                let distance = position.distance(other_boid.position);
                if boids.contains(other_boid.entity) {
                    if distance
                        < boid_configuration
                            .scalar_parametre("view_radius")
                            .min(position.distance(closest.unwrap_or(Vec2::MAX)))
                    {
                        closest = Some(other_boid.position);
                    }
                } else {
                    let avoidance_radius_squared = boid_configuration
                        .scalar_parametre("avoidance_radius")
                        .squared();
                    let distance_squared = distance.squared();
                    let r = other_boid.position - position;
                    if distance_squared < avoidance_radius_squared {
                        push_force -= (boid_configuration.scalar_parametre("separation_weight")
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
                    .scalar_parametre("wind_angle")
                    .to_radians(),
            ) * boid_configuration.scalar_parametre("wind_speed");

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

pub fn update_debug_boid(
    testing_unit_boid: Option<Single<(&Transform, &mut Sprite), With<BoidTestingUnit>>>,
    boid_configuration: Res<BoidConfiguration>,
    simulation_configuration: Res<SimulationConfiguration>,
    mut gizmos: Gizmos,
) {
    if !simulation_configuration.should_draw {
        return;
    }

    let Some(boid) = testing_unit_boid else {
        return;
    };
    let (transform, mut sprite) = boid.into_inner();
    let position = transform.translation.xy();
    sprite.color = Color::srgb(0.3, 0.3, 1.0);
    gizmos
        .circle_2d(
            position,
            boid_configuration.scalar_parametre("avoidance_radius"),
            RED,
        )
        .resolution(64);
    gizmos
        .circle_2d(
            position,
            boid_configuration.scalar_parametre("view_radius"),
            GREEN,
        )
        .resolution(64);
}

pub fn draw_debug(
    wind_currents: Query<&WindCurrent>,
    spatial_grid: Res<SpatialGrid>,
    simulation_configuration: Res<SimulationConfiguration>,
    mut gizmos: Gizmos,
) {
    if !simulation_configuration.should_draw {
        return;
    }

    let cell_size = spatial_grid.cell_size();
    for cell in spatial_grid.cells() {
        gizmos.rect_2d(cell.location(), Vec2::new(cell_size, cell_size), WHITE);
    }
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
}
