use bevy::prelude::*;

pub use crate::game::turns::TurnState;

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Gameplay,
}
