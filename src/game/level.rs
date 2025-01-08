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

/// the actual mouse position in viewport_to_world_2d coordinates
#[derive(Resource)]
pub struct CursorPosition(Vec2);

impl Default for CursorPosition {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

/// tiles that are highlighted on hover have this component
#[derive(Component)]
struct HighlightBorder;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
        .insert_resource(ChunkManager::default())
        .insert_resource(CursorPosition::default())
        .add_systems(Startup, spawn_level)
        .add_systems(Update, update_cursor_position)
        .add_systems(Update, highlight_hovered_tile)
        .add_systems(Update, spawn_chunks_around_camera);
}

fn update_cursor_position(
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_position: ResMut<CursorPosition>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // transform the mouse's window position by any transforms on the camera.
        // This is done by projecting the cursor position into camera space (world space).
        for (cam_t, cam) in camera_query.iter() {
            if let Ok(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_position = CursorPosition(pos);
            }
        }
    }
}

fn highlight_hovered_tile(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition>,
    tilemap_query: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    highlighted_borders_query: Query<Entity, With<HighlightBorder>>,
) {
    // remove all tile HighlightBorder entities
    for border_entity in highlighted_borders_query.iter() {
        commands.entity(border_entity).despawn();
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_query.iter() {
        let cursor_pos: Vec2 = cursor_pos.0;
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };

        let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        else {
            continue;
        };

        let Some(_tile_entity) = tile_storage.get(&tile_pos) else {
            continue;
        };

        let outline_thickness = 2.0;
        let outline_color = Color::srgb(255.0, 0.0, 0.0);
        let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.0);
        let transform = *map_transform * Transform::from_translation(tile_center);

        // Spawn all four borders
        // Top border
        commands.spawn((
            Sprite {
                color: outline_color,
                custom_size: Some(Vec2::new(grid_size.x, outline_thickness)),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x,
                transform.translation.y + (grid_size.y / 2.0),
                transform.translation.z + 1.0,
            ),
            HighlightBorder,
        ));

        // Bottom border
        commands.spawn((
            Sprite {
                color: outline_color,
                custom_size: Some(Vec2::new(grid_size.x, outline_thickness)),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x,
                transform.translation.y - (grid_size.y / 2.0),
                transform.translation.z + 1.0,
            ),
            HighlightBorder,
        ));

        // Left border
        commands.spawn((
            Sprite {
                color: outline_color,
                custom_size: Some(Vec2::new(outline_thickness, grid_size.y)),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x - (grid_size.x / 2.0),
                transform.translation.y,
                transform.translation.z + 1.0,
            ),
            HighlightBorder,
        ));

        // Right border
        commands.spawn((
            Sprite {
                color: outline_color,
                custom_size: Some(Vec2::new(outline_thickness, grid_size.y)),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x + (grid_size.x / 2.0),
                transform.translation.y,
                transform.translation.z + 1.0,
            ),
            HighlightBorder,
        ));
    }
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
