use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::map::{GameGrid, GridPos, TILE_SIZE};
use doryen_fov::{FovAlgorithm, FovRestrictive, MapData};

// Component for entities that have field of view
#[derive(Component)]
pub struct FieldOfView {
    pub view_range: usize,
    fov: FovRestrictive,
    fov_map: MapData,
    pub visible_positions: Vec<GridPos>,
}

impl FieldOfView {
    pub fn new(view_range: usize) -> Self {
        let grid_size = view_range * 2 + 1;
        Self {
            view_range,
            fov: FovRestrictive::new(),
            fov_map: MapData::new(grid_size, grid_size),
            visible_positions: Vec::new(),
        }
    }

    /// Get all grid positions within view range of a center position
    pub fn get_positions_in_view_range(&self, center: &GridPos) -> Vec<GridPos> {
        let range = self.view_range as i32;
        let mut positions = Vec::with_capacity(((range * 2 + 1) * (range * 2 + 1)) as usize);

        for y in -range..=range {
            for x in -range..=range {
                positions.push(GridPos {
                    x: center.x + x,
                    y: center.y + y,
                });
            }
        }
        positions
    }

    /// Update field of view from current position
    pub fn update(
        &mut self,
        position: Vec2,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) {
        self.fov_map.clear_fov();
        let center_pos = GridPos::from_world_pos(position);
        let positions_in_view_range = self.get_positions_in_view_range(&center_pos);

        // Set all positions that are in the FOV
        for pos in &positions_in_view_range {
            // all walkable positions are transparent
            let is_transparent = GameGrid::is_walkable(pos, chunks_query, tile_query);

            // Convert to FOV grid coordinates
            let fov_x = ((pos.x - center_pos.x) + self.view_range as i32) as usize;
            let fov_y = ((pos.y - center_pos.y) + self.view_range as i32) as usize;

            self.fov_map.set_transparent(fov_x, fov_y, is_transparent);
        }

        // Compute FOV from center
        let center = self.view_range;
        self.fov
            .compute_fov(&mut self.fov_map, center, center, self.view_range, true);

        // Update visible positions
        // Update visible positions, including blocking tiles that are in view
        self.visible_positions = positions_in_view_range
            .into_iter()
            .filter(|pos| {
                let fov_x = ((pos.x - center_pos.x) + self.view_range as i32) as usize;
                let fov_y = ((pos.y - center_pos.y) + self.view_range as i32) as usize;

                self.fov_map.is_in_fov(fov_x, fov_y)
            })
            .collect();
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
        let all_positions = fov.get_positions_in_view_range(&center_pos);

        for pos in all_positions {
            let world_pos = pos.to_world_pos();
            let color = if fov.visible_positions.contains(&pos) {
                Color::srgba(0.0, 1.0, 0.0, 0.2)
            } else {
                Color::srgba(1.0, 0.0, 0.0, 0.1)
            };

            gizmos.rect_2d(world_pos, Vec2::new(TILE_SIZE.x, TILE_SIZE.y), color);
        }
    }
}
