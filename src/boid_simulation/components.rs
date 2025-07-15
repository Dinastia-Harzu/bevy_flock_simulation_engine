use std::usize;

use super::resources::*;
use bevy::{math::FloatPow, prelude::*};

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

#[derive(Component)]
pub struct BoidPredator;

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct WindCurrent {
    pub wind_speed: f32,
    pub radius: f32,
    pub trajectory: CubicBezier<Vec2>,
    pub resolution: usize,
}

impl WindCurrent {
    pub fn new(speed: f32, radius: f32, control_points: [Vec2; 4]) -> Self {
        Self {
            wind_speed: speed,
            radius,
            trajectory: CubicBezier::new([control_points]),
            resolution: 100,
        }
    }

    pub fn arrow_resolution(&self) -> usize {
        (self.resolution as f32).sqrt().floor() as usize
    }

    pub fn curve(&self) -> CubicCurve<Vec2> {
        self.trajectory.to_curve().unwrap()
    }

    pub fn control_points(&self) -> &[Vec2] {
        &self.trajectory.control_points[0]
    }

    pub fn closest(&self, position: Vec2) -> Option<(f32, f32, Vec2)> {
        let curve = self.curve();
        let mut closest_t = 0.0f32;
        let mut closest_distance = None;
        for i in 0..self.resolution {
            let t = i as f32 / self.resolution as f32;
            let distance = position.distance(curve.position(t));
            if distance < self.radius.min(closest_distance.unwrap_or(f32::MAX)) {
                closest_distance = Some(distance);
                closest_t = t;
            }
        }
        match closest_distance {
            Some(distance) => Some((closest_t, distance, curve.position(closest_t))),
            None => None,
        }
    }
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct ForceField {
    pub charge: f32
}

impl ForceField {
    pub fn new(charge: f32) -> Self {
        Self { charge }
    }
}
