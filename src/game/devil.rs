use super::map::GameGrid;
use crate::{
    components::TurnTaker,
    game::map::{GridMovement, GridPos, TILE_SIZE},
    states::TurnState,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::seq::SliceRandom;

#[derive(Component)]
struct Devil;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, take_turn.run_if(in_state(TurnState::Environment)));
}

pub fn spawn_devil(commands: &mut Commands, asset_server: &AssetServer, world_pos: Vec3) {
    let image = asset_server.load("images/devil.png");
    commands.spawn((
        Name::new("Devil"),
        Devil,
        Transform {
            translation: world_pos,
            ..default()
        },
        GridMovement {
            current_pos: GridPos::from_world_pos(world_pos.xy()),
            target_pos: None,
        },
        TurnTaker {
            actions_per_turn: 1,
            actions_remaining: 1,
        },
        Visibility::Hidden,
        Sprite {
            image,
            custom_size: Some(Vec2 {
                x: TILE_SIZE.x,
                y: TILE_SIZE.y,
            }),
            ..default()
        },
    ));
}

fn take_turn(
    mut devil_query: Query<(&mut TurnTaker, &mut GridMovement), With<Devil>>,
    chunks_query: Query<(
        &TileStorage,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Transform,
    )>,
    tile_query: Query<&TileTextureIndex>,
) {
    for (mut turn_taker, mut grid_movement) in devil_query.iter_mut() {
        if grid_movement.target_pos.is_some() {
            return; // Still animating movement, unable to take another action
        }

        // Possible movement directions: right, left, up, down
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        // Shuffle directions and try them until we find a valid move
        let mut rng = rand::thread_rng();
        let mut shuffled_directions = directions.to_vec();
        shuffled_directions.shuffle(&mut rng);

        for (dx, dy) in shuffled_directions {
            let new_pos = GridPos {
                x: grid_movement.current_pos.x + dx,
                y: grid_movement.current_pos.y + dy,
            };

            if GameGrid::is_walkable(&new_pos, &chunks_query, &tile_query) {
                grid_movement.target_pos = Some(new_pos);
                turn_taker.actions_remaining -= 1;
                break;
            }
        }

        // If no valid move was found, still consume the action
        if grid_movement.target_pos.is_none() {
            turn_taker.actions_remaining -= 1;
        }
    }
}
