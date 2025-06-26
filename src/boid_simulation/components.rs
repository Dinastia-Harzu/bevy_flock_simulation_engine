use bevy::prelude::*;

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

    pub fn velocity(&self) -> Vec2 {
        Vec2::from_angle(self.angle) * self.speed
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
