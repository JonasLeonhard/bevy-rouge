use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

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

/// A single tile on the tilemap has a TilePosition that increments by 1 for each tile
/// TODO: requires MapPositionOccupied position?
#[derive(Component)]
pub struct TilePosition(pub IVec2);

/// Positions with this marker Component are Taken, meaning you cannot walk here. The MapPosition
/// might be a Wall, or a Player, a Forcefield or an Enemy etc.
#[derive(Component)]
pub struct TilePositionOccupied;
