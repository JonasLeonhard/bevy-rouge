use crate::components::{AnimationConfig, FieldOfView, TurnTaker};
use crate::states::TurnState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::{
    camera::FollowedByCamera,
    map::{GameGrid, GridMovement, GridPos},
};

#[derive(Component)]
pub struct Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, take_turn.run_if(in_state(TurnState::Player)));
}

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // TODO:
    let player_asset = asset_server.load("images/player/idle/idle.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(48, 64), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_config = AnimationConfig::new(0, 7, 10, true);

    commands.spawn((
        Name::new("Player"),
        Player, // TODO: add the rest as required components?
        Transform {
            translation: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.,
            },
            ..default()
        },
        GridMovement {
            current_pos: GridPos { x: 0, y: 0 },
            target_pos: None,
        },
        TurnTaker {
            actions_per_turn: 2, // -- movement takes an action!
            actions_remaining: 2,
        },
        FieldOfView::new(10),
        Sprite {
            image: player_asset,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            }),

            ..default()
        },
        animation_config,
    ));
}

fn take_turn(
    mut movement_query: Query<(Entity, &mut TurnTaker, &mut GridMovement, Option<&Player>)>,
    chunks_query: Query<(
        &TileStorage,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Transform,
    )>,
    tile_query: Query<&TileTextureIndex>,
    mut commands: Commands,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut occupied_positions: Vec<_> = movement_query
        .iter()
        .map(|(entity, _, grid_movement, _)| {
            (entity, grid_movement.current_pos, grid_movement.target_pos)
        })
        .collect();

    let players_to_move: Vec<_> = movement_query
        .iter_mut()
        .filter(|(_, _, grid_movement, player)| {
            player.is_some() && grid_movement.target_pos.is_none()
        })
        .collect();

    let mut direction = None;

    if key.just_pressed(KeyCode::KeyW) {
        direction = Some((0, 1));
    } else if key.just_pressed(KeyCode::KeyS) {
        direction = Some((0, -1));
    } else if key.just_pressed(KeyCode::KeyA) {
        direction = Some((-1, 0));
    } else if key.just_pressed(KeyCode::KeyD) {
        direction = Some((1, 0));
    }

    players_to_move
        .into_iter()
        .for_each(|(entity, mut turn_taker, mut grid_movement, _)| {
            if let Some((dx, dy)) = direction {
                let new_pos = GridPos {
                    x: grid_movement.current_pos.x + dx,
                    y: grid_movement.current_pos.y + dy,
                };

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

                if !is_occupied && GameGrid::is_walkable(&new_pos, &chunks_query, &tile_query) {
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
                    commands.entity(entity).insert(FollowedByCamera);
                }
            }
        });
}
