#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
}

@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
// How many tiles textures there are in the atlas
@group(2) @binding(2) var<uniform> atlas_dims: vec2<u32>;
// How many tiles are there in each chunk mesh
@group(2) @binding(3) var<uniform> tiles_per_chunk: vec2<u32>;
// Contains info about each individual tile
@group(2) @binding(4) var<storage, read> tiles_info: array<TilePod>;

const DISCARD = 0u;
const DISCARD_COLOR = vec4<f32>(0.01, 0.01, 0.015, 1.0);
const MAX_TILE_COUNT = 512.0;

struct TilePod {
    atlas_index: u32,
    height: f32,
    // TODO: Add more info here.
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tile_ids: vec4<u32>
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(in.position, 1.0)
    );
    out.clip_position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);

    out.uv = in.uv;

    let infos = array<TilePod, 4>(
        tiles_info[in.tile_ids[0]],
        tiles_info[in.tile_ids[1]],
        tiles_info[in.tile_ids[2]],
        tiles_info[in.tile_ids[3]],
    );

    out.atlas_index = vec4<u32>(
        infos[0].atlas_index,
        infos[1].atlas_index,
        infos[2].atlas_index,
        infos[3].atlas_index,
    );
    
    out.height = vec4<f32>(in.tile_ids) / vec4<f32>(MAX_TILE_COUNT);

    return out;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) atlas_index: vec4<u32>,
    @location(3) height: vec4<f32>,
}

fn get_tile_color(atlas_index: u32, chunk_uv: vec2<f32>) -> vec4<f32> {
    if atlas_index == DISCARD {
        return DISCARD_COLOR;
    }

    // Calculate the top-left corner of the desired tile in the atlas grid.
    let atlas_uv = vec2<f32>(
        f32(atlas_index % atlas_dims.y),
        f32(atlas_index / atlas_dims.y)
    );

    let tile_uv = fract(chunk_uv * vec2<f32>(tiles_per_chunk));

    // Combine the atlas corner with the interpolated tile_uv
    let final_uv = (atlas_uv + tile_uv) / vec2<f32>(atlas_dims);

    return textureSample(atlas_texture, atlas_sampler, final_uv);
}

//@fragment
//fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//    let color_tr = get_tile_color(in.atlas_index[0], in.uv);
//    let color_tl = get_tile_color(in.atlas_index[1], in.uv);
//    let color_bl = get_tile_color(in.atlas_index[2], in.uv);
//    let color_br = get_tile_color(in.atlas_index[3], in.uv);
//
//    let local_uv = fract(in.uv * vec2<f32>(tiles_per_chunk));
//
//    let h_t = mix(in.height[1], in.height[0], local_uv.x);
//    let h_b = mix(in.height[3], in.height[2], local_uv.x);
//    let h = mix(h_b, h_t, local_uv.y);
//
//    // 3. Bilinearly interpolate the four colors.
//    // First, mix the top and bottom colors horizontally using the x blend factor.
//    let top_mix = mix(color_tl, color_tr, h);
//    let bottom_mix = mix(color_bl, color_br, h);
//
//    // Then, mix the results vertically using the y blend factor.
//    let final_color = mix(bottom_mix, top_mix, h_b);
//
//    return final_color;
//}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color_tr = get_tile_color(in.atlas_index[0], in.uv);
    let color_tl = get_tile_color(in.atlas_index[1], in.uv);
    let color_bl = get_tile_color(in.atlas_index[2], in.uv);
    let color_br = get_tile_color(in.atlas_index[3], in.uv);

    let local_uv = fract(in.uv * vec2<f32>(tiles_per_chunk));

    let top_height = mix(0.3, 0.2, local_uv.x);
    let bot_height = mix(0.1, 0.0, local_uv.x);

    let pixel_height = mix(top_height, bot_height, local_uv.y);

    let top_color =  mix(color_tr, color_tl, top_height);
    let bot_color =  mix(color_br, color_bl, bot_height);

    let final_color = mix(top_color, bot_color, pixel_height);

    return final_color;
}
