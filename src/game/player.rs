use crate::{
    components::{AnimationConfig, FieldOfView, TurnTaker},
    states::TurnState,
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::{
    camera::FollowedByCamera,
    map::{GameGrid, GridMovement, GridPos, TILE_SIZE},
};

#[derive(Component)]
pub struct Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, (move_player).run_if(in_state(TurnState::Player)));
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
            actions_per_turn: 200, // -- movement takes an action!
            actions_remaining: 200,
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

fn move_player(
    mut player_query: Query<(Entity, &mut TurnTaker, &mut GridMovement), With<Player>>,
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
    let Ok((player_entity, mut player_turn_taker, mut player_grid_movement)) =
        player_query.get_single_mut()
    else {
        return;
    };

    if player_turn_taker.actions_remaining <= 0 || player_grid_movement.target_pos.is_some() {
        return;
    }
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

    if let Some((dx, dy)) = direction {
        let new_pos = GridPos {
            x: player_grid_movement.current_pos.x + dx,
            y: player_grid_movement.current_pos.y + dy,
        };

        if GameGrid::is_walkable(&new_pos, &chunks_query, &tile_query) {
            player_grid_movement.target_pos = Some(new_pos);
            player_turn_taker.actions_remaining -= 1;
            commands.entity(player_entity).insert(FollowedByCamera);
        }
    }
}
