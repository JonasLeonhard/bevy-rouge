use crate::{
    components::{Player, TilePosition, TilePositionOccupied, TurnTaker},
    resources::TurnState,
};
use bevy::prelude::*;

use crate::game::map::TILE_SIZE;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, (on_input).run_if(in_state(TurnState::Player)));
}

// TODO: we should probably call this in the generation code?
fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // TODO:
    let player_asset = asset_server.load("images/player.png");
    commands.spawn((
        Player, // TODO: add the rest as required components?
        Transform {
            translation: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.,
            },
            ..default()
        },
        TilePosition(IVec2::ZERO),
        TilePositionOccupied,
        TurnTaker {
            actions_per_turn: 1,
            actions_remaining: 1,
        },
        Sprite {
            image: player_asset,
            ..default()
        },
    ));
}

fn on_input(
    mut player_query: Query<(&mut Transform, &mut TilePosition), With<Player>>,
    movable_tile_query: Query<&TilePosition, (Without<Player>, Without<TilePositionOccupied>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, mut map_pos) = player_query.single_mut();
    let mut new_position = map_pos.0.clone();

    if keys.just_pressed(KeyCode::KeyW) {
        new_position.y += 1;
    } else if keys.just_pressed(KeyCode::KeyS) {
        new_position.y -= 1;
    } else if keys.just_pressed(KeyCode::KeyA) {
        new_position.x -= 1;
    } else if keys.just_pressed(KeyCode::KeyD) {
        new_position.x += 1;
    }

    if new_position == map_pos.0 {
        return; // We haven't moved
    }

    let can_move_to_new_position = movable_tile_query.iter().any(|pos| pos.0 == new_position);

    if !can_move_to_new_position {
        return;
    }

    // TODO: animate to target position?
    // move the player
    map_pos.0 = new_position;
    transform.translation.x = new_position.x as f32 * TILE_SIZE;
    transform.translation.y = new_position.y as f32 * TILE_SIZE;

    // TODO: animate to target position?
    // recenter the camera on the player after moving
    if let Ok(mut cam_transform) = camera_query.get_single_mut() {
        cam_transform.translation.x = new_position.x as f32 * TILE_SIZE;
        cam_transform.translation.y = new_position.y as f32 * TILE_SIZE;
    }
}
