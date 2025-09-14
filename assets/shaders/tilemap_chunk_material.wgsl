#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
// How many tiles textures there are in the atlas
@group(2) @binding(2) var<uniform> atlas_dims: vec2<u32>;
// How many tiles are there in each chunk mesh
@group(2) @binding(3) var<uniform> tiles_per_chunk: vec2<u32>;
// Contains info about each individual tile
@group(2) @binding(4) var tiles_data: texture_2d<u32>;

const DISCARD = 65535u;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // flip the UV, since shaders works top-down;
    let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    // Calculate the current tile position in the chunk mesh;
    let tile_pos = vec2<u32>(floor(uv * vec2<f32>(tiles_per_chunk)));

    // Get the info about current tile
    let tile_data = textureLoad(tiles_data, tile_pos, 0);

    // Get the desired atlas texture index to render on current tile;
    let tile_index = tile_data.x;

    if (tile_index == DISCARD) {
        discard;
    }

    // TODO: Add other data in the future

    // Calculate the UV relative to the tile;
    let tile_uv = fract(uv * vec2<f32>(tiles_per_chunk));

    // Calculate the coord on the desired tile texture inside atlas;
    let atlas_uv = vec2<f32>(
        f32(tile_index % atlas_dims.y),
        floor(f32(tile_index) / f32(atlas_dims.y))
    );

    // Compute the final coords and convert it to UV (0.0, 0.0) to (1.0, 1.0)
    let final_uv = (atlas_uv + tile_uv) / vec2<f32>(atlas_dims);

    return textureSample(atlas_texture, atlas_sampler, final_uv);
}
