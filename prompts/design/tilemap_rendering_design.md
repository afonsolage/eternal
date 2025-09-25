# Tilemap Design and Rendering Strategy Summary

This document summarizes the design discussion for a flexible and performant tilemap rendering
system for a 2D Bevy game. The goal is to support a data-driven workflow where tile
definitions can be edited without recompiling, while ensuring high performance by using
texture atlases.

## Phase 1: Core Data-Driven Design

The initial proposal is to separate tile data from visual representation.

### 1. Configuration (`tile_definitions.ron`)

A central RON file in `assets/` defines the properties of each tile. This allows for easy
editing.

```rust,ignore
// assets/tiles/tile_definitions.ron
(
  tiles: {
    "grass": (
      texture: "tiles/textures/grass_center.png",
      tile_type: "Terrain",
      collision: "Passable",
    ),
    "stone_wall": (
      texture: "tiles/textures/stone_wall.png",
      tile_type: "Structure",
      collision: "Solid",
    ),
  },
)
```

### 2. Runtime Atlas Generation

A Bevy `Startup` system performs the following:
1.  **Load Assets:** Loads the `.ron` file and all individual `.png` textures it references.
2.  **Generate Atlas:** Stitches all loaded images into a single, large texture atlas in
    memory.
3.  **Create Bevy Resources:**
    *   Creates a `Handle<Image>` and a `TextureAtlasLayout` for the generated atlas.
    *   Creates a final `Resource` (e.g., `TileDataMap`) that maps tile IDs (`"grass"`) to
        their final data, including their calculated index in the atlas and their behavioral
        properties (collision, etc.).

### 3. Pros & Cons

*   **Pros:** High flexibility (no recompiling), high performance (single texture atlas),
    clean code (logic uses the final resource, not raw files).
*   **Cons:** Potential for large atlas sizes, minor startup delay for generation, complexity
    in handling hot-reloading.

---

## Phase 2: Handling Complex Tile Variations

To support more advanced tiles, the `visuals` definition in the RON file is expanded into an
enum.

### 1. Evolved Configuration

```rust,ignore
// assets/tiles/tile_definitions.ron
(
  tiles: {
    // Blob Tile (e.g., Grass for auto-tiling)
    "grass": (
      visuals: (
        type: "Blob",
        texture: "tiles/textures/autotile/grass_blob_sheet.png",
        ruleset: "Blob47", 
      ),
      // ...
    ),

    // Animated Tile (e.g., Flower)
    "flower_red": (
      visuals: (
        type: "Animated",
        texture: "tiles/textures/animated/flower_red_strip.png",
        frames: 4,
        frame_duration: 0.25,
      ),
      // ...
    ),

    // Combined Animated Blob Tile (e.g., Water)
    "water": (
      visuals: (
        type: "AnimatedBlob",
        texture: "tiles/textures/autotile/water_animated_blob_sheet.png",
        ruleset: "Blob47",
        frames: 4,
        frame_duration: 0.3,
      ),
      // ...
    ),
  },
)
```

### 2. Implementation

*   The Rust structs now include a `Visuals` enum with variants for `Single`, `Animated`,
    `Blob`, and `AnimatedBlob`.
*   The atlas generator becomes more complex, slicing sprite sheets and strips according to
    the `visuals` type.
*   The final `TileDataMap` resource stores more complex data, like `HashMap<u8, u32>` to map
    neighbor bitmasks to blob tile indices.
*   **Recommendation:** Implement incrementally: `Single`/`Animated` -> `Blob` ->
    `AnimatedBlob`.

---

## Phase 3: Advanced Shader-Based Blending

To avoid the massive art workload of creating blob tiles and to achieve smooth, procedural transitions, a shader-based approach is recommended. There are two main levels of sophistication.

### 3.1. Approach A: Texture Splatting with Masks

This is a simpler shader technique that replaces blob art with a generic blob mask.

#### The Technique

1.  **Art Assets:** Use seamless, tiling base textures for each terrain type (e.g., `grass.png`, `sand.png`) and **one** generic 47-variant blob *mask* atlas.
2.  **Game Logic:** The CPU determines which two textures to blend (e.g., grass and sand) and which mask to use based on neighbors.
3.  **Custom Shader:** A custom WGSL shader receives the two base textures and the mask. It then blends the two textures pixel-by-pixel using the mask value as the factor (e.g., `final_color = mix(color_A, color_B, mask_value);`).

#### Pros & Cons

*   **Pros:** Reduces art requirements (`N` terrains need `N` textures + 1 mask atlas, not `N * 47` tile atlases).
*   **Cons:** Blends can look generic. Most importantly, **it does not solve complex edge cases**, as it's difficult to manage blending between 3 or 4 different terrain types at a single corner.

---

### 3.2. Approach B: Height-Map Blending (Recommended)

This is a more advanced and robust technique that provides superior blending quality and elegantly solves the multi-terrain corner problem. This is the recommended approach.

#### The Technique

1.  **Art Assets:** Requires only **one** seamless, tiling texture for each terrain type (e.g., `grass.png`, `sand.png`, `water.png`). No transition tiles or masks are needed.

2.  **Data-Driven Heights:** In the tile configuration `.ron` file, each terrain type is assigned a numeric `height` that defines its blending priority.
    ```rust,ignore
    // assets/config/tiles.ron
    (
      "water": ( height: 0.1, ... ),
      "sand":  ( height: 0.2, ... ),
      "grass": ( height: 0.5, ... ),
      "rock":  ( height: 0.8, ... ),
    )
    ```

3.  **Mesh Generation:** The world is divided into chunks for frustum culling. However, each chunk's mesh is no longer a single quad. Instead, it's a grid of quads (one per tile).
    *   For a `16x16` tile chunk, a mesh with `17x17` vertices is generated.
    *   For each vertex, the CPU looks at the 4 terrain tiles that meet at its location in the world grid.
    *   It fetches the `height` for each of these 4 terrains and passes them to the vertex shader as a custom attribute (e.g., `vec4<f32>`).

4.  **Custom Shader (WGSL):**
    *   The vertex shader passes the corner heights to the fragment shader. The GPU automatically interpolates these values across the face of the tile quad.
    *   The fragment shader receives the interpolated heights and calculates a single `pixel_height` for that specific fragment using bilinear interpolation.
    *   It then uses a series of `if/else if` checks to determine which two terrain layers the `pixel_height` falls between (e.g., between `HEIGHT_SAND` and `HEIGHT_GRASS`).
    *   It normalizes the height within that specific range to a `0.0-1.0` value and uses `smoothstep` to create a clean blend factor.
    *   Finally, it samples the two corresponding terrain textures and `mix()`es them.

#### Pros & Cons

*   **Pros:**
    *   **Superior Blending:** Elegantly and correctly solves the 3- and 4-terrain corner blending problem.
    *   **Minimal Art Assets:** The most art-efficient method possible (`N` terrains require only `N` base textures).
    *   **Highly Scalable & Data-Driven:** New terrains can be added simply by providing a texture and a `height` value in the configuration file.
    *   **High Performance:** Fully compatible with chunk-based frustum culling.
*   **Cons:**
    *   **Implementation Complexity:** This is the most complex approach, requiring more intricate mesh generation logic on the CPU and a more sophisticated shader.

### Verdict

The **Height-Map Blending** technique is the definitive professional-grade solution for a procedurally generated game where art scalability and high-quality, complex transitions are a priority. While more complex to implement, it solves the critical limitations of all other approaches. It is highly recommended to **prototype** this technique on a small scale to validate the implementation before a full rollout.