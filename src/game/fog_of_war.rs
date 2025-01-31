use bevy::{dev_tools::ui_debug_overlay::UiDebugOptions, prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;

use crate::components::{FieldOfView, Player};

use super::{map::GridPos, map::TILE_SIZE};

#[derive(Resource, Default)]
pub struct FogOfWar {
    /// positions the player once had in the fov
    viewed_positions: HashSet<GridPos>,
}

pub fn plugin(app: &mut App) {
    app.insert_resource(FogOfWar::default())
        .add_systems(Update, (update_viewed_positions, render_viewed_positions));
}

/// store all player fov position in fog_of_war.viewed_positions
/// TODO: PERF: only call this when the player fov was recalculated...
fn update_viewed_positions(
    mut fog_of_war: ResMut<FogOfWar>,
    player_query: Query<(&Transform, &FieldOfView)>,
) {
    for (_, fov) in player_query.iter() {
        for pos in &fov.visible_positions {
            fog_of_war.viewed_positions.insert(*pos);
        }
    }
}

fn render_viewed_positions(
    fog_of_war: Res<FogOfWar>,
    player_query: Query<&FieldOfView, With<Player>>,
    mut tile_query: Query<(&TilePos, &mut TileVisible, &mut TileColor, &Parent)>,
    tilemap_query: Query<&Transform>,
    debug_options: Res<UiDebugOptions>,
) {
    if debug_options.enabled {
        // make all tiles visible
        for (_, mut tile_visible, mut tile_color, _) in tile_query.iter_mut() {
            tile_visible.0 = true;
            tile_color.0 = Color::WHITE;
        }

        return;
    }

    let Ok(player_fov) = player_query.get_single() else {
        return;
    };

    for (tile_pos, mut tile_visible, mut tile_color, parent) in tile_query.iter_mut() {
        let Ok(chunk_transform) = tilemap_query.get(parent.get()) else {
            continue;
        };

        let world_pos =
            tile_pos.center_in_world(&TilemapGridSize::from(TILE_SIZE), &TilemapType::Square);
        let world_pos = (chunk_transform.compute_matrix() * world_pos.extend(0.0).extend(1.0)).xy();
        let grid_pos = GridPos::from_world_pos(world_pos);

        if player_fov.visible_positions.contains(&grid_pos) {
            // Fully visible in fov
            tile_visible.0 = true;
            tile_color.0 = Color::WHITE;
        } else if fog_of_war.viewed_positions.contains(&grid_pos) {
            // Partially visible (visited before) - we darken the tile
            tile_visible.0 = true;
            tile_color.0 = Color::srgba(1.0, 1.0, 1.0, 0.25); // render with less opacity!
        } else {
            // Not visible
            tile_visible.0 = false;
            tile_color.0 = Color::WHITE;
        }
    }
}
