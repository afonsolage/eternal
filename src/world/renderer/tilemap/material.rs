use bevy::{
    asset::{Asset, Handle, RenderAssetUsages},
    image::{Image, ImageSampler},
    math::{UVec2, Vec2},
    reflect::Reflect,
    render::render_resource::{
        AsBindGroup, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    shader::ShaderRef,
    sprite_render::Material2d,
};
use bytemuck::{Pod, Zeroable};

use crate::world::grid;

use super::TILES_PER_CHUNK;

const FRAGMENT_SHADER_PATH: &str = "shaders/tilemap_chunk_material.wgsl";

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TilePod {
    pub index: u16,  // Red channel
    pub height: u16, // Green channel.
}

impl TilePod {
    pub fn discard() -> Self {
        Self {
            index: u16::MAX,
            height: 0,
        }
    }
}

#[derive(Asset, AsBindGroup, Clone, Debug, Reflect)]
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
    #[uniform(4)]
    pub tile_size: Vec2,
    /// The encoded ``TilePod`` to be sent to fragment shader
    #[texture(5, sample_type = "u_int")]
    pub tiles_data: Handle<Image>,
}

impl Material2d for TilemapChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        FRAGMENT_SHADER_PATH.into()
    }
}

pub fn init_tile_data() -> Image {
    let empty_data = vec![0xFF; grid::DIMS.element_product() as usize * size_of::<TilePod>()];
    Image {
        data: Some(empty_data),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: grid::DIMS.x,
                height: grid::DIMS.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg16Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::nearest(),
        asset_usage: RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        ..Default::default()
    }
}
