@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
// How many tiles textures there are in the atlas
@group(2) @binding(2) var<uniform> atlas_dims: vec2<u32>;
// How many tiles are there in each chunk mesh
@group(2) @binding(3) var<uniform> tiles_per_chunk: vec2<u32>;
// The size of each individual tile
@group(2) @binding(4) var<uniform> tile_size: vec2<f32>;
// Contains info about each individual tile
@group(2) @binding(5) var tiles_data: texture_2d<u32>;

const DISCARD: TileData = TileData(65535u, 0.0);

struct TileData {
    atlas_index: u32,
    height: f32
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

fn get_tile_data(tile_pos: vec2<i32>) -> TileData {
    let dims = vec2<i32>(textureDimensions(tiles_data));
    if (tile_pos.x < 0 || tile_pos.x >= dims.x || tile_pos.y < 0 || tile_pos.y >= dims.y) {
        return DISCARD;
    }

    // Get the info about current tile
    let data = textureLoad(tiles_data, tile_pos, 0);

    // Get the desired atlas texture index to render on current tile;
    let atlas_index =  data.r;
    let height = f32(data.g);

    return TileData(atlas_index, height);
}

fn blend_neighbors(grid_pos: vec2<f32>) -> vec4<f32> {
    let tile_pos = vec2<i32>(floor(grid_pos));

    let center_uv = fract(grid_pos);
    let center_pos = tile_pos;

    var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var total_influence = 0.0;

    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            let xy = vec2<i32>(x, y);
            let nbor_pos = center_pos + xy;

            let nbor_tile_data = get_tile_data(nbor_pos);

            if (nbor_tile_data.atlas_index == DISCARD.atlas_index) {
                continue;
            }
            
            let nbor_uv_center = center_uv - vec2<f32>(xy);
            let dist = length(nbor_uv_center);

            let linear_influence = max(0.0, 1.0 - dist);

            let nbor_influence = linear_influence * linear_influence * (3.0 - 2.0 * linear_influence);

            let nbor_color = get_atlas_index_color(nbor_tile_data.atlas_index, center_uv);

            final_color += nbor_color * nbor_influence; 
            total_influence += nbor_influence;
        }
    }

    if (total_influence > 0.0) {
        return final_color / total_influence;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let grid_pos = vec2<f32>(in.world_pos.x, in.world_pos.y) / tile_size;

    // Calculate the current tile position in the chunk mesh;
    let tile_pos = vec2<i32>(floor(grid_pos));

    // Get the info about current tile
    let tile_data = get_tile_data(tile_pos);

    if (tile_data.atlas_index == DISCARD.atlas_index) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }

    return blend_neighbors(grid_pos);
    //return get_atlas_index_color(tile_data.atlas_index, fract(grid_pos));
}
