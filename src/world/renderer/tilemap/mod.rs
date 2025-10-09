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
        observer::On,
        query::{Changed, With},
        schedule::{
            IntoScheduleConfigs, SystemCondition,
            common_conditions::{resource_changed, resource_exists},
        },
        system::{Commands, Query, Res, ResMut, Single},
        world::{DeferredWorld, Mut},
    },
    image::Image,
    log::{debug, error, info, warn},
    math::{IVec2, U8Vec2, U16Vec2, UVec2, Vec2, primitives::Rectangle},
    mesh::{Mesh, Mesh2d, MeshTag, PrimitiveTopology},
    picking::events::{Pointer, Release},
    platform::collections::HashMap,
    prelude::{Deref, DerefMut},
    reflect::Reflect,
    sprite_render::{Material2dPlugin, MeshMaterial2d},
    transform::components::Transform,
    utils::default,
};

mod material;

pub use material::{TilePod, TilemapChunkMaterial};

use crate::{
    config::tile::TileConfigList,
    world::{
        grid::{self, Grid, GridId, LAYER_SIZE, LAYERS, LAYERS_COUNT, LayerIndex},
        renderer::tilemap::material::TilemapChunkMaterialConfig,
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

#[derive(Component, Default, Clone, Copy, Reflect, Hash, PartialEq, Eq)]
#[component(immutable)]
pub struct TilemapChunkPos {
    xy: U16Vec2,
    layer: LayerIndex,
}

impl std::fmt::Display for TilemapChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.layer, self.xy)
    }
}

#[derive(Default, Component, Reflect, Deref, DerefMut)]
#[component(immutable)]
pub struct TilemapChunkMap(HashMap<TilemapChunkPos, Entity>);

#[derive(Default, Clone, Component, Reflect)]
#[component(immutable)]
pub struct TilemapCache {
    pub material: Handle<TilemapChunkMaterial>,
    pub mesh: Handle<Mesh>,
}

#[derive(Clone)]
struct TilemapParams {
    parent: Entity,
    atlas_texture: Handle<Image>,
    atlas_dims: UVec2,
    tile_size: Vec2,
}

fn spawn_single_chunk(
    commands: &mut Commands,
    chunk_pos: TilemapChunkPos,
    tile_size: Vec2,
    parent: Entity,
    TilemapCache { material, mesh }: TilemapCache,
) -> Entity {
    let chunk_size = tile_size * TILES_PER_CHUNK.as_vec2();
    let chunk_world_pos = chunk_pos.xy.as_vec2() * chunk_size;
    let chunk_world_pos = chunk_world_pos.extend(chunk_pos.layer.height());

    commands
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            MeshTag(chunk_pos.layer as u32),
            Transform::from_translation(chunk_world_pos).with_scale(tile_size.extend(1.0)),
            chunk_pos,
            Name::new(format!("Chunk {chunk_pos}")),
            ChildOf(parent),
        ))
        .id()
}

fn spawn_chunks(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    // TODO: Use singletons in the future
    let mut state = world
        .try_query::<&Tilemap>()
        .expect("Tilemap is registered");
    if world.query(&mut state).count() != 1 {
        warn!("There should be only one Tilemap entity!");
    }

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
    let tiles_data = images.add(material::init_tile_data(LAYERS_COUNT));

    let mut materials = world.resource_mut::<Assets<TilemapChunkMaterial>>();
    let material = materials.add(TilemapChunkMaterial {
        atlas_texture,
        atlas_dims,
        tiles_per_chunk: TILES_PER_CHUNK.as_uvec2(),
        tile_size,
        tiles_data,
        config: Some(TilemapChunkMaterialConfig {
            disable_floor_blending: true,
            wall_hide_outline: true,
            wall_hide_shadow: true,
        }),
    });

    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let mesh = meshes.add(create_tilemap_chunk_mesh());

    let cache = TilemapCache { material, mesh };
    let chunks_count = grid::DIMS.as_u16vec2() / TILES_PER_CHUNK;

    let mut commands = world.commands();

    let mut chunk_map = TilemapChunkMap::default();
    for layer in LAYERS {
        let layer_entity = commands
            .spawn((
                Name::new(format!("Layer {layer:?}")),
                Transform::default(),
                ChildOf(entity),
                Visibility::Inherited,
                layer,
            ))
            .observe(|pick: On<Pointer<Release>>| {
                info!("Clicked!");
            })
            .id();

        for y in 0..chunks_count.y {
            for x in 0..chunks_count.x {
                let chunk_pos = TilemapChunkPos {
                    xy: U16Vec2::new(x, y),
                    layer,
                };
                let chunk_entity = spawn_single_chunk(
                    &mut commands,
                    chunk_pos,
                    tile_size,
                    layer_entity,
                    cache.clone(),
                );

                chunk_map.insert(chunk_pos, chunk_entity);
            }
        }
    }

    commands.entity(entity).insert((chunk_map, cache));
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
    tilemap: Single<(&GridId, &TilemapCache)>,
    tile_info_map: Res<TileRegistry>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (grid, TilemapCache { material, .. }) = *tilemap;

    // Using `get_mut` to trigger change detection and update this material on render world
    let Some(material) = materials.get_mut(material.id()) else {
        warn!("Failed to update tilemap material. Material not found.");
        return;
    };

    let Some(tile_data_image) = images.get_mut(material.tiles_data.id()) else {
        warn!("Failed to update tilemap material. Tile data not found.");
        return;
    };

    debug!("Updating material of tilemap.");

    for layer in LAYERS {
        let tile_data_pods = get_data_pod_layer(layer, tile_data_image);
        let grid_layer = &grid[layer];

        tile_data_pods
            .iter_mut()
            .enumerate()
            .for_each(|(idx, pod)| {
                let id = grid_layer[idx];
                let info = tile_info_map.get(&id).unwrap_or(&tile::NONE_INFO);

                pod.index = info.atlas_index;
                pod.weight = match info.blend_tech {
                    tile::BlendTech::None => u16::MAX,
                    tile::BlendTech::Weight(w) => w,
                };
            });
    }
}

fn get_data_pod_layer(layer: LayerIndex, tile_data_image: &mut Image) -> &mut [TilePod] {
    let begin = layer.base_index();
    let end = begin + LAYER_SIZE;

    &mut bytemuck::cast_slice_mut(
        tile_data_image
            .data
            .as_mut()
            .expect("Material must have been initialized"),
    )[begin..end]
}
