//! The game's main screen states and transitions between them.

mod gameplay;

use crate::states::Screen;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins(gameplay::plugin);
}
