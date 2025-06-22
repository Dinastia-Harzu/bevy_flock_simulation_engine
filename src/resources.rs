use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidConfiguration {
    pub speed: f32,
    pub inner_perception_radius: f32,
    pub outer_perception_radius: f32,
    pub separation_factor: f32,
    pub alignment_factor: f32,
    pub cohesion_factor: f32,
}

impl BoidConfiguration {
    pub const MAX_VEL: f32 = 600.0;
    pub const MAX_BOIDS: u32 = 100;
    pub const MAX_INNER_PERCEPTION_RADIUS: f32 = 500.0;
    pub const MAX_OUTER_PERCEPTION_RADIUS: f32 = 2000.0;
    pub const MAX_SEPARATION_FACTOR: f32 = 10.0;
    pub const MAX_ALIGNMENT_FACTOR: f32 = 10.0;
    pub const MAX_COHESION_FACTOR: f32 = 10.0;
}

#[derive(Resource)]
pub struct BoidSprite {
    pub fireball_handle: Handle<Image>,
    pub galaga_ship_handle: Handle<Image>,
    pub size: Vec2,
}
