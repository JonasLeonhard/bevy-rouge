use crate::components::HighlightBorder;
use crate::resources::{ChunkManager, CursorPosition};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
        .insert_resource(ChunkManager::default())
        .insert_resource(CursorPosition::default())
        .add_systems(Update, spawn_chunks_around_camera)
        .add_systems(Update, despawn_outofrange_chunks)
        .add_systems(Update, update_cursor_position)
        .add_systems(Update, highlight_hovered_tile);
}

fn spawn_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

    // Spawn the elements of the tilemap.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .insert(Name::new("Tile"))
                .id();
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
        0.0,
    ));
    let texture_handle: Handle<Image> = asset_server.load("images/brick_dark0.png");
    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
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
        })
        .insert(Name::new("Chunk"));
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.xy());
        for y in (camera_chunk_pos.y - 2)..(camera_chunk_pos.y + 2) {
            for x in (camera_chunk_pos.x - 2)..(camera_chunk_pos.x + 2) {
                if !chunk_manager.spawned_chunks.contains(&IVec2::new(x, y)) {
                    chunk_manager.spawned_chunks.insert(IVec2::new(x, y));
                    spawn_chunk(&mut commands, &asset_server, IVec2::new(x, y));
                }
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.xy();
            let distance = camera_transform.translation.xy().distance(chunk_pos);
            if distance > 320.0 {
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32;
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn_recursive();
            }
        }
    }
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
