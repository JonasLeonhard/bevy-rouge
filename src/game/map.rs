use crate::components::Player;
use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;
use pathfinding::prelude::astar;
use rand::prelude::*;

/// hovered highlighted tile
#[derive(Component)]
pub struct HighlightBorder;

/// the actual mouse position in viewport_to_world_2d coordinates
/// If offscreen - this is None
/// The Position always snaps to the actual tile center position
#[derive(Resource)]
pub struct HoveredTilePos(pub Option<Vec2>);

#[derive(Default, Debug, Resource)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 30, y: 30 };
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};
pub const CHUNK_DESPAWN_RANGE: f32 = 1000.0; // When a chunk is this far away from a player, it despawns automatically

// each tile is our world has a Grid Position that can be calculated from a World Position
// this is a basic building block for pathfinding and fov calculations
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn from_world_pos(world_pos: Vec2) -> Self {
        Self {
            x: (world_pos.x / TILE_SIZE.x).floor() as i32,
            y: (world_pos.y / TILE_SIZE.y).floor() as i32,
        }
    }

    pub fn to_world_pos(&self) -> Vec2 {
        Vec2::new(self.x as f32 * TILE_SIZE.x, self.y as f32 * TILE_SIZE.y)
    }

    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

// grid helper struct for pathfinding & position management
pub struct GameGrid;

impl GameGrid {
    /// Returns whether a position is walkable based on tile's at the position
    pub fn is_walkable(
        pos: &GridPos,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> bool {
        let world_pos = pos.to_world_pos();
        chunks_query
            .iter()
            .any(|(tile_storage, map_size, grid_size, map_type, transform)| {
                let pos_in_chunk: Vec2 = {
                    let pos = Vec4::from((world_pos, 0.0, 1.0));
                    let pos_in_chunk = transform.compute_matrix().inverse() * pos;
                    pos_in_chunk.xy()
                };

                let Some(tile_pos) =
                    TilePos::from_world_pos(&pos_in_chunk, map_size, grid_size, map_type)
                else {
                    return false;
                };

                let Some(tile_entity) = tile_storage.get(&tile_pos) else {
                    return false;
                };

                let Ok(texture_index) = tile_query.get(tile_entity) else {
                    return false;
                };

                texture_index.0 == 5 // TODO: Make this configurable
            })
    }

    fn get_successors(
        pos: &GridPos,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> Vec<(GridPos, i32)> {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        directions
            .iter()
            .map(|&(dx, dy)| GridPos {
                x: pos.x + dx,
                y: pos.y + dy,
            })
            .filter(|pos| Self::is_walkable(pos, chunks_query, tile_query))
            .map(|pos| (pos, 1))
            .collect()
    }

    /// Find a path between two world positions
    pub fn find_path(
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
        from: Vec2,
        to: Vec2,
    ) -> Option<Vec<Vec2>> {
        let start = GridPos::from_world_pos(from);
        let goal = GridPos::from_world_pos(to);

        let result = astar(
            &start,
            |p| Self::get_successors(p, chunks_query, tile_query),
            |p| p.manhattan_distance(&goal),
            |p| p == &goal,
        );

        result.map(|(path, _cost)| {
            path.into_iter()
                .map(|grid_pos| grid_pos.to_world_pos())
                .collect()
        })
    }

    /// TODO: dont know about this...
    /// Cast a ray from one grid position to another, returning all positions along the ray
    /// Including the start and end positions
    pub fn raycast(from: GridPos, to: GridPos) -> Vec<GridPos> {
        if from == to {
            return vec![from];
        }

        let dx = (to.x - from.x) as f32;
        let dy = (to.y - from.y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();

        // Use double the distance for steps to ensure we don't miss any grid cells
        let steps = (distance * 2.0).ceil() as i32;
        let mut positions = Vec::with_capacity(steps as usize);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = (from.x as f32 + dx * t).round() as i32;
            let y = (from.y as f32 + dy * t).round() as i32;

            let pos = GridPos { x, y };

            // Only add if it's a new position (avoid duplicates from rounding)
            if positions.last().map_or(true, |&last| last != pos) {
                positions.push(pos);
            }
        }

        positions
    }

    /// Cast a ray and return the first non-walkable position, if any
    /// Returns None if the ray reaches the target without hitting anything
    pub fn raycast_hit(
        from: GridPos,
        to: GridPos,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> Option<GridPos> {
        let positions = Self::raycast(from, to);

        for pos in positions {
            if !Self::is_walkable(&pos, chunks_query, tile_query) {
                return Some(pos);
            }
        }

        None
    }

    /// Check if there is direct line of sight between two positions
    pub fn has_line_of_sight(
        from: GridPos,
        to: GridPos,
        chunks_query: &Query<(
            &TileStorage,
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &Transform,
        )>,
        tile_query: &Query<&TileTextureIndex>,
    ) -> bool {
        if from == to {
            return true;
        }

        let hit = Self::raycast_hit(from, to, chunks_query, tile_query);

        // We have line of sight if we either:
        // 1. Hit nothing (ray reached target)
        // 2. Hit the target position exactly
        match hit {
            None => true,
            Some(hit_pos) => hit_pos == to,
        }
    }
}

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
                    visible: TileVisible(false), // INFO: TileVisibility will be set in fog_of_war
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

    let Some(path_to_target) =
        GameGrid::find_path(&chunks_query, &tile_query, player_pos, target_pos)
    else {
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
