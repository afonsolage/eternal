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

## Phase 3: Alternative - Shader-Based Edge Blending

To avoid the massive art workload of creating 47-variant blob tiles for every terrain type.

### 1. The Technique

This approach is a 2D version of **Texture Splatting**.
1.  **Art Assets:** Use seamless, tiling base textures for each terrain type (e.g.,
    `grass.png`, `sand.png`) and **one** generic 47-variant blob *mask* atlas.
2.  **Game Logic:** The CPU determines which two textures to blend (e.g., grass and sand) and
    which mask to use based on neighbors.
3.  **Custom Shader:** A custom WGSL shader receives the two base textures and the mask. It
    then blends the two textures pixel-by-pixel using the mask value as the factor (e.g.,
    `final_color = mix(color_A, color_B, mask_value);`).

### 2. Pros & Cons

*   **Pros:**
    *   **Massive Art Scalability:** The primary advantage. `N` terrains require only `N` base
        textures, not `N * 47`.
    *   **Flexibility:** New terrains are easy to add.
    *   **Low Memory Footprint.**
*   **Cons:**
    *   **Visual Fidelity:** Blends can look generic or "blurry" compared to hand-authored
        art.
    *   **Shader Complexity:** Requires writing a custom Bevy `Material` and WGSL shader.
    *   **Performance:** Increases texture lookups from 1 to 3 per tile.
    *   **Complex Edge Cases:** Handling corners where 3 or 4 terrains meet is difficult.

### 3. Mitigation & Verdict

The "blurry" look can be improved in the shader by using `step()` or `smoothstep()` for
harder, more stylized edges.

**Verdict:** This is an excellent, professional-grade solution for a proc-gen game where art
scalability is a higher priority than hand-polished transitions. It is highly recommended to
**prototype** this technique on a small scale to validate the visual style before full
implementation.

---

## Phase 4: Shader-Based Weighted Influence Blending

This is a more advanced, fully procedural alternative to the mask-based splatting in Phase 3. It calculates the entire blend dynamically in the fragment shader, offering maximum flexibility.

### 1. The Technique

This method renders the entire tilemap in large, single-quad chunks and performs all blending calculations for every pixel on the GPU.

1.  **Data to GPU:** Two key pieces of information are passed to the shader:
    *   A **Data Texture** (`TileInfoTexture`): A texture where each pixel corresponds to a tile in the world grid. It stores data like the tile's type ID and atlas index. This allows the shader to know about its neighbors at any world position.
    *   **Metadata Buffer:** A storage buffer containing metadata for each tile *type*, such as its `blending_weight`.

2.  **Fragment Shader Logic:** For each pixel on the screen, the shader performs these steps:
    a. It determines its `world_pos` and calculates which tile it is in (`center_tile_coord`) and its position within that tile (`local_uv`).
    b. It loops through a 3x3 grid of tiles centered on the current pixel's tile.
    c. For each of the 9 tiles (center + 8 neighbors), it calculates an **influence** value. The influence is 1.0 at the tile's center and falls off to 0.0 at the center of adjacent tiles. This is done using a linear gradient (`1.0 - abs(distance)`), which creates a diamond-shaped influence field.
    d. This influence value is then multiplied by the `blending_weight` from the tile's metadata. This allows some tiles (e.g., "Stone") to have a stronger, more dominant blend than others (e.g., "Grass").
    e. The final color of the pixel is the weighted average of the colors of all 9 neighbors, based on their final calculated influence. The shader samples the appropriate texture for each neighbor and blends it into the final color.
    f. The result is normalized by the total influence to maintain correct brightness.

### 2. Pros & Cons

*   **Pros:**
    *   **Dynamic & Data-Driven:** Blending behavior (weights) can be configured in asset files without changing art or code.
    *   **Handles Complex Corners:** Naturally handles corners where 3, 4, or more tile types meet by simply blending all of their influences together.
    *   **No Mask Textures:** Simplifies the art pipeline significantly. Only seamless, tiling textures are needed for each terrain type.
    *   **Weighted Blending:** Provides powerful artistic control over the "strength" of different terrain edges.
    *   **Highly Performant:** Keeps vertex counts extremely low. The parallel nature of the fragment shader is perfect for this kind of work.
*   **Cons:**
    *   **Shader Complexity:** The WGSL shader is significantly more complex than any other approach.
    *   **"Blocky" Blending:** The default linear, diamond-shaped influence function results in blends that can look geometric or "blocky", which may or may not be desirable for the game's art style.
    *   **Data Management:** Requires careful management of the `TileInfoTexture` and metadata buffers on the CPU side.
