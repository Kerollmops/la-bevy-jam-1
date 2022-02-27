use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
pub struct GameAssets {}

#[derive(AssetCollection)]
pub struct BonusesAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    #[asset(path = "images/bonuses.png")]
    pub texture_atlas: Handle<TextureAtlas>,
}

#[derive(AssetCollection)]
pub struct BallAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 1, rows = 1))]
    #[asset(path = "images/ball.png")]
    pub texture_atlas: Handle<TextureAtlas>,
}
