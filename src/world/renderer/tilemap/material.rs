use bevy::{
    asset::{Asset, Handle, RenderAssetUsages},
    image::{Image, ImageSampler},
    math::UVec2,
    mesh::{Mesh, MeshVertexAttribute, VertexFormat},
    reflect::Reflect,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
    sprite_render::Material2d,
};
use bytemuck::{NoUninit, Pod, Zeroable};

use super::TILES_PER_CHUNK;

const FRAGMENT_SHADER_PATH: &str = "shaders/tilemap_chunk_material.wgsl";

pub const ATTRIBUTE_TILE_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("TileId", 100000, VertexFormat::Uint32x4);

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct TilePod {
    pub atlas_index: u32,
    pub height: f32,
    // TODO: Add more info here.
}

impl TilePod {
    pub fn discard() -> Self {
        Self {
            atlas_index: 0,
            ..Default::default()
        }
    }
}

#[derive(Asset, Default, AsBindGroup, Clone, Debug, Reflect)]
pub struct TilemapChunkMaterial {
    /// Texture image of the atlas
    #[texture(0, dimension = "2d")]
    #[sampler(1)]
    pub atlas_texture: Handle<Image>,
    /// How many tile textures there are in the atlas
    #[uniform(2)]
    pub atlas_dims: UVec2,
    // How many tiles are there in each chunk mesh
    #[uniform(3)]
    pub tiles_per_chunk: UVec2,
    /// The encoded ``TilePod`` to be sent to fragment shader
    #[storage(4, read_only)]
    pub tiles_info: Handle<ShaderStorageBuffer>,
}

impl Material2d for TilemapChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        FRAGMENT_SHADER_PATH.into()
    }

    fn vertex_shader() -> ShaderRef {
        FRAGMENT_SHADER_PATH.into()
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::sprite_render::Material2dKey<Self>,
    ) -> bevy::ecs::error::Result<(), bevy::render::render_resource::SpecializedMeshPipelineError>
    {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            ATTRIBUTE_TILE_ID.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub fn init_tile_data() -> Image {
    let empty_data = vec![0xFF; TILES_PER_CHUNK.element_product() as usize * size_of::<TilePod>()];
    Image {
        data: Some(empty_data),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: TILES_PER_CHUNK.x as u32,
                height: TILES_PER_CHUNK.y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R16Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::nearest(),
        asset_usage: RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        ..Default::default()
    }
}
