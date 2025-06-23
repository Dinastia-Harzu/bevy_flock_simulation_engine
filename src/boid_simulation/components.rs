use bevy::prelude::*;

#[derive(Component, Clone, Copy, Default)]
pub struct Boid {
    pub speed: f32,
    pub angle: f32,
}

impl Boid {
    pub fn new(speed: f32, angle: f32) -> Self {
        Self { speed, angle }
    }

    pub fn velocity(&self) -> Vec2 {
        Vec2::from_angle(self.angle) * self.speed
    }
}

#[derive(Component, Clone, Copy)]
pub struct BoidTestingUnit;
