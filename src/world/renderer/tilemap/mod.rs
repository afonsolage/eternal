#![allow(unused)]
use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle, RenderAssetUsages},
    camera::visibility::{Visibility, VisibilityClass, add_visibility_class},
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::ChildOf,
        lifecycle::HookContext,
        name::Name,
        query::With,
        schedule::{
            IntoScheduleConfigs,
            common_conditions::{resource_exists, resource_exists_and_changed},
        },
        system::{Commands, Query, Res, ResMut},
        world::DeferredWorld,
    },
    image::Image,
    log::{debug, error, warn},
    math::{IVec2, U8Vec2, U16Vec2, UVec2, Vec2, primitives::Rectangle},
    mesh::{Indices, Mesh, Mesh2d, MeshVertexAttribute, PrimitiveTopology, VertexFormat},
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::Reflect,
    render::storage::ShaderStorageBuffer,
    sprite_render::{Material2dPlugin, MeshMaterial2d},
    transform::components::Transform,
};

mod material;

use material::{TilePod, TilemapChunkMaterial};

use crate::{
    config::tile::TileConfigList,
    world::{
        grid::{self, Grid},
        tile::{self, TileId, TileInfos},
    },
};

const TILES_PER_CHUNK: U16Vec2 = U16Vec2::new(32, 32);
// We need one vertex more at the end to form a quad
const VERTICES_PER_CHUNK: U16Vec2 = U16Vec2::new(TILES_PER_CHUNK.x + 1, TILES_PER_CHUNK.y + 1);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(Material2dPlugin::<TilemapChunkMaterial>::default())
            .add_systems(
                Update,
                (
                    update_tilemap_chunk_material.run_if(resource_exists_and_changed::<TileInfos>),
                    update_tilemap_chunk_mesh,
                ),
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

#[derive(Default, Component, Reflect)]
#[component(immutable)]
pub struct TilemapChunkMaterialHandler(Handle<TilemapChunkMaterial>);

#[derive(Clone)]
struct TilemapParams {
    parent: Entity,
    atlas_texture: Handle<Image>,
    atlas_dims: UVec2,
    tile_size: Vec2,
}

fn create_chunk_mesh() -> Mesh {
    // Each tile will have 4 shared vertex, so we just need to extend the tile count by 1 in each
    // dimension
    let vertex_count = VERTICES_PER_CHUNK.element_product() as usize;
    let mut pos = Vec::with_capacity(vertex_count);
    let mut uv = Vec::with_capacity(vertex_count);

    for y in 0..VERTICES_PER_CHUNK.y {
        for x in 0..VERTICES_PER_CHUNK.x {
            // We are offseting each vertex by half tile, so the middle of the tile is always an
            // integer unit, like 10, 10, which would make easier to compute the tile based on a
            // world coord.
            pos.push([x as f32 - 0.5, y as f32 - 0.5, 0.0]);

            uv.push([
                f32::clamp(x as f32 / TILES_PER_CHUNK.x as f32, 0.0, 1.0),
                f32::clamp(y as f32 / TILES_PER_CHUNK.y as f32, 0.0, 1.0),
            ]);
        }
    }

    // Each tile has 2 triangles, with 3 vertex indices each
    let indice_count = TILES_PER_CHUNK.element_product() as usize * 6;
    let mut indices = Vec::with_capacity(indice_count);
    let row_size = VERTICES_PER_CHUNK.x;

    for y in 0..TILES_PER_CHUNK.y {
        for x in 0..TILES_PER_CHUNK.x {
            let i = y * row_size + x;

            //
            //i+r       i+r+1
            //   +-----+
            //   |     |
            //   |     |
            //   +-----+
            //  i       i+1
            //
            //  Y
            //  |
            //  +---x

            indices.push(i);
            indices.push(i + 1);
            indices.push(i + row_size + 1);

            indices.push(i);
            indices.push(i + row_size + 1);
            indices.push(i + row_size);
        }
    }

    let tile_ids = vec![[0u32; 4]; vertex_count];

    Mesh::new(PrimitiveTopology::TriangleList, Default::default())
        .with_inserted_indices(Indices::U16(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(material::ATTRIBUTE_TILE_ID, tile_ids)
}

fn spawn_single_chunk(
    world: &mut DeferredWorld,
    chunk_pos: U16Vec2,
    tile_size: Vec2,
    parent: Entity,
    material: Handle<TilemapChunkMaterial>,
) -> Entity {
    let chunk_size = tile_size * TILES_PER_CHUNK.as_vec2();
    let chunk_world_pos = chunk_pos.as_vec2() * chunk_size;
    let chunk_world_pos = chunk_world_pos.extend(0.0);

    // A simple rectagle mesh should be enough.
    // TODO: Check if this should be cached in the future.
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    //let mesh = meshes.add(Rectangle::new(chunk_size.x, chunk_size.y));
    let mesh = meshes.add(create_chunk_mesh());

    let chunk_entity = world
        .commands()
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(chunk_world_pos).with_scale(tile_size.extend(1.0)),
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

    let tile_size = tile_map.tile_size;
    let atlas_dims = tile_map.atlas_dims;
    let atlas_texture = tile_map.atlas_texture.clone();

    let mut materials = world.resource_mut::<Assets<TilemapChunkMaterial>>();
    let material = materials.add(TilemapChunkMaterial {
        atlas_texture,
        atlas_dims,
        tiles_per_chunk: TILES_PER_CHUNK.as_uvec2(),
        ..Default::default()
    });

    let chunks_count = U16Vec2::new(grid::WIDTH as u16, grid::HEIGHT as u16) / TILES_PER_CHUNK;
    let chunk_pos_entity_map = (0..chunks_count.x)
        .flat_map(move |x| (0..chunks_count.y).map(move |y| (x, y)))
        .map(|(x, y)| {
            let chunk_pos = U16Vec2::new(x, y);
            let chunk_entity =
                spawn_single_chunk(&mut world, chunk_pos, tile_size, entity, material.clone());
            (chunk_pos, chunk_entity)
        })
        .collect();

    world.commands().entity(entity).insert((
        TilemapChunkMap(chunk_pos_entity_map),
        TilemapChunkMaterialHandler(material),
    ));
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct TilemapIndex(pub u16);

fn update_tilemap_chunk_material(
    q_tilemaps: Query<&TilemapChunkMaterialHandler>,
    tile_info_list: Res<TileInfos>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut commands: Commands,
) {
    let pods = tile_info_list
        .iter()
        .map(|info| TilePod {
            atlas_index: info.atlas_index as u32,
            height: 0.0,
        })
        .collect::<Vec<_>>();

    let buffer_handle = buffers.add(ShaderStorageBuffer::from(&pods));

    for TilemapChunkMaterialHandler(material_handler) in q_tilemaps {
        let Some(material) = materials.get_mut(material_handler.id()) else {
            error!("Failed to update tilemap material. Material not found.");
            continue;
        };

        material.tiles_info = buffer_handle.clone();
    }
}

const CORNER_DIRS: [IVec2; 4] = [
    IVec2::new(0, 0),   // Top Right
    IVec2::new(-1, 0),  // Top Left
    IVec2::new(-1, -1), // Bottom Left
    IVec2::new(0, -1),  // Bottom Right
];
fn update_tilemap_chunk_mesh(
    q_chunks: Query<(Entity, &Mesh2d, &TilemapChunkPos, &ChildOf), With<TilemapChunkDirty>>,
    q_tilemaps: Query<(&Grid<TileId>, &Tilemap)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, mesh_handle, &TilemapChunkPos(chunk_pos), &ChildOf(parent)) in q_chunks {
        commands.entity(entity).remove::<TilemapChunkDirty>();

        let Ok((grid, tilemap)) = q_tilemaps.get(parent) else {
            warn!("Failed to update chunk mesh {chunk_pos}: Tilemap not found.");
            continue;
        };

        debug!("Updating mesh of chunk {chunk_pos}");

        let vertex_count = (VERTICES_PER_CHUNK).element_product() as usize;
        let grid_base_pos = (chunk_pos * TILES_PER_CHUNK).as_ivec2();

        let mut tile_ids = (0..vertex_count)
            .map(|index| {
                let x = index as i32 % VERTICES_PER_CHUNK.x as i32;
                let y = index as i32 / VERTICES_PER_CHUNK.x as i32;
                let vertex_pos = grid_base_pos + IVec2::new(x, y);

                CORNER_DIRS
                    .map(|dir| grid.try_get(vertex_pos + dir).unwrap_or(TileId::NONE).id() as u32)
            })
            .collect::<Vec<_>>();

        if let Some(mesh) = meshes.get_mut(mesh_handle.id()) {
            mesh.insert_attribute(material::ATTRIBUTE_TILE_ID, tile_ids);
        }
    }
}
