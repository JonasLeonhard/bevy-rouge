use bevy::prelude::*;

/// .run_if(in_state(TurnState::Player))
#[derive(States, Hash, Debug, Clone, Eq, PartialEq, Default)]
pub enum TurnState {
    #[default]
    Player,
    Environment,
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Gameplay,
}
