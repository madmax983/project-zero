# 2. Bevy ECS with Ratatui Rendering

Date: 2024-10-24
Status: Accepted

## Context

We need a robust simulation engine to handle complex game state (thousands of entities, systems, resources) but we have chosen a Terminal User Interface (TUI) for the presentation layer to maintain a specific aesthetic and low resource footprint.

Bevy is a popular Rust game engine, but it is tightly coupled with its own rendering engine (wgpu) and windowing (winit) in its default configuration. Ratatui is the standard for rich TUIs in Rust.

## Decision

We will use `bevy_ecs` as a standalone crate for the data model and simulation logic. We will NOT use the full `bevy` engine crate (App, Plugin, rendering).

We will use `ratatui` with `crossterm` for the presentation layer.

We will implement a custom "Game Loop" in `main.rs` that:
1.  Polls Input (Crossterm).
2.  Runs the ECS Schedule (`world.run_schedule`).
3.  Renders the World State to the Terminal Frame (Ratatui).

## Consequences

**Positive:**
-   **Flexibility:** We have full control over the game loop and can optimize the TUI rendering separately from the simulation tick.
-   **Performance:** Avoiding the overhead of a full graphical engine.
-   **Architecture:** Separation of Model (ECS) and View (Ratatui) is enforced by the tool boundary.

**Negative:**
-   **Manual Plumbing:** We lose Bevy's built-in plugins for Input, Time, and States. We must re-implement these (e.g., `SimulationTime` resource, `InputRouter`).
-   **Ecosystem Compatibility:** Many Bevy plugins assume the full engine and won't work with just `bevy_ecs`.
-   **Boilerplate:** We have to manually manage the terminal lifecycle (raw mode, alternate screen).
