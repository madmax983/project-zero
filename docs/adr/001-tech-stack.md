# 1. Use Bevy ECS and Ratatui

Date: 2024-05-23
Status: Accepted

## Context
We are building "Scale", a 4X colony simulation game that operates on three distinct layers (Colony, System, Galaxy). The game aims for deep simulation ("Dwarf Fortress style") which implies tracking a large number of entities and interactions. The target platform is the terminal.

We need:
1. A performance-oriented architecture to handle the simulation state.
2. A robust library to render the user interface in a terminal.

## Decision
We will use **Bevy ECS** (`bevy_ecs` crate) as the core architecture for simulation state and logic.
We will use **Ratatui** (`ratatui` crate) backed by **Crossterm** for the Terminal User Interface.

As observed in the codebase (`src/main.rs`), we are using `bevy_ecs` as a standalone crate, not the full `bevy` engine. The main loop is manually implemented to coordinate input (Crossterm), simulation ticks (Bevy Schedule), and rendering (Ratatui).

## Consequences
### Positive
- **Data-Oriented Design:** Bevy ECS allows for efficient cache usage and parallel system execution, critical for scaling the simulation to thousands of entities.
- **Decoupled Logic:** Simulation logic (Systems) is separated from data (Components) and rendering.
- **Rich TUI:** Ratatui provides high-level widgets and layout primitives, making it easier to build complex UIs than raw terminal escape codes.

### Negative
- **Manual Integration:** We cannot use Bevy's built-in rendering or windowing plugins. We must manually bridge the ECS world state to the Ratatui render pass.
- **Terminal Limitations:** Visualization is limited to character cells, which constraints how much information can be displayed densely.
- **Boilerplate:** Using `bevy_ecs` standalone requires setting up our own scheduling and loop boilerplate, unlike using `App::new()` in full Bevy.
