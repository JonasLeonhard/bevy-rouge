use super::map::TILE_SIZE;
use bevy::prelude::*;
use bevy_ecs_tilemap::{
    map::{TilemapGridSize, TilemapSize, TilemapTexture, TilemapType},
    tiles::{TilePos, TileStorage, TileTextureIndex},
};
use pathfinding::prelude::astar;

// ----- PATHFINDING ON THE MAP ----------
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn successors(
        &self,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> Vec<(GridPosition, i32)> {
        let directions = [
            (0, 1),  // up
            (1, 0),  // right
            (0, -1), // down
            (-1, 0), // left
        ];

        directions
            .iter()
            .map(|&(dx, dy)| GridPosition::new(self.x + dx, self.y + dy))
            .filter(|pos| Self::is_visitable(chunks_query, tile_query, pos))
            .map(|pos| (pos, 1)) // Pathfinding cost of 1 for each step
            .collect()
    }

    fn heuristic(&self, goal: &GridPosition) -> i32 {
        self.manhattan_distance(goal)
    }

    fn is_visitable(
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
        pos: &GridPosition,
    ) -> bool {
        let world_pos = pos.to_world_pos();

        let found_visitible_tile_in_loaded_chunks =
            chunks_query
                .iter()
                .find(|(tile_storage, map_size, grid_size, map_type, transform)| {
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

                    texture_index.0 == 5 // TODO: texture_index 5 is walkable for now... We need a more robust system here
                });

        found_visitible_tile_in_loaded_chunks.is_some()
    }

    pub fn from_world_pos(world_pos: Vec2) -> Self {
        Self {
            x: (world_pos.x / TILE_SIZE.x).floor() as i32,
            y: (world_pos.y / TILE_SIZE.y).floor() as i32,
        }
    }

    // returns x,y of (center of tile)
    pub fn to_world_pos(&self) -> Vec2 {
        Vec2::new(self.x as f32 * TILE_SIZE.x, self.y as f32 * TILE_SIZE.y)
    }

    fn manhattan_distance(&self, other: &GridPosition) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// Find a path using AStar
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
    let start = GridPosition::from_world_pos(from);
    let goal = GridPosition::from_world_pos(to);

    let result = astar(
        &start,
        |p| p.successors(chunks_query, tile_query),
        |p| p.heuristic(&goal),
        |p| p == &goal,
    );

    // Convert the result back to world positions
    result.map(|(path, _cost)| {
        path.into_iter()
            .map(|grid_pos| grid_pos.to_world_pos())
            .collect()
    })
}
