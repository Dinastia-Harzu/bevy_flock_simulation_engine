use super::{components::*, resources::*};
use crate::{asset_related::resources::*, constants::*, states::SimulationState};
use bevy::{
    color::palettes::css::*,
    math::{FloatPow, NormedVectorSpace},
    prelude::*,
};
use core::f32;
use rand::Rng;
use std::collections::HashMap;

pub fn clear_simulation(mut commands: Commands, boids: Query<Entity, With<Boid>>) {
    for entity in boids {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_boids(
    mut commands: Commands,
    boid_configuration: Res<BoidConfiguration>,
    mut spatial_grid: ResMut<SpatialGrid>,
    image_assets: Res<ImageAssets>,
    mut app_next_state: ResMut<NextState<SimulationState>>,
) {
    let mut rng = rand::rng();
    let pi = f32::consts::PI;
    let bounds = spatial_grid.grid_size() / 2.0;
    let scale = Vec3::ONE;
    for _ in 0..BoidConfiguration::MAX_BOIDS {
        let angle = rng.random_range(-pi..=pi) - f32::consts::FRAC_PI_2;
        let transform = Transform::from_scale(scale)
            .with_rotation(Quat::from_axis_angle(Vec3::Z, angle))
            .with_translation(Vec3::new(
                rng.random_range(-bounds.x..=bounds.x),
                rng.random_range(-bounds.y..=bounds.y),
                0.0,
            ));
        let boid = Boid::new(boid_configuration.speed, angle);
        let boid_entity = commands
            .spawn((
                Name::from("Boid"),
                boid,
                Sprite::from_image(image_assets.normal_boid_sprite.clone()),
                transform,
            ))
            .id();
        spatial_grid
            .at_world_position_mut(transform.translation.xy())
            .push(SpatialGridBoid::new(boid_entity, boid.velocity()));
    }
    commands.spawn((
        Name::from("Boid objetivo"),
        Boid::default(),
        Sprite::from_image(image_assets.target_boid_sprite.clone()),
        Transform::from_scale(scale).with_translation(Vec3::Z),
        BoidTestingUnit::default(),
    ));

    app_next_state.set(SimulationState::Running);
}

pub fn update_boids(
    mut boids: Query<(
        Entity,
        &mut Boid,
        &mut Transform,
        Option<&mut BoidTestingUnit>,
    )>,
    boid_configuration: Res<BoidConfiguration>,
    time: Res<Time>,
) {
}

pub fn update_boids_OLD(
    mut query: Query<(
        Entity,
        &mut Boid,
        &mut Transform,
        Option<&mut BoidTestingUnit>,
    )>,
    boid_configuration: Res<BoidConfiguration>,
    time: Res<Time>,
) {
    let mut steerings = HashMap::new();
    let mut combinations = query.iter_combinations();
    while let Some(
        [(entity1, boid1, transform1, testing_unit1), (entity2, boid2, transform2, testing_unit2)],
    ) = combinations.next()
    {
        if testing_unit1.is_some_and(|testing_unit1| !testing_unit1.follow_boids)
            || testing_unit2.is_some_and(|testing_unit2| !testing_unit2.follow_boids)
        {
            continue;
        }
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
    for (entity, mut boid, mut transform, _testing_unit) in &mut query {
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

pub fn wrap_edges(mut query: Query<&mut Transform, With<Boid>>) {
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

pub fn update_debug_boid(
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

pub fn draw_spatial_grid(spatial_grid: Res<SpatialGrid>, mut gizmos: Gizmos) {
    let cell_size = spatial_grid.cell_size();
    // gizmos.grid_2d(
    //     Isometry2d::IDENTITY,
    //     (spatial_grid.columns(), spatial_grid.rows()).into(),
    //     (cell_size, cell_size).into(),
    //     WHITE,
    // );
    for cell in spatial_grid.cells() {
        gizmos.rect_2d(cell.location(), Vec2::new(cell_size, cell_size), WHITE);
        gizmos.circle_2d(cell.location(), 1.0, WHITE);
    }
}
