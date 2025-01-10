use bevy::prelude::*;

use crate::components::TilePosition;

pub const MAP_SIZE: usize = 8;
pub const MAP_Z_INDEX: f32 = 0.;
pub const TILE_SIZE: f32 = 32.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_map);
}

// Systems:
pub fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("images/brick_dark0.png");

    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile_pos = IVec2::new(x as i32, y as i32);
            let tile_translation = Vec3::new(
                TILE_SIZE * tile_pos.x as f32,
                TILE_SIZE * tile_pos.y as f32,
                MAP_Z_INDEX,
            );

            commands.spawn((
                Sprite {
                    image: texture_handle.clone(),
                    ..default()
                },
                Transform {
                    translation: tile_translation,
                    ..Default::default()
                },
                TilePosition(tile_pos),
            ));
        }
    }
}

// Helpers:
// pub fn is_on_map(v: IVec2) -> bool {
//     v.x >= 0 && v.y >= 0 && v.x < MAP_SIZE as i32 && v.y < MAP_SIZE as i32
// }
