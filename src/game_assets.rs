use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

pub const LIFEBAR_MIN_INDEX: usize = 0;
pub const LIFEBAR_MAX_INDEX: usize = 15;

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

#[derive(AssetCollection)]
pub struct LifebarAssets {
    #[asset(texture_atlas(tile_size_x = 192., tile_size_y = 16., columns = 1, rows = 16))]
    #[asset(path = "images/lifebar.png")]
    pub texture_atlas: Handle<TextureAtlas>,
}
