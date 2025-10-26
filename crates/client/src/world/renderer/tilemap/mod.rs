use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::VisibilityClass,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    math::U16Vec2,
    mesh::{MeshTag, PrimitiveTopology},
    platform::collections::HashMap,
    prelude::*,
    sprite_render::Material2dPlugin,
};

mod material;

use eternal_grid::ecs::TileRegistry;
pub use material::{TilePod, TilemapChunkMaterial};

use crate::{
    ClientState,
    world::{
        grid::{self, GridId, GridIdChanged, LAYER_SIZE, LAYERS, LAYERS_COUNT, LayerIndex},
        tile::{self},
    },
};

pub const TILES_PER_CHUNK: U16Vec2 = U16Vec2::new(32, 32);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(Material2dPlugin::<TilemapChunkMaterial>::default())
            .add_systems(
                Update,
                (update_tilemap_chunk_material
                    .run_if(resource_changed::<TileRegistry>.or(state_changed::<ClientState>)),)
                    .run_if(in_state(ClientState::Playing)),
            )
            .add_observer(on_grid_id_changed);
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
}

impl Default for Tilemap {
    fn default() -> Self {
        Self {
            atlas_texture: Default::default(),
            atlas_dims: UVec2::new(4, 4),
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

fn spawn_single_chunk(
    commands: &mut Commands,
    chunk_pos: TilemapChunkPos,
    parent: Entity,
    TilemapCache { material, mesh }: TilemapCache,
) -> Entity {
    let tile_size = tile::SIZE.as_vec2();
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

    let mut images = world.resource_mut::<Assets<Image>>();
    let tiles_data = images.add(material::init_tile_data(LAYERS_COUNT));

    let mut materials = world.resource_mut::<Assets<TilemapChunkMaterial>>();
    let material = materials.add(TilemapChunkMaterial {
        atlas_texture,
        atlas_dims,
        tiles_per_chunk: TILES_PER_CHUNK.as_uvec2(),
        tile_size: tile::SIZE.as_vec2(),
        tiles_data,
        config: None,
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
            .id();

        for y in 0..chunks_count.y {
            for x in 0..chunks_count.x {
                let chunk_pos = TilemapChunkPos {
                    xy: U16Vec2::new(x, y),
                    layer,
                };
                let chunk_entity =
                    spawn_single_chunk(&mut commands, chunk_pos, layer_entity, cache.clone());

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

fn update_tilemap_chunk_material(
    tilemap: Single<(&GridId, &TilemapCache)>,
    tile_info_map: Res<TileRegistry>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (grid, TilemapCache { material, .. }) = tilemap.into_inner();

    // Using `get_mut` to trigger change detection and update this material on render world
    let Some(material) = materials.get_mut(material.id()) else {
        warn!("Failed to update tilemap material. Material not found.");
        return;
    };

    let Some(tile_data_image) = images.get_mut(material.tiles_data.id()) else {
        warn!("Failed to update tilemap material. Tile data not found.");
        return;
    };

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
                    tile::BlendTech::None => u8::MAX,
                    tile::BlendTech::Weight(w) => w,
                };
                pod.outline = if info.outline { 1 } else { 0 };
            });
    }
}

fn on_grid_id_changed(
    changed: On<GridIdChanged>,
    tilemap: Single<(&GridId, &TilemapCache)>,
    tile_info_map: Res<TileRegistry>,
    mut materials: ResMut<Assets<TilemapChunkMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (grid, TilemapCache { material, .. }) = tilemap.into_inner();

    // Using `get_mut` to trigger change detection and update this material on render world
    let Some(material) = materials.get_mut(material.id()) else {
        warn!("Failed to update tilemap material. Material not found.");
        return;
    };

    let Some(tile_data_image) = images.get_mut(material.tiles_data.id()) else {
        warn!("Failed to update tilemap material. Tile data not found.");
        return;
    };

    let GridIdChanged(layer, positions) = &*changed;

    let tile_data_pods = get_data_pod_layer(*layer, tile_data_image);
    let grid_layer = &grid[*layer];

    for &U16Vec2 { x, y } in positions {
        let id = grid_layer.get(x, y);
        let info = tile_info_map.get(id).unwrap_or(&tile::NONE_INFO);

        let pod = &mut tile_data_pods[grid::to_index(x, y)];
        pod.index = info.atlas_index;
        pod.weight = match info.blend_tech {
            tile::BlendTech::None => u8::MAX,
            tile::BlendTech::Weight(w) => w,
        };
        pod.outline = if info.outline { 1 } else { 0 };
    }
}
