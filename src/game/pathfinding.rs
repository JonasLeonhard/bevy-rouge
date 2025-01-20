use super::map::TILE_SIZE;
use bevy::prelude::*;
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

    fn successors(&self) -> Vec<(GridPosition, i32)> {
        let directions = [
            (0, 1),  // up
            (1, 0),  // right
            (0, -1), // down
            (-1, 0), // left
        ];

        directions
            .iter()
            .map(|&(dx, dy)| GridPosition::new(self.x + dx, self.y + dy))
            .filter(|pos| Self::is_visitable(pos))
            .map(|pos| (pos, 1)) // Pathfinding cost of 1 for each step
            .collect()
    }

    fn heuristic(&self, goal: &GridPosition) -> i32 {
        self.manhattan_distance(goal)
    }

    fn is_visitable(_pos: &GridPosition) -> bool {
        true // TODO: currently all tiles are visitible
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
pub fn find_path(from: Vec2, to: Vec2) -> Option<Vec<Vec2>> {
    let start = GridPosition::from_world_pos(from);
    let goal = GridPosition::from_world_pos(to);

    let result = astar(
        &start,
        |p| p.successors(),
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
