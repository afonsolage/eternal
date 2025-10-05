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
const BLEND_RECT = vec4<f32>(0.3, 0.3, 0.7, 0.7);

const WALL_RECT = vec4<f32>(0.3, 0.3, 0.7, 0.7);
const WALL_OUTLINE = vec4<f32>(0.03, 0.03, 0.03, 1.0);

const SHADOW_INTENSITY = 0.8;

const FLOOR_LAYER = 0u;
const WALL_LAYER = 1u;

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
    out.layer = mesh2d_functions::get_tag(in.instance_index);

    return out;
}

/// Get the final color of the uv at the atlas index.
fn get_atlas_index_color(atlas_index: u32, uv: vec2<f32>) -> vec4<f32> {
    let atlas_uv = vec2<f32>(
        f32(atlas_index % atlas_dims.x),
        f32(atlas_index / atlas_dims.x)
    );

    let inverted_uv = vec2<f32>(uv.x, 1.0 - uv.y);

    let final_uv = (atlas_uv + inverted_uv) / vec2<f32>(atlas_dims);

    return textureSample(atlas_texture, atlas_sampler, final_uv);
}

/// Get the tile data at the given tile position and layer.
/// Returns `DISCARD` data if the position is invalid
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

/// Check if a given UV is inside the given rect (min_x, min_y, max_x, max_y)
fn is_inside_rect(uv: vec2<f32>, rect: vec4<f32>) -> bool {
    return all(vec4(uv > rect.xy, uv < rect.zw));
}

/// Return the direction of the closest boder
/// (-1, 1) means it is closer to the top-left border
fn get_border_dir(uv: vec2<f32>) -> vec2<i32> {
    let above_max = step(BLEND_RECT.zw, uv);
    let bellow_min = step(uv, BLEND_RECT.xy);
    return vec2<i32>(above_max - bellow_min);
}

/// Calculate the blend factor where 0.0 is no blend and 1.0 is high blend.
/// 0.0 means base color and 1.0 means neighbor color.
/// So a blend factor of 0.5 will mix the two color half and half.
fn calc_blend_factor(uv: vec2<f32>) -> vec2<f32> {
    let alpha_x = smoothstep(BLEND_RECT.x, 0.0, uv.x) + smoothstep(BLEND_RECT.z, 1.0, uv.x);
    let alpha_y = smoothstep(BLEND_RECT.y, 0.0, uv.y) + smoothstep(BLEND_RECT.w, 1.0, uv.y);

    return vec2<f32>(alpha_x, alpha_y);
}

/// Blend the colors of the neighbor floors at edge transition.
fn blend_floor_neighbors(tile_pos: vec2<i32>, uv: vec2<f32>) -> vec4<f32> {
    let tile_data = get_tile_data(tile_pos, FLOOR_LAYER);
    var base_color = get_atlas_index_color(tile_data.atlas_index, uv);

    // No blend to do if we are inside the border
    if (is_inside_rect(uv, BLEND_RECT)) {
        return base_color;
    }

    let blend_factor = calc_blend_factor(uv);

    // Get which border the uv is closer, to get the closer neighbor
    let dir = get_border_dir(uv);

    // Get the horizontal, vertical and diagonal neighbor data
    var h_nbor_data = get_tile_data(tile_pos + vec2<i32>(dir.x, 0), FLOOR_LAYER);
    var v_nbor_data = get_tile_data(tile_pos + vec2<i32>(0, dir.y), FLOOR_LAYER);
    var d_nbor_data = get_tile_data(tile_pos + vec2<i32>(dir.x, dir.y), FLOOR_LAYER);

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

/// Check if there is a wall in nearby in the layer above and cast a shadow
fn cast_shadow(tile_pos: vec2<i32>, uv: vec2<f32>, color: vec4<f32>) -> vec4<f32> {
    // if there is an wall above, just cast shadow max intensity
    if (get_tile_data(tile_pos, WALL_LAYER).atlas_index != DISCARD.atlas_index) {
        return vec4<f32>(color.rgb * (1.0 - SHADOW_INTENSITY), color.a);
    }

    // NOTE: It may be possible to use this to cast directional shadow, base on sun dir.
    let shadow_thickness = vec4<f32>(0.5, 0.5, 0.5, 0.5);

    var total_intensity = 0.0;

    // Pre-calculate shadow values for each edge based on the fragment's uv
    let top_shadow = smoothstep(1.0 - shadow_thickness.w, 1.0, uv.y);
    let bottom_shadow = 1.0 - smoothstep(0.0, shadow_thickness.y, uv.y);
    let right_shadow = smoothstep(1.0 - shadow_thickness.z, 1.0, uv.x);
    let left_shadow = 1.0 - smoothstep(0.0, shadow_thickness.x, uv.x);

    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            if (x == 0 && y == 0) { continue; }

            let dir = vec2<i32>(x, y);
            let neighbor_wall = get_tile_data(tile_pos + dir, WALL_LAYER);

            if (neighbor_wall.atlas_index != DISCARD.atlas_index) {
                var shadow = vec2<f32>(0.0);

                if (dir.x > 0) {
                    shadow.x = right_shadow;
                } else if (dir.x < 0) {
                    shadow.x = left_shadow;
                }

                if (dir.y > 0) {
                    shadow.y = top_shadow;
                } else if (dir.y < 0) {
                    shadow.y = bottom_shadow;
                }

                var final_shadow = 0.0;

                if (dir.x != 0 && dir.y != 0) {
                    final_shadow = shadow.x * shadow.y;
                } else {
                    final_shadow = max(shadow.x, shadow.y);
                }

                total_intensity = max(total_intensity, final_shadow);
            }
        }
    }

    if (total_intensity > 0) {
        return vec4<f32>(color.rgb * (1.0 - total_intensity * SHADOW_INTENSITY), color.a);
    } else {
        return color;
    }
}

/// Get the final color of the wall outline. This function computes the wall
/// color and the corners of the wall, to give a better illusion of depth.
fn get_wall_color(uv: vec2<f32>, tile_pos: vec2<i32>) -> vec4<f32> {
    let is_at_diag = abs(uv.x - uv.y) < 0.01 || abs(uv.x + uv.y - 1.0) < 0.01;

    // If the current uv coordinates is in a "cross"-shape, with matches the corner
    if (is_at_diag) {
        let dir = get_border_dir(uv);
        let d_nbor_data = get_tile_data(tile_pos + vec2<i32>(dir.x, dir.y), WALL_LAYER);

        // Only draw the corner depth color if there is no neighbor
        if (d_nbor_data.atlas_index == DISCARD.atlas_index) {
            let h_nbor_data = get_tile_data(tile_pos + vec2<i32>(dir.x, 0), WALL_LAYER);
            let v_nbor_data = get_tile_data(tile_pos + vec2<i32>(0, dir.y), WALL_LAYER);

            if (h_nbor_data.atlas_index == DISCARD.atlas_index &&
                v_nbor_data.atlas_index == DISCARD.atlas_index) {

                // If both neighbor are missing, draw a brigther color
                return vec4<f32>(WALL_OUTLINE.xyz * 2.0, 1.0);

            } else if (h_nbor_data.atlas_index != DISCARD.atlas_index &&
                       v_nbor_data.atlas_index != DISCARD.atlas_index) {

                // If both neighbor are present, draw a darker color
                return vec4<f32>(WALL_OUTLINE.xyz * 0.5, 1.0);
            }
        }
    }

    return WALL_OUTLINE;
}

/// Draw the wall outline. This is need to give the illusion of dept when
/// drawing the tile of a wall, which will be ablove some floor tile.
fn draw_outline_wall(tile_pos: vec2<i32>, uv: vec2<f32>) -> vec4<f32> {
    let tile_data = get_tile_data(tile_pos, WALL_LAYER);
    var base_color = get_atlas_index_color(tile_data.atlas_index, uv);

    // The `WALL_RECT` defines the width of the wall outline
    if (!is_inside_rect(uv, WALL_RECT)) {
        for (var y = -1; y <= 1; y = y + 1) {
            for (var x = -1; x <= 1; x = x + 1) {
                if (x == 0 && y == 0) { continue ;}

                let dir = vec2<i32>(x, y);

                // Only draw if this draw is near an valid edge of the wall rect
                let h_valid = (x > 0 && uv.x > WALL_RECT.z) || (x < 0 && uv.x < WALL_RECT.x) || (x == 0);
                let v_valid = (y > 0 && uv.y > WALL_RECT.w) || (y < 0 && uv.y < WALL_RECT.y) || (y == 0);
       
                if (h_valid && v_valid) {
                    // If there is no tile neighboring this tile
                    let nbor_data = get_tile_data(tile_pos + dir, WALL_LAYER);
                    if (nbor_data.atlas_index == DISCARD.atlas_index) {
                        return get_wall_color(uv, tile_pos);
                    }
                }

            }
        }
    }

    return base_color;
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

    // If there is no valid tile there, just skip this frag
    if (tile_data.atlas_index == DISCARD.atlas_index) {
        discard;
    }

    if in.layer == FLOOR_LAYER {
        let color = blend_floor_neighbors(tile_pos, uv);
        return cast_shadow(tile_pos, uv, color);
    } else if in.layer == WALL_LAYER {
        return draw_outline_wall(tile_pos, uv);
    } else {
        discard;
    }
}
