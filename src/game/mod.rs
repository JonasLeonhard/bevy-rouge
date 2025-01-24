use bevy::prelude::*;

use crate::states::TurnState;

pub mod animation;
mod camera;
pub mod fog_of_war;
pub mod fov;
pub mod map;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.insert_state(TurnState::default()).add_plugins((
        map::plugin,
        fov::plugin,
        fog_of_war::plugin,
        player::plugin,
        camera::plugin,
        animation::plugin,
    ));
}
