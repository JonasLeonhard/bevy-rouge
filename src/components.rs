use bevy::prelude::*;

pub use crate::game::animation::AnimationConfig;
pub use crate::game::fov::FieldOfView;
pub use crate::game::player::Player;

// TODO: implement actions
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
