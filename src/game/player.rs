use crate::{
    components::{AnimationConfig, Player, TurnTaker},
    resources::{HoveredTilePos, PlayerTargetPos},
    states::TurnState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(PlayerTargetPos(None))
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (on_click_set_target_pos, move_player)
                .chain()
                .run_if(in_state(TurnState::Player)),
        );
}

// TODO: we should probably call this in the generation code?
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
        TurnTaker {
            actions_per_turn: 2, // -- movement takes an action!
            actions_remaining: 2,
        },
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

fn on_click_set_target_pos(
    key: Res<ButtonInput<MouseButton>>,
    hovered_tile_pos: Res<HoveredTilePos>,
    mut target_pos: ResMut<PlayerTargetPos>,
) {
    if key.just_pressed(MouseButton::Left) {
        if let Some(tile_pos) = hovered_tile_pos.0 {
            target_pos.0 = Some(tile_pos);
        }
    }
}

fn move_player(
    mut player_query: Query<(&mut Transform, &mut TurnTaker), With<Player>>,
    mut target_pos: ResMut<PlayerTargetPos>,
) {
    if let Some(target) = target_pos.0 {
        let Ok((mut transform, mut turn_taker)) = player_query.get_single_mut() else {
            return;
        };

        if turn_taker.actions_remaining > 0 {
            // Move the player to the target position
            transform.translation.x = target.x;
            transform.translation.y = target.y;

            // Moving Consumes an action
            turn_taker.actions_remaining -= 1;

            // TODO: Only Clear the target position if the player reached the position
            target_pos.0 = None;
        }
    }
}
