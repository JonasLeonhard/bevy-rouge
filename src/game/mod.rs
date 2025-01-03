use bevy::prelude::*;

mod camera;
pub mod level;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((level::plugin, camera::plugin));
}
