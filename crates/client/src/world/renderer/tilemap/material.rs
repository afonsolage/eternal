use bevy::{
    asset::{Asset, Handle},
    image::{Image, ImageSampler},
    log::debug,
    math::{UVec2, Vec2},
    reflect::Reflect,
    render::render_resource::{
        AsBindGroup, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};
use bytemuck::{Pod, Zeroable};

use crate::world::grid;

const SHADER_PATH: &str = "shaders/tilemap_chunk_material.wgsl";

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TilePod {
    pub index: u16,  // Red channel
    pub weight: u16, // Green channel.
}

impl TilePod {
    pub fn discard() -> Self {
        Self {
            index: u16::MAX,
            weight: u16::MAX,
        }
    }
}

impl From<&TilemapChunkMaterial> for TilemapChunkMaterialConfig {
    fn from(material: &TilemapChunkMaterial) -> Self {
        material.config.unwrap_or_default()
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Reflect, PartialEq, Eq, Hash)]
pub struct TilemapChunkMaterialConfig {
    pub disable_floor_blending: bool,
    pub wall_hide_outline: bool,
    pub wall_hide_shadow: bool,
}

#[derive(Asset, AsBindGroup, Clone, Debug, Reflect)]
#[bind_group_data(TilemapChunkMaterialConfig)]
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
    #[texture(5, dimension = "2d_array", sample_type = "u_int")]
    pub tiles_data: Handle<Image>,
    pub config: Option<TilemapChunkMaterialConfig>,
}

impl Material2d for TilemapChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::sprite_render::Material2dKey<Self>,
    ) -> bevy::ecs::error::Result<(), bevy::render::render_resource::SpecializedMeshPipelineError>
    {
        let config = key.bind_group_data;
        let fragment = descriptor.fragment.as_mut().unwrap();

        if config.disable_floor_blending {
            fragment.shader_defs.push("DISABLE_FLOOR_BLENDING".into());
        }

        if config.wall_hide_outline {
            fragment.shader_defs.push("WALL_HIDE_OUTLINE".into());
        }

        if config.wall_hide_shadow {
            fragment.shader_defs.push("WALL_HIDE_SHADOW".into());
        }

        debug!("Shader defs: {:?}", fragment.shader_defs);

        Ok(())
    }
}

pub fn init_tile_data(layers: usize) -> Image {
    let empty_data =
        vec![0xFF; grid::DIMS.element_product() as usize * size_of::<TilePod>() * layers];
    Image {
        data: Some(empty_data),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: grid::DIMS.x,
                height: grid::DIMS.y,
                depth_or_array_layers: layers as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg16Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::nearest(),
        ..Default::default()
    }
}
