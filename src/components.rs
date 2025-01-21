use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use doryen_fov::{FovAlgorithm, FovRestrictive, MapData};

use crate::game::grid::{GameGrid, GridPos};

#[derive(Component)]
pub struct Player;

// TODO: implement actions
#[derive(Component)]
pub struct TurnTaker {
    pub actions_per_turn: u32,
    pub actions_remaining: u32,
}

impl Default for TurnTaker {
    fn default() -> Self {
        Self {
            actions_per_turn: 1,
            actions_remaining: 1,
        }
    }
}

#[derive(Component)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
    pub should_loop: bool,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8, should_loop: bool) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps, should_loop),
            should_loop,
        }
    }

    pub fn timer_from_fps(fps: u8, should_loop: bool) -> Timer {
        let duration = Duration::from_secs_f32(1.0 / (fps as f32));
        let mode = if should_loop {
            TimerMode::Repeating
        } else {
            TimerMode::Once
        };
        Timer::new(duration, mode)
    }
}

#[derive(Component)]
pub struct HighlightBorder;

// Component for entities that have field of view
#[derive(Component)]
pub struct FieldOfView {
    pub view_range: usize,
    fov: FovRestrictive,
    fov_map: MapData,
    visible_positions: Vec<GridPos>,
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
    pub fn get_fov_positions(&self, center: &GridPos) -> Vec<GridPos> {
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
        let positions = self.get_fov_positions(&center_pos);

        // Update transparency map
        for pos in &positions {
            let is_transparent = GameGrid::is_walkable(pos, chunks_query, tile_query);

            // Convert to FOV grid coordinates
            let fov_x = ((pos.x - center_pos.x) + self.view_range as i32) as usize;
            let fov_y = ((pos.y - center_pos.y) + self.view_range as i32) as usize;

            self.fov_map.set_transparent(fov_x, fov_y, is_transparent);
        }

        // Compute FOV from center
        let center = self.view_range;
        self.fov
            .compute_fov(&mut self.fov_map, center, center, self.view_range, false);

        // Update visible positions
        self.visible_positions = positions
            .into_iter()
            .filter(|pos| {
                let fov_x = ((pos.x - center_pos.x) + self.view_range as i32) as usize;
                let fov_y = ((pos.y - center_pos.y) + self.view_range as i32) as usize;
                self.fov_map.is_in_fov(fov_x, fov_y)
            })
            .collect();
    }

    /// Get currently visible positions
    pub fn visible_positions(&self) -> &[GridPos] {
        &self.visible_positions
    }

    /// Check if a specific position is visible
    pub fn is_position_visible(&self, pos: &GridPos) -> bool {
        self.visible_positions.contains(pos)
    }
}
