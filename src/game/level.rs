use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_level);
}

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // TODO:
    commands.spawn(Sprite::from_image(asset_server.load("images/ducky.png")));
}
