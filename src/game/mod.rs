use bevy::prelude::*;

pub mod animation;
mod camera;
pub mod devil;
pub mod fog_of_war;
pub mod fov;
pub mod map;
pub mod player;
pub mod turns;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        map::plugin,
        fov::plugin,
        fog_of_war::plugin,
        player::plugin,
        devil::plugin,
        camera::plugin,
        animation::plugin,
        turns::plugin,
    ));
}
