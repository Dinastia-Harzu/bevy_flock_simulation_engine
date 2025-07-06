use super::resources::*;
use bevy::{
    math::{FloatPow, NormedVectorSpace},
    prelude::*,
};

pub fn setup_rules(mut rules: ResMut<BoidRules>, mut config: ResMut<BoidConfiguration>) {
    // rules.add(cohesion).add(separation).add(alignment);
    rules.add(separation);
    // rules.add(cohesion);

    config
        .add_scalar_parametre("avoidance_radius", 50.0, 1.0..=100.0)
        .add_scalar_parametre("view_radius", 100.0, 1.0..=200.0);
}

pub fn cohesion(params: BoidRuleParametres, _config: &BoidConfiguration) -> Vec2 {
    let BoidRuleParametres {
        entity,
        position,
        cell,
        ..
    } = params;
    let mut perceived_centre = Vec2::ZERO;
    let mut neighbour_count = 0;
    for other_boid in cell
        .cell_boids()
        .iter()
        .filter(|cell_boid| cell_boid.entity != entity)
    {
        perceived_centre += other_boid.position;
        neighbour_count += 1;
    }
    if neighbour_count > 1 {
        perceived_centre /= neighbour_count as f32;
    }
    (perceived_centre - position) / 100.0
}

pub fn separation(params: BoidRuleParametres, config: &BoidConfiguration) -> Vec2 {
    let BoidRuleParametres {
        entity,
        position,
        cell,
        ..
    } = params;
    let mut push_force = Vec2::ZERO;
    let radius = config.scalar_parametre("avoidance_radius");
    let radius_squared = radius.squared();
    for other_boid in cell
        .cell_boids()
        .iter()
        .filter(|cell_boid| cell_boid.entity != entity)
    {
        let distance_squared = position.distance_squared(other_boid.position);
        if distance_squared < radius_squared {
            let r = other_boid.position - position;
            push_force -= radius_squared
                * if distance_squared < 0.1 {
                    Vec2::Y
                } else {
                    1.0 / distance_squared * r.normalize()
                };
        }
    }
    push_force
}

pub fn alignment(params: BoidRuleParametres, _config: &BoidConfiguration) -> Vec2 {
    let BoidRuleParametres {
        entity,
        velocity,
        cell,
        ..
    } = params;
    let mut perceived_velocity = Vec2::ZERO;
    let mut neighbour_count = 0;
    for other_boid in cell
        .cell_boids()
        .iter()
        .filter(|cell_boid| cell_boid.entity != entity)
    {
        perceived_velocity += other_boid.velocity;
        neighbour_count += 1;
    }
    perceived_velocity /= neighbour_count as f32;
    (perceived_velocity - velocity) / 8.0
}
