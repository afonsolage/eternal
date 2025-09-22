#![allow(unused)]
use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    camera::visibility::{Visibility, VisibilityClass, add_visibility_class},
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::ChildOf,
        lifecycle::HookContext,
        name::Name,
        query::With,
        schedule::{IntoScheduleConfigs, common_conditions::resource_exists},
        system::{Commands, Query, Res, ResMut},
        world::DeferredWorld,
    },
    image::Image,
    log::{debug, error, warn},
    math::{IVec2, U8Vec2, U16Vec2, UVec2, Vec2, primitives::Rectangle},
    mesh::{Mesh, Mesh2d},
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::Reflect,
    sprite_render::{Material2dPlugin, MeshMaterial2d},
    transform::components::Transform,
};

mod material;

use material::{TilePod, TilemapChunkMaterial};

use crate::{
    config::tile::TileConfigList,
    world::{
        grid::{self, Grid},
        tile::{self, TileId, TileInfoMap},
    },
};

const TILES_PER_CHUNK: U16Vec2 = U16Vec2::new(32, 32);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(Material2dPlugin::<TilemapChunkMaterial>::default())
            .add_systems(
                Update,
                update_tilemap_chunk_material.run_if(resource_exists::<TileInfoMap>),
            );
    }
}

#[derive(Debug, Component, Reflect)]
#[require(TilemapChunkMap, Transform, Visibility, VisibilityClass)]
#[component(immutable, on_add = spawn_chunks)]
pub struct Tilemap {
    /// The atlas texture which contains all tile textures.
    pub atlas_texture: Handle<Image>,
    /// How many tile textures there are in the atlas.
    pub atlas_dims: UVec2,
    /// The size of each rendered individual tile.
    pub tile_size: Vec2,
}

impl Default for Tilemap {
    fn default() -> Self {
        Self {
            atlas_texture: Default::default(),
            atlas_dims: UVec2::new(4, 4),
            tile_size: Vec2::new(1.0, 1.0),
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
#[component(immutable)]
struct TilemapChunkDirty;

#[derive(Component, Default, Clone, Copy, Reflect)]
#[component(immutable)]
struct TilemapChunkPos(U16Vec2);

#[derive(Default, Component, Reflect, Deref, DerefMut)]
#[component(immutable)]
pub struct TilemapChunkMap(HashMap<U16Vec2, Entity>);

#[derive(Clone)]
struct TilemapParams {
    parent: Entity,
    atlas_texture: Handle<Image>,
    atlas_dims: UVec2,
    tile_size: Vec2,
}

fn spawn_single_chunk(
    world: &mut DeferredWorld,
    chunk_pos: U16Vec2,
    params: TilemapParams,
) -> Entity {
    let TilemapParams {
        parent,
        atlas_texture,
        atlas_dims,
        tile_size,
    } = params;

    let chunk_size = tile_size * TILES_PER_CHUNK.as_vec2();
    let chunk_world_pos = chunk_pos.as_vec2() * chunk_size;
    let chunk_world_pos = chunk_world_pos.extend(0.0);

    // A simple rectagle mesh should be enough.
    // TODO: Check if this should be cached in the future.
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let mesh = meshes.add(Rectangle::new(chunk_size.x, chunk_size.y));

    // This is the image which will hold all tile data used by shader
    let mut images = world.resource_mut::<Assets<Image>>();
    let tiles_data = images.add(material::init_tile_data());

    let mut materials = world.resource_mut::<Assets<TilemapChunkMaterial>>();
    let material = materials.add(TilemapChunkMaterial {
        atlas_texture,
        atlas_dims,
        tiles_per_chunk: TILES_PER_CHUNK.as_uvec2(),
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

    world.commands().entity(parent).add_child(chunk_entity);

    chunk_entity
}

fn spawn_chunks(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    // Since tilemap will group chunks together, we need to make it act like if it had Mesh2d too.
    if let Some(mut visibility_class) = world.get_mut::<VisibilityClass>(entity) {
        visibility_class.push(std::any::TypeId::of::<Mesh2d>());
    }

    let Some(tile_map) = world.entity(entity).get::<Tilemap>() else {
        error!("Failed to get Tilemap component.");
        return;
    };

    let params = TilemapParams {
        parent: entity,
        atlas_texture: tile_map.atlas_texture.clone(),
        atlas_dims: tile_map.atlas_dims,
        tile_size: tile_map.tile_size,
    };

    let chunks_count = U16Vec2::new(grid::WIDTH as u16, grid::HEIGHT as u16) / TILES_PER_CHUNK;
    let chunk_pos_entity_map = (0..chunks_count.x)
        .flat_map(move |x| (0..chunks_count.y).map(move |y| (x, y)))
        .map(|(x, y)| {
            let chunk_pos = U16Vec2::new(x, y);
            let chunk_entity = spawn_single_chunk(&mut world, chunk_pos, params.clone());
            (chunk_pos, chunk_entity)
        })
        .collect();

    world
        .commands()
        .entity(entity)
        .insert(TilemapChunkMap(chunk_pos_entity_map));
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
    q_parents: Query<&Grid<TileId>>,
    tile_info_map: Res<TileInfoMap>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (chunk_entity, TilemapChunkPos(chunk_pos), material, &ChildOf(parent)) in q_chunks {
        let Ok(tile_ids) = q_parents.get(parent) else {
            error!("Failed to update tilemap material. Tilemap not found.");
            continue;
        };

        // Using `get_mut` to trigger change detection and update this material on render world
        let Some(material) = materials.get_mut(material.id()) else {
            warn!("Failed to update tilemap material. Material not found for chunk {chunk_pos}.");
            continue;
        };

        let Some(tile_data_image) = images.get_mut(material.tiles_data.id()) else {
            warn!("Failed to update tilemap material. Tile data not found for chunk {chunk_pos}.");
            continue;
        };

        debug!("Updating material of chunk {chunk_pos}.");

        let tile_data_pods: &mut [TilePod] = bytemuck::cast_slice_mut(
            tile_data_image
                .data
                .as_mut()
                .expect("Material must have been initialized"),
        );

        let base_grid_pos = chunk_pos * TILES_PER_CHUNK;
        for x in 0..TILES_PER_CHUNK.x {
            for y in 0..TILES_PER_CHUNK.y {
                let grid_pos = base_grid_pos + U16Vec2::new(x, y);

                // Row-Major
                let grid_index = grid_pos.y as usize * grid::WIDTH + grid_pos.x as usize;
                let tile_id = tile_ids[grid_index];
                let tile_info = tile_info_map.get(&tile_id).unwrap_or_else(|| {
                    error!("Tile info not found for id: {}", *tile_id);
                    &tile::NONE_INFO
                });

                let tile_data_index = y as usize * TILES_PER_CHUNK.x as usize + x as usize;

                tile_data_pods[tile_data_index] = TilePod {
                    index: tile_info.atlas_index,
                };
            }
        }

        commands.entity(chunk_entity).remove::<TilemapChunkDirty>();
    }
}
