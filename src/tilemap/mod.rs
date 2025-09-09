#![allow(unused)]
use bevy::{
    app::Plugin,
    asset::{Assets, Handle},
    ecs::{
        component::{Component, HookContext},
        entity::Entity,
        hierarchy::ChildOf,
        world::DeferredWorld,
    },
    image::Image,
    log::{debug, error, warn},
    math::{IVec2, UVec2, Vec2, primitives::Rectangle},
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        //
    }
}

#[derive(Debug, Component)]
#[require(TilemapChunkMap)]
#[component(immutable)]
pub struct Tilemap {
    pub tile_dims: UVec2,
    pub texture: Handle<Image>,
    pub tiles_per_chunk: UVec2,
    pub tile_size: Vec2,
}

impl Default for Tilemap {
    fn default() -> Self {
        Self {
            tile_dims: Default::default(),
            texture: Default::default(),
            tiles_per_chunk: UVec2::new(16, 16),
            tile_size: Vec2::new(1.0, 1.0),
        }
    }
}

#[derive(Debug)]
struct TilemapChunk {
    entity: Entity,
    tiles: Box<[Option<Entity>]>,
}

#[derive(Default, Component, Deref, DerefMut)]
struct TilemapChunkMap(HashMap<IVec2, TilemapChunk>);

#[derive(Component)]
#[require(TilemapLayer, TilemapType)]
#[component(
    immutable,
    on_insert = on_insert_tilemap_pos,
    on_remove = on_remove_tilemap_pos
)]
pub struct TilemapPos(pub i32, pub i32);

fn spawn_chunk(world: &mut DeferredWorld, tilemap_entity: Entity, chunk_pos: IVec2) -> Entity {
    debug!("Spawning chunk ({chunk_pos:?})");

    let tilemap = world
        .get::<Tilemap>(tilemap_entity)
        .expect("Tilemap exists");
    let texture = tilemap.texture.clone();

    let chunk_size = tilemap.tile_size * tilemap.tiles_per_chunk.as_vec2();

    let chunk_world_pos = chunk_pos.as_vec2() * chunk_size;
    let chunk_world_pos = chunk_world_pos.extend(0.0);

    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let mesh = meshes.add(Rectangle::new(chunk_size.x, chunk_size.y));

    let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
    let material = materials.add(ColorMaterial {
        texture: Some(texture),
        ..Default::default()
    });

    world
        .commands()
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(chunk_world_pos),
        ))
        .id()
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

        if chunk.tiles.iter().all(Option::is_none) {
            // We need to copy entity before using world.commands(), due to borrow checker.
            let chunk_entity = chunk.entity;

            world.commands().entity(chunk_entity).despawn();
        }
    } else {
        warn!("Unable to find chunk for tile pos {x}, {y}");
    }
}

#[derive(Component, Default, Debug)]
#[component(immutable)]
pub struct TilemapLayer(pub u16);

#[derive(Component, Default, Debug)]
#[component(immutable)]
pub struct TilemapType(pub u16);
