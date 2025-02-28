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
    mut movement_query: Query<(Entity, &mut TurnTaker, &mut GridMovement, Option<&Devil>)>,
    chunks_query: Query<(
        &TileStorage,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Transform,
    )>,
    tile_query: Query<&TileTextureIndex>,
) {
    let mut occupied_positions: Vec<_> = movement_query
        .iter()
        .map(|(entity, _, grid_movement, _)| {
            (entity, grid_movement.current_pos, grid_movement.target_pos)
        })
        .collect();

    let devils_to_move: Vec<_> = movement_query
        .iter_mut()
        .filter(|(_, _, grid_movement, devil)| {
            devil.is_some() && grid_movement.target_pos.is_none()
        })
        .collect();

    devils_to_move
        .into_iter()
        .for_each(|(entity, mut turn_taker, mut grid_movement, _)| {
            let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            let mut rng = rand::rng();
            let mut shuffled_directions = directions.to_vec();
            shuffled_directions.shuffle(&mut rng);

            for (dx, dy) in shuffled_directions {
                let new_pos = GridPos {
                    x: grid_movement.current_pos.x + dx,
                    y: grid_movement.current_pos.y + dy,
                };

                if !GameGrid::is_walkable(&new_pos, &chunks_query, &tile_query) {
                    continue;
                }

                let is_occupied =
                    occupied_positions
                        .iter()
                        .any(|(other_entity, other_current, other_target)| {
                            if *other_entity == entity {
                                return false;
                            }
                            *other_current == new_pos
                                || other_target.is_some_and(|pos| pos == new_pos)
                        });

                if !is_occupied {
                    // update our entities old occupied_position so other entities can't move to
                    // its new position
                    if let Some((_, _, target_pos)) = occupied_positions
                        .iter_mut()
                        .find(|(other_entity, _, _)| *other_entity == entity)
                    {
                        *target_pos = Some(new_pos);
                    }

                    grid_movement.target_pos = Some(new_pos);
                    turn_taker.actions_remaining -= 1;
                    break;
                }
            }

            // we cant move in any direction, but still take an action
            if grid_movement.target_pos.is_none() {
                turn_taker.actions_remaining -= 1;
            }
        });
}
