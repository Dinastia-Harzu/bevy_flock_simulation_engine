use super::resources::*;
use bevy::prelude::*;

pub fn setup_rules(mut rules: ResMut<BoidRules>) {
    rules.add(cohesion).add(separation).add(alignment);
}

pub fn cohesion(params: BoidRuleParametres) -> Vec2 {
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
    perceived_centre /= neighbour_count as f32;
    (perceived_centre - position) / 100.0
}

pub fn separation(params: BoidRuleParametres) -> Vec2 {
    let BoidRuleParametres {
        entity,
        position,
        cell,
        ..
    } = params;
    let mut c = Vec2::ZERO;
    for other_boid in cell
        .cell_boids()
        .iter()
        .filter(|cell_boid| cell_boid.entity != entity)
    {
        if other_boid.position.distance_squared(position) < 35.0 {
            c -= other_boid.position - position;
        }
    }
    c
}

pub fn alignment(params: BoidRuleParametres) -> Vec2 {
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
