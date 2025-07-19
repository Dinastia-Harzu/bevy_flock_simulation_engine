use super::components::*;
use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct BoidBundle {
    name: Name,
    boid: Boid,
    sprite: Sprite,
    transform: Transform,
}

impl BoidBundle {
    pub fn start() -> BoidBundleBuilder {
        BoidBundleBuilder(Self::default())
    }
}

pub struct BoidBundleBuilder(BoidBundle);

impl BoidBundleBuilder {
    pub fn build(self) -> BoidBundle {
        self.0
    }

    pub fn name(mut self, name: &str) -> Self {
        self.0.name = Name::from(name);
        self
    }

    pub fn boid(mut self, speed: f32, angle: f32) -> Self {
        self.0.boid = Boid::new(speed, angle);
        self
    }

    pub fn sprite(mut self, image: Handle<Image>, colour: Color) -> Self {
        self.0.sprite = Sprite {
            image,
            color: colour,
            ..default()
        };
        self
    }

    pub fn transform(mut self, angle: f32, position: Vec2) -> Self {
        self.0.transform = Transform::from_scale(Vec3::ONE)
            .with_rotation(Quat::from_axis_angle(Vec3::Z, angle))
            .with_translation(position.extend(0.0));
        self
    }
}

#[derive(Bundle)]
pub struct WindCurrentBundle {
    name: Name,
    wind_current: WindCurrent,
}

impl WindCurrentBundle {
    pub fn new(speed: f32, radius: f32, control_points: [Vec2; 4]) -> Self {
        Self {
            name: Name::from("Corriente de viento"),
            wind_current: WindCurrent::new(speed, radius, control_points),
        }
    }
}

#[derive(Bundle)]
pub struct ForceFieldBundle {
    name: Name,
    force_field: ForceField,
    transform: Transform,
}

impl ForceFieldBundle {
    pub fn new(charge: f32, position: Vec2) -> Self {
        Self {
            name: Name::from("Campo de fuerza"),
            force_field: ForceField::new(charge),
            transform: Transform::from_translation(position.extend(0.0)),
        }
    }
}
