use super::{components::*, resources::*};
use crate::{asset_related::resources::*, states::*};
use bevy::{color::palettes::css::*, prelude::*};
use core::f32;
use rand::Rng;

pub fn clear_simulation(mut commands: Commands, boids: Query<Entity, With<Boid>>) {
    for entity in boids {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_boids(
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
    for _ in 0..boid_configuration.boid_count {
        let angle = rng.random_range(-pi..=pi);
        let boid = Boid::new(
            (boid_configuration.min_speed + boid_configuration.max_speed) / 2.0,
            angle,
        );
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
    if simulation_configuration.with_predator {
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
            BoidPredator::default(),
        ));
    }

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
    mut boids: Query<(
        Entity,
        &mut Boid,
        &mut Transform,
        Option<&mut BoidTestingUnit>,
        Option<&BoidPredator>,
    )>,
    boid_configuration: Res<BoidConfiguration>,
    spatial_grid: Res<SpatialGrid>,
    rules: Res<BoidRules>,
    time: Res<Time>,
) {
    boids.par_iter_mut().for_each(
        |(entity, mut boid, mut transform, testing_unit, predator_boid)| {
            let Transform {
                translation,
                rotation,
                scale,
            } = &mut *transform;
            let position = translation.xy();
            let cell = spatial_grid.at_world_position(position);
            if (testing_unit.is_none()
                || testing_unit.is_some_and(|testing_unit| testing_unit.follow_boids))
                && predator_boid.is_none()
            {
                let mut velocity = Vec2::ZERO;
                for rule in &*rules {
                    velocity += rule(
                        BoidRuleParametres {
                            entity,
                            position,
                            velocity: boid.velocity(),
                            cell,
                        },
                        &boid_configuration,
                    );
                }
                boid.add_velocity(velocity, &boid_configuration);
                // boid.velocity += velocity;
            } else if predator_boid.is_some() {
                let mut closest = None;
                for other_boid in cell
                    .cell_boids()
                    .iter()
                    .filter(|cell_boid| cell_boid.entity != entity)
                {
                    if position.distance(other_boid.position)
                        < position.distance(closest.unwrap_or(Vec2::MAX))
                    {
                        closest = Some(other_boid.position);
                    }
                }
                let velocity = {
                    let current_velocity = boid.velocity();
                    boid_configuration.scalar_parametre("Predator follow weight")
                        * if let Some(closest) = closest {
                            (closest - position).normalize_or(current_velocity) * boid.speed
                        } else {
                            current_velocity
                        }
                };
                boid.add_velocity(velocity, &boid_configuration);
            }
            *translation += boid.velocity().extend(0.0) * time.delta_secs();
            *rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
            *scale = Vec2::splat(boid_configuration.scale).extend(1.0);
        },
    );
}

pub fn wrap_edges(boids: Query<&mut Transform, With<Boid>>, spatial_grid: Res<SpatialGrid>) {
    for mut transform in boids {
        let Vec2 { x: bx, y: by } = spatial_grid.grid_size() / 2.0;
        let Vec3 { x, y, .. } = &mut transform.translation;
        let safe_offset = 0.1f32;
        if *x >= bx {
            *x = safe_offset - bx;
        } else if *x <= -bx {
            *x = bx - safe_offset;
        }
        if *y >= by {
            *y = safe_offset - by;
        } else if *y <= -by {
            *y = by - safe_offset;
        }
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

pub fn draw_spatial_grid(
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
}
