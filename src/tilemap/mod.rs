use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    ecs::{
        component::{Component, HookContext},
        entity::Entity,
        hierarchy::ChildOf,
        name::Name,
        query::With,
        system::{Commands, Query, Res, ResMut},
        world::DeferredWorld,
    },
    image::Image,
    log::{debug, error, warn},
    math::{IVec2, UVec2, Vec2, primitives::Rectangle},
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::Reflect,
    render::{
        mesh::{Mesh, Mesh2d},
        view::{Visibility, VisibilityClass, add_visibility_class},
    },
    sprite::{Material2dPlugin, MeshMaterial2d},
    transform::components::Transform,
};
use bevy_inspector_egui::quick::AssetInspectorPlugin;

use crate::tilemap::material::{TilePod, TilemapChunkMaterial};

mod material;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(Material2dPlugin::<TilemapChunkMaterial>::default())
            .add_plugins(AssetInspectorPlugin::<TilemapChunkMaterial>::default())
            .add_systems(Update, update_tilemap_chunk_material);
    }
}

#[derive(Debug, Component, Reflect)]
#[require(TilemapChunkMap, Transform, Visibility, VisibilityClass)]
#[component(immutable, on_add = add_visibility_class::<Mesh2d>)]
pub struct Tilemap {
    /// The atlas texture which contains all tile textures.
    pub atlas_texture: Handle<Image>,
    /// How many tile textures there are in the atlas.
    pub atlas_dims: UVec2,
    /// How many tiles should be in each chunk mesh.
    pub tiles_per_chunk: UVec2,
    /// The size of each rendered individual tile.
    pub tile_size: Vec2,
}

impl Default for Tilemap {
    fn default() -> Self {
        Self {
            atlas_texture: Default::default(),
            atlas_dims: UVec2::new(4, 4),
            tiles_per_chunk: UVec2::new(16, 16),
            tile_size: Vec2::new(1.0, 1.0),
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
#[component(immutable)]
struct TilemapChunkDirty;

#[derive(Component, Default, Clone, Copy, Reflect)]
#[component(immutable)]
struct TilemapChunkPos(IVec2);

#[derive(Debug)]
struct TilemapChunk {
    entity: Entity,
    tiles: Box<[Option<Entity>]>,
}

#[derive(Default, Component, Deref, DerefMut)]
struct TilemapChunkMap(HashMap<IVec2, TilemapChunk>);

#[derive(Component, Clone, Copy, Reflect)]
#[require(TilemapIndex)]
#[component(
    immutable,
    on_insert = on_insert_tilemap_pos,
    on_remove = on_remove_tilemap_pos
)]
pub struct TilemapPos(pub i32, pub i32);

fn spawn_chunk(world: &mut DeferredWorld, tilemap_entity: Entity, chunk_pos: IVec2) -> Entity {
    debug!("Spawning chunk ({chunk_pos})");

    let tilemap = world
        .get::<Tilemap>(tilemap_entity)
        .expect("Tilemap exists");

    // Collect needed data from tilemap, since we need to release it in order to use world
    let atlas_texture = tilemap.atlas_texture.clone();
    let atlas_dims = tilemap.atlas_dims;
    let tiles_per_chunk = tilemap.tiles_per_chunk;

    let chunk_size = tilemap.tile_size * tilemap.tiles_per_chunk.as_vec2();
    let chunk_world_pos = chunk_pos.as_vec2() * chunk_size;
    let chunk_world_pos = chunk_world_pos.extend(0.0);

    // A simple rectagle mesh should be enough.
    // TODO: Check if this should be cached in the future.
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let mesh = meshes.add(Rectangle::new(chunk_size.x, chunk_size.y));

    // This is the image which will hold all tile data used by shader
    let mut images = world.resource_mut::<Assets<Image>>();
    let tiles_data = images.add(material::create_empty_tile_indices_image(tiles_per_chunk));

    let mut materials = world.resource_mut::<Assets<TilemapChunkMaterial>>();
    let material = materials.add(TilemapChunkMaterial {
        atlas_texture,
        atlas_dims,
        tiles_per_chunk,
        tiles_data,
    });

    let chunk_entity = world
        .commands()
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(chunk_world_pos),
            TilemapChunkPos(chunk_pos),
            TilemapChunkDirty,
            Name::new(format!("Chunk {chunk_pos}")),
        ))
        .id();

    world
        .commands()
        .entity(tilemap_entity)
        .add_child(chunk_entity);

    chunk_entity
}

fn on_insert_tilemap_pos(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let Some(&TilemapPos(x, y)) = world.get::<TilemapPos>(entity) else {
        warn!("Failed to get TilemapPos component");
        return;
    };

    let Some(&ChildOf(parent)) = world.get::<ChildOf>(entity) else {
        panic!("TilemapPos should be added to an entity children of Tilmeap. No parent found.");
    };

    let Some(tilemap) = world.get::<Tilemap>(parent) else {
        panic!(
            "TilemapPos should be added to an entity children of Tilmeap. No Tilemap found on parent."
        );
    };

    let tiles_per_chunk = tilemap.tiles_per_chunk;

    // Check which chunk position the new tile position belongs to.
    let chunk_pos = IVec2::new(
        (x as f32 / tiles_per_chunk.x as f32).floor() as i32,
        (y as f32 / tiles_per_chunk.y as f32).floor() as i32,
    );

    // Converts the global tile position to the local chunk (inside chunk) position.
    // This would make (-1, 33) into (15, 1), assuming (16, 16) as tiles per chunk.
    let tile_local_pos = UVec2::new(
        x.rem_euclid(tiles_per_chunk.x as i32) as u32,
        y.rem_euclid(tiles_per_chunk.y as i32) as u32,
    );

    // Using column-major
    let tile_index = (tile_local_pos.x * tiles_per_chunk.y + tile_local_pos.y) as usize;

    let Some(mut chunk_map) = world.get_mut::<TilemapChunkMap>(parent) else {
        error!("Tilemap must have a ChunkMap component");
        return;
    };

    if let Some(chunk) = chunk_map.get_mut(&chunk_pos) {
        chunk.tiles[tile_index] = Some(entity);

        let chunk_entity = chunk.entity;
        world
            .commands()
            .entity(chunk_entity)
            .insert(TilemapChunkDirty);
    } else {
        let chunk_entity = spawn_chunk(&mut world, parent, chunk_pos);

        // Needs to reborrow, since the previous borrow had to be dropped due to borrow checker.
        let Some(mut chunk_map) = world.get_mut::<TilemapChunkMap>(parent) else {
            error!("Tilemap must have a ChunkMap component");
            return;
        };

        let mut tiles = vec![None; tiles_per_chunk.element_product() as usize].into_boxed_slice();
        tiles[tile_index] = Some(entity);

        chunk_map.insert(
            chunk_pos,
            TilemapChunk {
                entity: chunk_entity,
                tiles,
            },
        );
    }
}

fn on_remove_tilemap_pos(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let Some(&TilemapPos(x, y)) = world.get::<TilemapPos>(entity) else {
        warn!("Failed to get TilemapPos component");
        return;
    };

    let Some(&ChildOf(parent)) = world.get::<ChildOf>(entity) else {
        panic!("TilemapPos should be added to an entity children of Tilmeap. No parent found.");
    };

    let Some(tilemap) = world.get::<Tilemap>(parent) else {
        panic!(
            "TilemapPos should be added to an entity children of Tilmeap. No Tilemap found on parent."
        );
    };

    let tiles_per_chunk = tilemap.tiles_per_chunk;

    // Check which chunk position the new tile position belongs to.
    let chunk_pos = IVec2::new(
        (x as f32 / tiles_per_chunk.x as f32).floor() as i32,
        (y as f32 / tiles_per_chunk.y as f32).floor() as i32,
    );

    // Converts the global tile position to the local chunk (inside chunk) position.
    // This would make (-1, 33) into (15, 1), assuming (16, 16) as tiles per chunk.
    let tile_local_pos = UVec2::new(
        x.rem_euclid(tiles_per_chunk.x as i32) as u32,
        y.rem_euclid(tiles_per_chunk.y as i32) as u32,
    );

    // Using column-major
    let tile_index = (tile_local_pos.x * tiles_per_chunk.y + tile_local_pos.y) as usize;

    let Some(mut chunk_map) = world.get_mut::<TilemapChunkMap>(parent) else {
        error!("Tilemap must have a ChunkMap component");
        return;
    };

    if let Some(chunk) = chunk_map.get_mut(&chunk_pos) {
        chunk.tiles[tile_index] = None;

        // We need to copy entity before using world.commands(), due to borrow checker.
        let chunk_entity = chunk.entity;
        if chunk.tiles.iter().all(Option::is_none) {
            world.commands().entity(chunk_entity).despawn();
        } else {
            world
                .commands()
                .entity(chunk_entity)
                .insert(TilemapChunkDirty);
        }
    } else {
        error!("Unable to find chunk for tile pos {x}, {y}");
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct TilemapIndex(pub u16);

fn update_tilemap_chunk_material(
    q_chunks: Query<
        (
            Entity,
            &TilemapChunkPos,
            &mut MeshMaterial2d<TilemapChunkMaterial>,
            &ChildOf,
        ),
        With<TilemapChunkDirty>,
    >,
    q_tiles: Query<(&TilemapPos, &TilemapIndex)>,
    q_tilemaps: Query<(&Tilemap, &TilemapChunkMap)>,
    materials: Res<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (chunk_entity, TilemapChunkPos(chunk_pos), material, &ChildOf(tilemap_entity)) in q_chunks {
        let Ok((_tilemap, chunk_map)) = q_tilemaps.get(tilemap_entity) else {
            error!("Failed to update tilemap material. Tilemap not found.");
            continue;
        };

        let Some(chunk) = chunk_map.get(chunk_pos) else {
            warn!("Failed to update tilemap material. Chunk not found for chunk {chunk_pos}.");
            continue;
        };

        let Some(material) = materials.get(material.id()) else {
            warn!("Failed to update tilemap material. Material not found for chunk {chunk_pos}.");
            continue;
        };

        let Some(tile_data_image) = images.get_mut(material.tiles_data.id()) else {
            warn!("Failed to update tilemap material. Tile data not found for chunk {chunk_pos}.");
            continue;
        };

        #[cfg(debug_assertions)]
        {
            let tiles = chunk.tiles.iter().filter(|o| o.is_some()).count();
            debug!("Updating material of chunk {chunk_pos}. {tiles} will be rendered");
        }

        let tile_data_pods: &mut [TilePod] = bytemuck::cast_slice_mut(
            tile_data_image
                .data
                .as_mut()
                .expect("Material must have been initialized"),
        );

        chunk
            .tiles
            .iter()
            .enumerate()
            .for_each(|(idx, &maybe_entity)| {
                let pod = if let Some(entity) = maybe_entity
                    && let Ok((_tile_pos, tile_index)) = q_tiles.get(entity)
                {
                    TilePod {
                        index: tile_index.0,
                    }
                } else {
                    TilePod::discard()
                };

                tile_data_pods[idx] = pod;
            });

        commands.entity(chunk_entity).remove::<TilemapChunkDirty>();
    }
}
