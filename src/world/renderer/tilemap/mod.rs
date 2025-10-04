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
        query::{Changed, With},
        schedule::{
            IntoScheduleConfigs, SystemCondition,
            common_conditions::{resource_changed, resource_exists},
        },
        system::{Commands, Query, Res, ResMut},
        world::{DeferredWorld, Mut},
    },
    image::Image,
    log::{debug, error, warn},
    math::{IVec2, U8Vec2, U16Vec2, UVec2, Vec2, primitives::Rectangle},
    mesh::{Mesh, Mesh2d, PrimitiveTopology},
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
        grid::{self, Grid, GridId},
        tile::{self, TileId, TileRegistry},
    },
};

const TILES_PER_CHUNK: U16Vec2 = U16Vec2::new(32, 32);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(Material2dPlugin::<TilemapChunkMaterial>::default())
            .add_systems(
                Update,
                update_tilemap_chunk_material.run_if(
                    resource_exists::<TileRegistry>.and(
                        resource_changed::<TileRegistry>
                            .or(|q: Query<(), Changed<Grid<TileId>>>| !q.is_empty()),
                    ),
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

#[derive(Component, Default, Clone, Copy, Reflect)]
#[component(immutable)]
struct TilemapChunkPos(U16Vec2);

#[derive(Default, Component, Reflect, Deref, DerefMut)]
#[component(immutable)]
pub struct TilemapChunkMap(HashMap<U16Vec2, Entity>);

#[derive(Default, Clone, Component, Reflect)]
#[component(immutable)]
struct TilemapCache {
    material: Handle<TilemapChunkMaterial>,
    mesh: Handle<Mesh>,
}

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
    layer: i32,
    tile_size: Vec2,
    parent: Entity,
    TilemapCache { material, mesh }: TilemapCache,
) -> Entity {
    let chunk_size = tile_size * TILES_PER_CHUNK.as_vec2();
    let chunk_world_pos = chunk_pos.as_vec2() * chunk_size;
    let chunk_world_pos = chunk_world_pos.extend(layer as f32);

    let chunk_entity = world
        .commands()
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(chunk_world_pos).with_scale(tile_size.extend(1.0)),
            TilemapChunkPos(chunk_pos),
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

    let atlas_texture = tile_map.atlas_texture.clone();
    let atlas_dims = tile_map.atlas_dims;
    let tile_size = tile_map.tile_size;

    let mut images = world.resource_mut::<Assets<Image>>();
    let tiles_data = images.add(material::init_tile_data());

    let mut materials = world.resource_mut::<Assets<TilemapChunkMaterial>>();
    let material = materials.add(TilemapChunkMaterial {
        atlas_texture,
        atlas_dims,
        tiles_per_chunk: TILES_PER_CHUNK.as_uvec2(),
        tile_size,
        tiles_data,
    });

    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let mesh = meshes.add(create_tilemap_chunk_mesh());

    let cache = TilemapCache { material, mesh };

    let chunks_count = grid::DIMS.as_u16vec2() / TILES_PER_CHUNK;
    let chunk_pos_entity_map = (0..chunks_count.x)
        .flat_map(move |x| (0..chunks_count.y).map(move |y| (x, y)))
        .map(|(x, y)| {
            let chunk_pos = U16Vec2::new(x, y);
            let chunk_entity =
                spawn_single_chunk(&mut world, chunk_pos, 0, tile_size, entity, cache.clone());
            (chunk_pos, chunk_entity)
        })
        .collect();

    world
        .commands()
        .entity(entity)
        .insert((TilemapChunkMap(chunk_pos_entity_map), cache));
}

fn create_tilemap_chunk_mesh() -> Mesh {
    let x = TILES_PER_CHUNK.x as f32;
    let y = TILES_PER_CHUNK.y as f32;

    let positions = vec![[0.0, 0.0, 0.0], [x, 0.0, 0.0], [x, y, 0.0], [0.0, y, 0.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U16(indices))
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct TilemapIndex(pub u16);

fn update_tilemap_chunk_material(
    q_tilemaps: Query<(&GridId, &TilemapCache)>,
    tile_info_map: Res<TileRegistry>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (grid, TilemapCache { material, .. }) in q_tilemaps {
        // Using `get_mut` to trigger change detection and update this material on render world
        let Some(material) = materials.get_mut(material.id()) else {
            warn!("Failed to update tilemap material. Material not found.");
            continue;
        };

        let Some(tile_data_image) = images.get_mut(material.tiles_data.id()) else {
            warn!("Failed to update tilemap material. Tile data not found.");
            continue;
        };

        debug!("Updating material of tilemap.");

        let tile_data_pods: &mut [TilePod] = bytemuck::cast_slice_mut(
            tile_data_image
                .data
                .as_mut()
                .expect("Material must have been initialized"),
        );

        tile_data_pods
            .iter_mut()
            .enumerate()
            .for_each(|(idx, pod)| {
                let id = grid[0][idx];
                let info = tile_info_map.get(&id).unwrap_or(&tile::NONE_INFO);

                pod.index = info.atlas_index;
                pod.weight = match info.blend_tech {
                    tile::BlendTech::None => u16::MAX,
                    tile::BlendTech::Weight(w) => w,
                }; // TODO: Set height;
            });
    }
}
