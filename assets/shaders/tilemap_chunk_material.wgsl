#import bevy_sprite::{mesh2d_functions}

@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
// How many tiles textures there are in the atlas
@group(2) @binding(2) var<uniform> atlas_dims: vec2<u32>;
// How many tiles are there in each chunk mesh
@group(2) @binding(3) var<uniform> tiles_per_chunk: vec2<u32>;
// The size of each individual tile
@group(2) @binding(4) var<uniform> tile_size: vec2<f32>;
// Contains info about each individual tile
@group(2) @binding(5) var tiles_data: texture_2d_array<u32>;

const WEIGHT_NONE = 65535u;
const DISCARD: TileData = TileData(65535u, 0u);
const BORDER_RECT = vec4<f32>(0.3, 0.3, 0.7, 0.7);

struct TileData {
    atlas_index: u32,
    weight: u32,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.uv = in.uv;

    var world_from_local = mesh2d_functions::get_world_from_local(in.instance_index);
    out.world_pos = mesh2d_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(in.position, 1.0)
    );
    out.clip_pos = mesh2d_functions::mesh2d_position_world_to_clip(out.world_pos);

    out.world_normal = mesh2d_functions::mesh2d_normal_local_to_world(in.normal, in.instance_index);

    return out;
}

fn get_atlas_index_color(atlas_index: u32, uv: vec2<f32>) -> vec4<f32> {
    let atlas_uv = vec2<f32>(
        f32(atlas_index % atlas_dims.x),
        f32(atlas_index / atlas_dims.x)
    );

    let inverted_uv = vec2<f32>(uv.x, 1.0 - uv.y);

    let final_uv = (atlas_uv + inverted_uv) / vec2<f32>(atlas_dims);

    return textureSample(atlas_texture, atlas_sampler, final_uv);
}

fn get_tile_data(tile_pos: vec2<i32>, layer: u32) -> TileData {
    let dims = vec2<i32>(textureDimensions(tiles_data));

    if (tile_pos.x < 0 || tile_pos.x > dims.x || tile_pos.y < 0 || tile_pos.y > dims.y) {
        return DISCARD;
    }

    // Get the info about current tile
    let data = textureLoad(tiles_data, tile_pos, layer, 0);

    // Get the desired atlas texture index to render on current tile;
    let atlas_index =  data.r;
    let weight = data.g;

    return TileData(atlas_index, weight);
}

fn is_inside_border(uv: vec2<f32>) -> bool {
    return all(vec4(uv > BORDER_RECT.xy, uv < BORDER_RECT.zw));
}

fn get_border_dir(uv: vec2<f32>) -> vec2<i32> {
    let above_max = step(BORDER_RECT.zw, uv);
    let bellow_min = step(uv, BORDER_RECT.xy);
    return vec2<i32>(above_max - bellow_min);
}

fn calc_blend_factor(uv: vec2<f32>) -> vec2<f32> {
    let alpha_x = smoothstep(BORDER_RECT.x, 0.0, uv.x) + smoothstep(BORDER_RECT.z, 1.0, uv.x);
    let alpha_y = smoothstep(BORDER_RECT.y, 0.0, uv.y) + smoothstep(BORDER_RECT.w, 1.0, uv.y);

    return vec2<f32>(alpha_x, alpha_y);
}

fn blend_neighbors(tile_pos: vec2<i32>, uv: vec2<f32>, layer: u32) -> vec4<f32> {
    let tile_data = get_tile_data(tile_pos, layer);
    var base_color = get_atlas_index_color(tile_data.atlas_index, uv);

    // No blend to do if we are inside the border
    if (is_inside_border(uv)) {
        return base_color;
    }

    // Calculate the blend factor where 0.0 is no blend and 1.0 is high blend.
    // 0.0 means base color and 1.0 means neighbor color.
    // So a blend factor of 0.5 will mix the two color half and half.
    let blend_factor = calc_blend_factor(uv);

    // Get which border the uv is closer, to get the closer neighbor
    let dir = get_border_dir(uv);

    // Get the horizontal, vertical and diagonal neighbor data
    var h_nbor_data = get_tile_data(tile_pos + vec2<i32>(dir.x, 0), layer);
    var v_nbor_data = get_tile_data(tile_pos + vec2<i32>(0, dir.y), layer);
    var d_nbor_data = get_tile_data(tile_pos + vec2<i32>(dir.x, dir.y), layer);

    // If the horizontal neighbor is higher or if the diagonal neighbor is higher than 
    // the vertical neighbor, blend on the horizontal axis.
    var alpha_x = 0.0;
    if ((h_nbor_data.weight < WEIGHT_NONE && h_nbor_data.weight > tile_data.weight) ||
        (d_nbor_data.weight < WEIGHT_NONE && d_nbor_data.weight > v_nbor_data.weight)) {
        alpha_x = blend_factor.x;
    }

    // If the vertical neighbor is higher or if the diagonal neighbor is higher than 
    // the horizontal neighbor, blend on the vertical axis.
    var alpha_y = 0.0;
    if ((v_nbor_data.weight < WEIGHT_NONE && v_nbor_data.weight > tile_data.weight) ||
        (d_nbor_data.weight < WEIGHT_NONE && d_nbor_data.weight > h_nbor_data.weight)) {
        alpha_y = blend_factor.y;
    }

    // Get the neighborhood colors
    let h_color = get_atlas_index_color(h_nbor_data.atlas_index, uv);
    let v_color = get_atlas_index_color(v_nbor_data.atlas_index, uv);
    let d_color = get_atlas_index_color(d_nbor_data.atlas_index, uv);

    // First blend on horizontal axis, then blend on vertical axis
    let bottom_blend = mix(base_color, h_color, alpha_x);
    let top_blend = mix(v_color, d_color, alpha_x);
    let final_color = mix(bottom_blend, top_blend, alpha_y);

    return final_color;
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) layer: u32,
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = textureDimensions(tiles_data);

    // Tile position relative to the current grid;
    var grid_pos = vec2<f32>(in.world_pos.x, in.world_pos.y) / tile_size;
    var uv = fract(grid_pos);

    // Clamp to avoid artifacts on the edge and convert to int
    let tile_pos = clamp(
        vec2<i32>(grid_pos),
        vec2<i32>(0),
        vec2<i32>(dims)
    );

    // Get the info about current tile
    let tile_data = get_tile_data(tile_pos, in.layer);

    if (tile_data.atlas_index == DISCARD.atlas_index) {
        discard;
    }

    return blend_neighbors(tile_pos, uv, in.layer);
}
