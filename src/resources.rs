use bevy::{prelude::*, utils::HashSet};

#[derive(Default, Debug, Resource)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

/// the actual mouse position in viewport_to_world_2d coordinates
#[derive(Resource)]
pub struct CursorPosition(pub Vec2);

impl Default for CursorPosition {
    fn default() -> Self {
        // initialize at offscreen position. this will get
        // updated when the cursor moves
        Self(Vec2::new(-1000.0, -1000.0))
    }
}
