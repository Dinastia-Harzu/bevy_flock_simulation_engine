use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, AssetCollection)]
pub struct ImageAssets {
    #[asset(path = "textures/boid.png")]
    pub normal_boid_sprite: Handle<Image>,
    #[asset(path = "textures/boid-seleccionado.png")]
    pub target_boid_sprite: Handle<Image>
}
