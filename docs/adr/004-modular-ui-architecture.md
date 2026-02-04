# 4. Modular UI Architecture

Date: 2024-10-27
Status: Accepted

## Context

As the SCALE project evolved, the initial approach of handling UI rendering within `src/main.rs` or a single `src/ui.rs` file became unsustainable. We needed to display increasingly complex information—map visualization, entity inspection, logs, and status bars—simultaneously.

Lumping this logic together led to:
1.  **Bloated Files:** `main.rs` was becoming a "God Object".
2.  **Coupling:** Simulation logic and rendering logic were intertwining.
3.  **Cognitive Load:** It was difficult to reason about specific UI components (like the Inspector) in isolation.

## Decision

We have adopted a **Modular UI Architecture** residing in `src/ui/`.

1.  **Domain-Specific Modules:**
    *   `src/ui/map.rs`: Handles the visualization of the `TerrainGrid` and entities on the map. It uses a `RenderCache` to optimize entity lookups.
    *   `src/ui/inspector.rs`: Implements the "Inspector Pattern". It reads the `Selection` resource and renders detailed `ratatui` widgets for the selected Entity or Tile.
    *   `src/ui/chronicle.rs`: Renders the message log/history.
    *   `src/ui/status.rs`: Renders the global resource bar (top of screen).
    *   `src/ui/panels.rs`: Handles high-level layout (splitting the screen into Rects).

2.  **Immediate Mode Rendering:**
    *   We utilize `ratatui`'s immediate mode paradigm.
    *   Functions follow the signature: `fn render_component(frame: &mut Frame, area: Rect, world: &World)`.
    *   These functions query the ECS `World` (read-only) and emit widgets to the `Frame` for the current tick.

3.  **Selection-Driven Inspection:**
    *   The `Selection` resource (in `shared`) acts as the bridge between Input and UI.
    *   The Input System updates `Selection`.
    *   The UI System (`inspector.rs`) reads `Selection` to determine what to show.

## Consequences

**Positive:**
*   **Separation of Concerns:** Rendering logic is strictly separated from simulation logic (`layer1`).
*   **Scalability:** New UI panels can be added as new modules without touching existing ones.
*   **Testability:** Individual render functions can be unit-tested with a mock `World` and `TestBackend`.

**Negative:**
*   **Boilerplate:** Requires passing `Rect`s and `World` references down the stack.
*   **Layout Management:** We manually manage the `Layout` splits in `main.rs` (or `panels.rs`), which can be brittle if screen sizes change drastically.
