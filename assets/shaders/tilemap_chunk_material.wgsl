#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile_ids: vec4<u32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile_ids: vec4<u32>
}

@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
// How many tiles textures there are in the atlas
@group(2) @binding(2) var<uniform> atlas_dims: vec2<u32>;
// How many tiles are there in each chunk mesh
@group(2) @binding(3) var<uniform> tiles_per_chunk: vec2<u32>;
// Contains info about each individual tile
@group(2) @binding(4) var tiles_data: texture_2d<u32>;

const DISCARD = 65535u;

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(in.position, 1.0)
    );
    out.clip_position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);

    out.tile_ids = in.tile_ids;
    out.uv = in.uv;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tile_index = 0u;

    if (tile_index == DISCARD) {
        discard;
    }

    // Calculate the top-left corner of the desired tile in the atlas grid.
    let atlas_xy = vec2<f32>(
        f32(tile_index % atlas_dims.y),
        f32(tile_index / atlas_dims.y)
    );

    let tile_uv = fract(in.uv * vec2<f32>(tiles_per_chunk));

    // Combine the atlas corner with the interpolated tile_uv
    let final_uv = (atlas_xy + tile_uv) / vec2<f32>(atlas_dims);

    return textureSample(atlas_texture, atlas_sampler, final_uv);
}


