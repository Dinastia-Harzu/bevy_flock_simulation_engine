use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct ImageAssets {
    #[asset(path = "textures/wave-fireball.png")]
    pub normal_boid_sprite: Handle<Image>,
    #[asset(path = "textures/wave-blue-fireball.png")]
    pub target_boid_sprite: Handle<Image>
}
