use bevy::prelude::*;

use super::resources::BoidConfiguration;

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Boid {
    pub speed: f32,
    pub angle: f32,
}

impl Boid {
    pub fn new(speed: f32, angle: f32) -> Self {
        Self { speed, angle }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
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
    pub fn new(follow_boids: bool) -> Self {
        Self { follow_boids }
    }
}

impl Default for BoidTestingUnit {
    fn default() -> Self {
        Self::new(true)
    }
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct BoidPredator {
    pub follow_weight: f32
}

impl BoidPredator {
    pub fn new(follow_weight: f32) -> Self {
        Self { follow_weight }
    }
}

impl Default for BoidPredator {
    fn default() -> Self {
        Self::new(0.1)
    }
}
