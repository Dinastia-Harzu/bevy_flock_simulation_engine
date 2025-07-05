use bevy::prelude::*;

use super::resources::BoidConfiguration;

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Boid {
    pub speed: f32,
    pub angle: f32,
    pub velocity: Vec2,
}

impl Boid {
    pub fn new(speed: f32, angle: f32) -> Self {
        let velocity = Vec2::from_angle(angle) * speed;
        Self { speed, angle, velocity }
    }

    pub fn velocity(&self) -> Vec2 {
        Vec2::from_angle(self.angle) * self.speed
    }

    pub fn set_velocity(&mut self, velocity: Vec2, config: &BoidConfiguration) {
        self.speed = velocity.length().clamp(config.min_speed, config.max_speed);
        self.angle = velocity.to_angle();
    }

    pub fn add_velocity(&mut self, velocity: Vec2, config: &BoidConfiguration) {
        self.set_velocity(self.velocity() + velocity, config);
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct BoidTestingUnit {
    pub follow_boids: bool,
}

impl BoidTestingUnit {
    fn new(follow_boids: bool) -> Self {
        Self { follow_boids }
    }
}

impl Default for BoidTestingUnit {
    fn default() -> Self {
        Self::new(true)
    }
}
