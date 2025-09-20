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
