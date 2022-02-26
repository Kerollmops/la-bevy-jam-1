use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
pub struct GameAssets {
    // #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 8, rows = 4))]
// #[asset(path = "images/RedSelector01.png")]
// pub red_selector_01: Handle<TextureAtlas>,
}

#[derive(Default)]
pub struct BallAssets {
    pub mesh: Mesh2dHandle,
    pub color_material: Handle<ColorMaterial>,
}
