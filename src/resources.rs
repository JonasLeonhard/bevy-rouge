use bevy::prelude::*;

/// .run_if(in_state(TurnState::AwaitingInput))
#[derive(States, Hash, Debug, Clone, Eq, PartialEq, Default)]
pub enum TurnState {
    #[default]
    Player,
    Environment,
}
