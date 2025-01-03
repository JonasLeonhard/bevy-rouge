use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 32.0, y: 32.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };
// Render chunk sizes are set to 4 render chunks per user specified chunk.
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

#[derive(Default, Debug, Resource)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

impl ChunkManager {
    /// Takes a general transform position like a x,y of Transform.translation.xy() and returns a
    /// its actual chunk position x,y
    fn position_to_chunk(position: &Vec2) -> IVec2 {
        let camera_pos = position.as_ivec2();
        let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
        let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
        camera_pos / (chunk_size * tile_size)
    }

    /// spawn a chunk in the world
    fn spawn(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) {
        // first we spawn an empty Tilemap component which we append Tiles to
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

        // Spawn multiple chunk Tile's as a children of a tilemap_entity
        // TODO: Research world generation algorithms. This is where world generation would take place, i think?
        // can we use wave_function collapse here? Or something similar?
        // we could also look at: fill_tilemap_rect from the examples here
        for x in 0..CHUNK_SIZE.x {
            for y in 0..CHUNK_SIZE.y {
                let tile_pos = TilePos { x, y };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        ..Default::default()
                    })
                    .id();
                commands.entity(tilemap_entity).add_child(tile_entity);
                tile_storage.set(&tile_pos, tile_entity);
            }
        }

        // Now we have a Tilemap Component with Tile's as children, we can build Building the actual Tilemap
        let transform = Transform::from_translation(Vec3::new(
            chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
            chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
            -1.0, // z-index layer
        ));
        let texture_handle: Handle<Image> = asset_server.load("images/brick_dark0.png");
        commands.entity(tilemap_entity).insert((
            TilemapBundle {
                grid_size: TILE_SIZE.into(),
                size: CHUNK_SIZE.into(),
                storage: tile_storage,
                texture: TilemapTexture::Single(texture_handle),
                tile_size: TILE_SIZE,
                transform,
                render_settings: TilemapRenderSettings {
                    render_chunk_size: RENDER_CHUNK_SIZE,
                    ..Default::default()
                },
                ..Default::default()
            },
            ZIndex(-1),
        ));
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
        .insert_resource(ChunkManager::default())
        .add_systems(Startup, spawn_level)
        .add_systems(Update, spawn_chunks_around_camera);
}

#[derive(Component)]
struct Player;

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // TODO:
    let player_asset = asset_server.load("images/player.png");
    commands.spawn((
        Player,
        Transform {
            translation: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            ..default()
        },
        Sprite {
            image: player_asset,
            ..default()
        },
    ));
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for camera in camera_query.iter() {
        let camera_chunk = ChunkManager::position_to_chunk(&camera.translation.xy());

        // -2 and +2 will create a spawn range of tile chunks to load around the camera position
        // x == camera_chunk
        // _ == chunk that will be loaded
        // . == unloaded chunk
        // . . . . . . .
        // . _ _ _ _ _ .
        // . _ _ _ _ _ .
        // . _ _ x _ _ .
        // . _ _ _ _ _ .
        // . _ _ _ _ _ .
        // . . . . . . .
        for y in (camera_chunk.y - 2)..(camera_chunk.y + 2) {
            for x in (camera_chunk.x - 2)..(camera_chunk.x + 2) {
                if !chunk_manager.spawned_chunks.contains(&IVec2::new(x, y)) {
                    chunk_manager.spawned_chunks.insert(IVec2::new(x, y));
                    ChunkManager::spawn(&mut commands, &asset_server, IVec2::new(x, y));
                }
            }
        }
    }
}
