use crate::components::FieldOfView;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use pathfinding::prelude::astar;

use super::map::TILE_SIZE;

// each tile is our world has a Grid Position that can be calculated from a World Position
// this is a basic building block for pathfinding and fov calculations
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn from_world_pos(world_pos: Vec2) -> Self {
        Self {
            x: (world_pos.x / TILE_SIZE.x).floor() as i32,
            y: (world_pos.y / TILE_SIZE.y).floor() as i32,
        }
    }

    pub fn to_world_pos(&self) -> Vec2 {
        Vec2::new(self.x as f32 * TILE_SIZE.x, self.y as f32 * TILE_SIZE.y)
    }

    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

// grid helper struct for pathfinding & position management
pub struct GameGrid;

impl GameGrid {
    /// Returns whether a position is walkable based on tile's at the position
    pub fn is_walkable(
        pos: &GridPos,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> bool {
        let world_pos = pos.to_world_pos();
        chunks_query
            .iter()
            .any(|(tile_storage, map_size, grid_size, map_type, transform)| {
                let pos_in_chunk: Vec2 = {
                    let pos = Vec4::from((world_pos, 0.0, 1.0));
                    let pos_in_chunk = transform.compute_matrix().inverse() * pos;
                    pos_in_chunk.xy()
                };

                let Some(tile_pos) =
                    TilePos::from_world_pos(&pos_in_chunk, map_size, grid_size, map_type)
                else {
                    return false;
                };

                let Some(tile_entity) = tile_storage.get(&tile_pos) else {
                    return false;
                };

                let Ok(texture_index) = tile_query.get(tile_entity) else {
                    return false;
                };

                texture_index.0 == 5 // TODO: Make this configurable
            })
    }

    fn get_successors(
        pos: &GridPos,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> Vec<(GridPos, i32)> {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        directions
            .iter()
            .map(|&(dx, dy)| GridPos {
                x: pos.x + dx,
                y: pos.y + dy,
            })
            .filter(|pos| Self::is_walkable(pos, chunks_query, tile_query))
            .map(|pos| (pos, 1))
            .collect()
    }

    /// Find a path between two world positions
    pub fn find_path(
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
        from: Vec2,
        to: Vec2,
    ) -> Option<Vec<Vec2>> {
        let start = GridPos::from_world_pos(from);
        let goal = GridPos::from_world_pos(to);

        let result = astar(
            &start,
            |p| Self::get_successors(p, chunks_query, tile_query),
            |p| p.manhattan_distance(&goal),
            |p| p == &goal,
        );

        result.map(|(path, _cost)| {
            path.into_iter()
                .map(|grid_pos| grid_pos.to_world_pos())
                .collect()
        })
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_fov)
        .add_systems(Update, debug_fov);
}

// System to update FOV for all entities that have one
fn update_fov(
    mut query: Query<(&Transform, &mut FieldOfView)>,
    chunks_query: Query<(
        &TileStorage,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Transform,
    )>,
    tile_query: Query<&TileTextureIndex>,
) {
    for (transform, mut fov) in query.iter_mut() {
        fov.update(transform.translation.xy(), &chunks_query, &tile_query);
    }
}

// TODO: probably toggle this in dev_tools?
fn debug_fov(mut gizmos: Gizmos, query: Query<(&Transform, &FieldOfView)>) {
    for (transform, fov) in query.iter() {
        let center_pos = GridPos::from_world_pos(transform.translation.xy());
        let all_positions = fov.get_fov_positions(&center_pos);
        let visible_positions = fov.visible_positions();

        for pos in all_positions {
            let world_pos = pos.to_world_pos();
            let color = if visible_positions.contains(&pos) {
                Color::srgba(0.0, 1.0, 0.0, 0.2)
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.1)
            };

            gizmos.rect_2d(world_pos, Vec2::new(TILE_SIZE.x, TILE_SIZE.y), color);
        }
    }
}
