use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::AudioSource;

#[derive(AssetCollection)]
pub struct BonusesAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 4, rows = 5))]
    #[asset(path = "images/bonuses.png")]
    pub texture_atlas: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 2, rows = 1))]
    #[asset(path = "images/shrink_increase_paddle.png")]
    pub paddle_texture_atlas: Handle<TextureAtlas>,
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
pub struct HudAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 4, rows = 1))]
    #[asset(path = "images/hud.png")]
    pub texture_atlas: Handle<TextureAtlas>,
}

#[derive(AssetCollection)]
pub struct VersusAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    #[asset(path = "images/versus.png")]
    pub texture_atlas: Handle<TextureAtlas>,
}

#[derive(AssetCollection)]
pub struct AudioAssets {
    #[asset(path = "sfx/whistle.wav")]
    pub whistle: Handle<AudioSource>,

    #[asset(path = "sfx/powerup_gain.wav")]
    pub powerup_gain: Handle<AudioSource>,

    #[asset(path = "sfx/powerup_spawn.wav")]
    pub powerup_spawn: Handle<AudioSource>,

    #[asset(path = "sfx/goal.wav")]
    pub goal: Handle<AudioSource>,

    #[asset(path = "sfx/hit_0.wav")]
    pub hit_0: Handle<AudioSource>,

    #[asset(path = "sfx/hit_1.wav")]
    pub hit_1: Handle<AudioSource>,

    #[asset(path = "audiotracks/bevyjam.wav")]
    pub track: Handle<AudioSource>,
}
