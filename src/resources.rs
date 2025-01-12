use bevy::{prelude::*, utils::HashSet};

#[derive(Default, Debug, Resource)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

/// the actual mouse position in viewport_to_world_2d coordinates
/// If offscreen - this is None
/// The Position always snaps to the actual tile center position
#[derive(Resource)]
pub struct HoveredTilePos(pub Option<Vec2>);
