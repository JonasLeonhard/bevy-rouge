use super::pathfinding::find_path;
use crate::components::{HighlightBorder, Obstacle, Player};
use crate::resources::{ChunkManager, HoveredTilePos};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::prelude::*;

pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};
pub const CHUNK_DESPAWN_RANGE: f32 = 256.0; // When a chunk is this far away from a player, it despawns automatically

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
        .insert_resource(ChunkManager::default())
        .insert_resource(HoveredTilePos(None))
        .add_systems(
            Update,
            (
                spawn_chunks_around_player,
                despawn_outofrange_chunks,
                update_cursor_position,
                draw_path_to_hovered_tile,
                highlight_hovered_tile,
            ),
        );
}

fn spawn_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
    const OBSTACLE_CHANCE: f32 = 0.2;
    let mut rng = rand::thread_rng();

    // Spawn the elements of the tilemap.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let is_obstacle = rng.gen::<f32>() < OBSTACLE_CHANCE;
            let texture_index = if is_obstacle { 52 } else { 5 };

            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(texture_index),
                    ..Default::default()
                })
                .insert(Name::new(if is_obstacle { "Wall" } else { "Floor" }))
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

    let texture_atlas = asset_server.load("images/atlas.png");
    let mut entity = commands.entity(tilemap_entity);

    entity
        .insert(TilemapBundle {
            grid_size: TILE_SIZE.into(),
            size: CHUNK_SIZE.into(),
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_atlas),
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

fn pos_to_chunk_pos(pos: &Vec2) -> IVec2 {
    let ipos = pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    ipos / (chunk_size * tile_size)
}

fn spawn_chunks_around_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let Ok(transform) = player_query.get_single() else {
        return;
    };

    let player_pos = transform.translation.xy();
    let player_chunk_pos = pos_to_chunk_pos(&player_pos);

    for y in (player_chunk_pos.y - 2)..(player_chunk_pos.y + 2) {
        for x in (player_chunk_pos.x - 2)..(player_chunk_pos.x + 2) {
            let chunk_pos = IVec2::new(x, y);
            if !chunk_manager.spawned_chunks.contains(&chunk_pos) {
                chunk_manager.spawned_chunks.insert(chunk_pos);
                spawn_chunk(&mut commands, &asset_server, chunk_pos);
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    chunks_query: Query<(Entity, &Transform), With<TileStorage>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.xy();

    for (entity, chunk_transform) in chunks_query.iter() {
        let chunk_pos = chunk_transform.translation.xy();
        let distance = player_pos.distance(chunk_pos);

        if distance > CHUNK_DESPAWN_RANGE {
            let chunk_grid_pos = IVec2::new(
                (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32,
                (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32,
            );

            chunk_manager.spawned_chunks.remove(&chunk_grid_pos);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_cursor_position(
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_position: ResMut<HoveredTilePos>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    chunk_query: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
) {
    for cursor_moved in cursor_moved_events.read() {
        let mut found_tile = false;

        // transform the mouse's window position to world space
        for (cam_t, cam) in camera_query.iter() {
            if let Ok(world_pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                // Check each chunk/tilemap for a tile at this position
                for (map_size, grid_size, map_type, tile_storage, map_transform) in
                    chunk_query.iter()
                {
                    let cursor_in_map_pos: Vec2 = {
                        let cursor_pos = Vec4::from((world_pos, 0.0, 1.0));
                        let cursor_in_map_pos =
                            map_transform.compute_matrix().inverse() * cursor_pos;
                        cursor_in_map_pos.xy()
                    };

                    if let Some(tile_pos) =
                        TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
                    {
                        // Check if there's actually a tile here
                        if tile_storage.get(&tile_pos).is_some() {
                            // Get the center of the tile in world coordinates
                            let tile_center = tile_pos.center_in_world(grid_size, map_type);
                            let world_tile_center = (map_transform.compute_matrix()
                                * Vec4::from((tile_center, 0.0, 1.0)))
                            .xy();

                            *cursor_position = HoveredTilePos(Some(world_tile_center));
                            found_tile = true;
                            break;
                        }
                    }
                }
            }
        }

        if !found_tile {
            *cursor_position = HoveredTilePos(None);
        }
    }
}

fn highlight_hovered_tile(
    mut commands: Commands,
    hovered_tile_pos: Res<HoveredTilePos>,
    highlighted_borders_query: Query<Entity, With<HighlightBorder>>,
) {
    // remove all tile HighlightBorder entities
    for border_entity in highlighted_borders_query.iter() {
        commands.entity(border_entity).despawn();
    }

    let Some(hovered_tile_center) = hovered_tile_pos.0 else {
        return;
    };

    let outline_thickness = 2.0;
    let outline_color = Color::srgb(255.0, 0.0, 0.0);
    // Spawn all four borders at tile center position
    // Top border
    commands.spawn((
        Sprite {
            color: outline_color,
            custom_size: Some(Vec2::new(TILE_SIZE.x, outline_thickness)),
            ..default()
        },
        Transform::from_xyz(
            hovered_tile_center.x,
            hovered_tile_center.y + (TILE_SIZE.y / 2.0),
            1.0,
        ),
        HighlightBorder,
    ));

    // Bottom border
    commands.spawn((
        Sprite {
            color: outline_color,
            custom_size: Some(Vec2::new(TILE_SIZE.x, outline_thickness)),
            ..default()
        },
        Transform::from_xyz(
            hovered_tile_center.x,
            hovered_tile_center.y - (TILE_SIZE.y / 2.0),
            1.0,
        ),
        HighlightBorder,
    ));

    // Left border
    commands.spawn((
        Sprite {
            color: outline_color,
            custom_size: Some(Vec2::new(outline_thickness, TILE_SIZE.y)),
            ..default()
        },
        Transform::from_xyz(
            hovered_tile_center.x - (TILE_SIZE.x / 2.0),
            hovered_tile_center.y,
            1.0,
        ),
        HighlightBorder,
    ));

    // Right border
    commands.spawn((
        Sprite {
            color: outline_color,
            custom_size: Some(Vec2::new(outline_thickness, TILE_SIZE.y)),
            ..default()
        },
        Transform::from_xyz(
            hovered_tile_center.x + (TILE_SIZE.x / 2.0),
            hovered_tile_center.y,
            1.0,
        ),
        HighlightBorder,
    ));
}

pub fn draw_path_to_hovered_tile(
    chunks_query: Query<(
        &TileStorage,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &Transform,
    )>,
    tile_query: Query<&TileTextureIndex>,
    player_query: Query<&Transform, With<Player>>,
    hovered_tile_pos: Res<HoveredTilePos>,
    mut gizmos: Gizmos,
) {
    let player_pos = if let Ok(transform) = player_query.get_single() {
        transform.translation.xy()
    } else {
        return;
    };

    let Some(target_pos) = hovered_tile_pos.0 else {
        return;
    };

    let Some(path_to_target) = find_path(&chunks_query, &tile_query, player_pos, target_pos) else {
        return;
    };

    // Draw path
    let points: Vec<Vec3> = path_to_target
        .iter()
        .map(|pos| Vec3::new(pos.x, pos.y, 1.0))
        .collect();

    if points.len() >= 2 {
        for points in points.windows(2) {
            gizmos.line_2d(
                points[0].xy(),
                points[1].xy(),
                Color::srgba(22.0, 101.0, 52.0, 1.0),
            );
        }
    }
}

// field of view using MRPAS: https://www.roguebasin.com/index.php?title=Restrictive_Precise_Angle_Shadowcasting
