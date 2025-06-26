use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct ImageAssets {
    #[asset(path = "textures/boid.png")]
    pub boid_sprite: Handle<Image>,
}
