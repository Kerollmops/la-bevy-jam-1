use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::AudioSource;

#[derive(AssetCollection)]
pub struct BonusesAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 4, rows = 5))]
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

#[derive(AssetCollection)]
pub struct SpacebarAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 16., columns = 1, rows = 3))]
    #[asset(path = "images/spacebar.png")]
    pub texture_atlas: Handle<TextureAtlas>,
    pub loop_animation: Handle<SpriteSheetAnimation>,
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    #[asset(path = "sfx/hit_0.wav")]
    pub hit_0: Handle<AudioSource>,

    #[asset(path = "sfx/blip.wav")]
    pub blip: Handle<AudioSource>,
}
