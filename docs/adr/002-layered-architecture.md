# 2. Layered Simulation Architecture

Date: 2024-05-23
Status: Accepted

## Context
"Scale" is defined by its ability to zoom from a single colonist's needs up to galactic diplomacy. This involves three distinct scales of simulation:
1. **Colony:** Tile-based, unit-level simulation (Dwarf Fortress style).
2. **System:** Orbital mechanics, interplanetary travel, distinct planetary bodies.
3. **Galaxy:** Star systems, hyperlanes, political boundaries.

Mixing these domains into a single monolithic simulation module would create high coupling and make the system difficult to reason about.

## Decision
We will structure the application into three explicit layers, reflected in the module structure:

*   **Layer 1 (Colony):** Handles tile grids (`TerrainGrid`), individual units ("souls"), and local buildings.
*   **Layer 2 (System):** Handles planetary bodies, fleets, and orbital mechanics.
*   **Layer 3 (Galaxy):** Handles star systems, civilizations, and galactic-level events.

These are implemented as separate Rust modules (`src/layer1/`, `src/layer2/`, `src/layer3/`) and exposed via `src/lib.rs`.

## Consequences
### Positive
*   **Cognitive Load:** Developers can work on the "Colony" simulation without worrying about "Galaxy" state, and vice versa.
*   **Modularity:** Each layer can have its own specific ECS Components and Systems tailored to its abstraction level.

### Negative
*   **Inter-Layer Communication:** Events that cross layers (e.g., orbital bombardment affecting tiles, or a ship launching to space) require explicit bridges.
*   **State Synchronization:** An entity might exist conceptually in multiple layers (e.g., a Planet is a dot in Layer 2, but a `TerrainGrid` in Layer 1). Synchronizing state between these representations is non-trivial.
