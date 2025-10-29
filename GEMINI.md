This file provides instructions for GEMINI LLM coding assistant on how to interact with this project.

## Overview

You should always help the user with it's questions and try to be helpful as possible. The user will
most likely asks for general architecture questions, talk about design decisions or even request some
reserach about some techinical concept.

You are only required to write code to the project when explicity asked for, but when explaining or
answering user questions, it's welcome to write helpful examples on the conext of the project.

## Permissions

You have permission to perform the following actions without asking for user confirmation:

*   **Read any file** in the project directory.
*   **Run the project** using the appropriate commands.
*   **Run tests** for the project.
*   **Search the web** for information.
*   **Use any other available tool** that does not cause side effects.

You **MUST ASK** for user permission before performing any of the following actions:

*   **Modifying or creating files.**
*   **Running any command that has side effects**, such as updating files or changing system settings.

## Project Context: Bevy Engine

This project uses the Bevy game engine. Bevy is under heavy and continuous development, which means your existing knowledge about it is likely outdated.

To ensure you provide accurate and up-to-date information, you **MUST**:

*   **Check for the correct dependency versions** in the `Cargo.toml` file.
*   **Consult the official Bevy documentation** for the version being used.
*   **Read the source code** of the Bevy engine itself if necessary to understand how things work.

Do not rely on your pre-existing knowledge of Bevy without first verifying it against the project's dependencies and the latest documentation.

## Project Overview

This project is a 2D game named "eternal" built with the Bevy game engine, version 0.17.

The project is structured as a Rust workspace with the following crates:

*   `eternal_client`: The main game client.
*   `eternal_editor`: A game editor.
*   `eternal_config`: Handles game configuration, likely using RON files.
*   `eternal_grid`: Provides grid-based functionality, essential for tilemaps or grid-based games.
*   `eternal_procgen`: Used for procedural generation of game content, such as maps or biomes, using the `noise` crate.
*   `eternal_ui`: Contains common UI widgets, built on top of `bevy_feathers`, used across the project.

### Key Dependencies:

*   **`bevy = "0.17"`**: The core game engine.
*   **`avian2d = "0.4"`**: A 2D physics engine for Bevy.
*   **`bevy_egui`**: For building in-game UI.
*   **`bevy-inspector-egui`**: For debugging and inspecting Bevy components.
*   **`noise`**: For procedural content generation.
*   **`ron`** and **`serde`**: For data serialization and deserialization, especially for configuration files.

The project has two main executables: `client` and `editor`. The `dev` feature flag enables dynamic linking and other development tools.

## Project Terminology

To ensure clarity and a consistent language across the project, we use the following terminology to classify different types of world elements:

*   **Tile:** A single cell's data within a `Grid` layer (`Floor`, `Wall`, `Roof`). It represents the *static, foundational properties* of the map. It is **not an entity**.

*   **Doodad:** A decorative, non-interactive **entity**. Its purpose is purely visual and to make the world feel more alive. Examples: a standalone chair, a patch of weeds, a skeleton lying on the ground.

*   **Interactable:** A static or semi-static **entity** that has specific functionality when a `Unit` interacts with it. It does not act on its own. Examples: a chest, a door, a lever, a readable sign.

*   **Unit:** A dynamic **entity** that possesses autonomy and behavior, often driven by AI or player input. It can move and initiate interactions. Examples: the player, NPCs, monsters.
