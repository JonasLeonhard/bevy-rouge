use bevy::prelude::*;

use super::player::Player;

#[derive(States, Hash, Debug, Clone, Eq, PartialEq, Default)]
pub enum TurnState {
    #[default]
    Player,
    Environment,
}

#[derive(Component)]
pub struct TurnTaker {
    pub actions_per_turn: u32,
    pub actions_remaining: u32,
}

impl Default for TurnTaker {
    fn default() -> Self {
        Self {
            actions_per_turn: 1,
            actions_remaining: 1,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.insert_state(TurnState::default())
        .add_systems(PostUpdate, advance_turn_state);
}

fn advance_turn_state(
    turn_state: Res<State<TurnState>>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
    mut player_query: Query<&mut TurnTaker, With<Player>>,
    mut environment_query: Query<&mut TurnTaker, Without<Player>>,
) {
    match turn_state.get() {
        TurnState::Player => {
            let Ok(mut player) = player_query.get_single_mut() else {
                next_turn_state.set(TurnState::Environment);
                return;
            };

            if player.actions_remaining <= 0 {
                next_turn_state.set(TurnState::Environment);
                player.actions_remaining = player.actions_per_turn;
            }
        }
        TurnState::Environment => {
            let all_actions_taken = environment_query
                .iter()
                .all(|turn_taker| turn_taker.actions_remaining <= 0);

            if all_actions_taken || environment_query.is_empty() {
                // Reset all environment actions and switch to player
                for mut turn_taker in environment_query.iter_mut() {
                    turn_taker.actions_remaining = turn_taker.actions_per_turn;
                }
                next_turn_state.set(TurnState::Player);
            }
        }
    }
}
