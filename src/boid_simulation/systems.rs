use super::{components::*, resources::*};
use crate::{asset_related::resources::*, states::*};
use bevy::{
    color::palettes::css::*,
    prelude::*,
};
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
                Sprite {
                    image: image_assets.boid_sprite.clone(),
                    color: Color::srgb(1.0, 0.2, 0.2),
                    ..default()
                },
                transform,
            ))
            .id();
        let position = transform.translation.xy();
        spatial_grid
            .at_world_position_mut(position)
            .push(SpatialGridBoid::new(boid_entity, position, boid.velocity()));
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

// pub fn apply_rules(mut commands: Commands, boids: Query<(Entity, &Transform, &Boid)>) {
//     for (entity, transform, boid) in boids {
//         commands.trigger_targets(
//             ApplyRules::new(transform.translation.xy(), boid.velocity()),
//             entity,
//         );
//     }
// }

pub fn update_boids(
    boids: Query<(
        Entity,
        &mut Boid,
        &mut Transform,
        Option<&mut BoidTestingUnit>,
    )>,
    _boid_configuration: Res<BoidConfiguration>,
    spatial_grid: Res<SpatialGrid>,
    rules: Res<BoidRules>,
    time: Res<Time>,
) {
    for (entity, mut boid, mut transform, testing_unit) in boids {
        if testing_unit.is_none()
            || testing_unit.is_some_and(|testing_unit| testing_unit.follow_boids)
        {
            let mut vel = Vec2::ZERO;
            // let mut average_velocity = Vec2::ZERO;
            // let mut nearby_boids = 0;
            let position = transform.translation.xy();
            let cell = spatial_grid.at_world_position(position);
            // for cell_boid in cell
            //     .cell_boids()
            //     .iter()
            //     .filter(|cell_boid| cell_boid.entity != entity)
            // {
            //     average_velocity += cell_boid.velocity;
            //     nearby_boids += 1;
            // }
            for rule in &*rules {
                vel += rule(BoidRuleParametres {
                    entity,
                    position,
                    velocity: boid.velocity(),
                    cell,
                });
            }
            boid.set_velocity(vel);
            // if nearby_boids > 1 {
            //     average_velocity /= nearby_boids as f32;
            //     if average_velocity.length_squared() > boid_configuration.threshold {
            //         let current_dir = boid.velocity().normalize_or_zero();
            //         let force_direction = average_velocity.normalize_or_zero();
            //         let new_dir = current_dir.lerp(force_direction, 0.03).normalize_or_zero();
            //         let new_velocity = new_dir * boid.velocity().length();
            //         boid.speed = new_velocity.length();
            //         boid.angle = new_velocity.to_angle();
            //     }
            // }
        }
        transform.translation += boid.velocity().extend(0.0) * time.delta_secs();
        transform.rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
    }
}

// pub fn update_boids_OLD(
//     mut query: Query<(
//         Entity,
//         &mut Boid,
//         &mut Transform,
//         Option<&mut BoidTestingUnit>,
//     )>,
//     boid_configuration: Res<BoidConfiguration>,
//     time: Res<Time>,
// ) {
//     let mut steerings = HashMap::new();
//     let mut combinations = query.iter_combinations();
//     while let Some(
//         [(entity1, boid1, transform1, testing_unit1), (entity2, boid2, transform2, testing_unit2)],
//     ) = combinations.next()
//     {
//         if testing_unit1.is_some_and(|testing_unit1| !testing_unit1.follow_boids)
//             || testing_unit2.is_some_and(|testing_unit2| !testing_unit2.follow_boids)
//         {
//             continue;
//         }
//         let pos1 = transform1.translation.xy();
//         let pos2 = transform2.translation.xy();
//         let initial_values = (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0f32, 0.0f32);
//         let pos1_to_pos2_squared = pos1.distance_squared(pos2);
//         let pos1_to_pos2 = pos1.distance(pos2);
//         let in_inner_radius =
//             pos1_to_pos2_squared <= boid_configuration.inner_perception_radius.squared();
//         let in_outer_radius =
//             pos1_to_pos2_squared <= boid_configuration.outer_perception_radius.squared();
//
//         let (cohesion1, separation1, alignment1, total_outer1, total_inner1) =
//             steerings.entry(entity1).or_insert(initial_values);
//         if in_outer_radius {
//             *cohesion1 += pos2;
//             *alignment1 += boid2.velocity();
//             *total_outer1 += 1.0;
//         }
//         if in_inner_radius {
//             let other_to_current = pos1 - pos2;
//             *separation1 += other_to_current / pos1_to_pos2;
//             *total_inner1 += 1.0;
//         }
//
//         let (cohesion2, separation2, alignment2, total_outer2, total_inner2) =
//             steerings.entry(entity2).or_insert(initial_values);
//         if in_outer_radius {
//             *cohesion2 += pos1;
//             *alignment2 += boid1.velocity();
//             *total_outer2 += 1.0;
//         }
//         if in_inner_radius {
//             let other_to_current = pos2 - pos1;
//             *separation2 += other_to_current / pos1_to_pos2;
//             *total_inner2 += 1.0;
//         }
//     }
//     for (entity, mut boid, mut transform, _testing_unit) in &mut query {
//         if let Some((cohesion, separation, alignment, total_outer, total_inner)) =
//             steerings.get_mut(&entity)
//         {
//             *cohesion /= *total_outer;
//             *cohesion = (*cohesion - transform.translation.xy()) / 100.0;
//             *cohesion *= boid_configuration.cohesion_factor;
//
//             *separation /= *total_inner;
//             *separation *= boid_configuration.separation_factor;
//
//             *alignment /= *total_outer;
//             *alignment = (*alignment - boid.velocity()) / 8.0;
//             *alignment *= boid_configuration.alignment_factor;
//
//             let mut velocity = boid.velocity();
//             velocity += *cohesion + *alignment + *separation;
//             // velocity += *cohesion + *alignment;
//             // velocity += *separation;
//             velocity =
//                 velocity.normalize_or(Vec2::from_angle(boid.angle)) * boid_configuration.speed;
//
//             boid.speed = velocity.norm();
//             boid.angle = velocity.to_angle();
//             transform.translation += (velocity * time.delta_secs()).extend(0.0);
//             transform.rotation = Quat::from_axis_angle(Vec3::Z, boid.angle);
//         }
//     }
// }

pub fn wrap_edges(boids: Query<&mut Transform, With<Boid>>, spatial_grid: Res<SpatialGrid>) {
    for mut transform in boids {
        let Vec2 { x: bx, y: by } = spatial_grid.grid_size() / 2.0;
        let Vec3 { x, y, .. } = &mut transform.translation;
        if *x > bx {
            *x = -bx;
        } else if *x < -bx {
            *x = bx;
        }
        if *y > by {
            *y = -by;
        } else if *y < -by {
            *y = by;
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

    if let Some(boid) = testing_unit_boid {
        let (transform, mut sprite) = boid.into_inner();
        let position = transform.translation.xy();
        sprite.color = Color::srgb(0.3, 0.3, 1.0);
        gizmos
            .circle_2d(position, boid_configuration.inner_perception_radius, RED)
            .resolution(64);
        gizmos
            .circle_2d(position, boid_configuration.outer_perception_radius, GREEN)
            .resolution(64);
    }
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
