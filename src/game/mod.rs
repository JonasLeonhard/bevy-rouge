use bevy::prelude::*;

use crate::states::TurnState;

mod animation;
mod camera;
mod map;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.insert_state(TurnState::default()).add_plugins((
        map::plugin,
        player::plugin,
        camera::plugin,
        animation::plugin,
    ));
}
