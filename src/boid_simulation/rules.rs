use super::resources::*;
use bevy::prelude::*;

pub fn setup_rules(mut rules: ResMut<BoidRules>) {
    rules.add(cohesion).add(|_| Vec2::Y * -100.0);
}

pub fn cohesion(params: BoidRuleParametres) -> Vec2 {
    let BoidRuleParametres {
        entity,
        position,
        velocity,
        cell,
    } = params;
    Vec2::X * 50.0
}
