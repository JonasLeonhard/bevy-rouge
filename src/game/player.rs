use crate::{
    components::{AnimationConfig, Player, TurnTaker},
    states::TurnState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, (on_input).run_if(in_state(TurnState::Player)));
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
            actions_per_turn: 1,
            actions_remaining: 1,
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

fn on_input() {
    // TODO: player input
}
