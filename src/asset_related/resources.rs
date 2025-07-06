use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct ImageAssets {
    #[asset(key = "image.boid")]
    pub boid_sprite: Handle<Image>,
}
