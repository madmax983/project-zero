# 24. Visual Particle System

Date: 2024-06-03

## Status

Accepted

## Context

The simulation requires visual feedback ("Juice") to communicate events to the player, such as:
*   Work progress (sparks/dust).
*   Combat hits (damage numbers, blood).
*   Status effects (icons above heads).

Implementing these as standard `Entity` components with full simulation overhead (e.g., `GridPosition`, `Health`, `Needs`) is heavy and pollutes the main simulation loop. Furthermore, these entities are ephemeral and should not be saved/loaded or affect the simulation state (e.g., blocking movement).

We needed a lightweight, fire-and-forget system for visual-only entities that can move smoothly between grid tiles (sub-grid movement) for better animation fluidity.

## Decision

We implemented a dedicated **Particle System** in `src/layer1/particles.rs`.

1.  **Particle Component**: A lightweight component (`Particle`) storing only rendering data (`char`, `color`) and `lifetime`.
2.  **Sub-Grid Physics**: A `ParticleVelocity` and `ParticleAccumulator` component allow particles to move fractionally across the grid (e.g., `0.5` tiles per tick), providing smooth animation on top of the discrete grid.
3.  **Simulation Isolation**:
    *   Particles are **not** saved (excluded from serialization).
    *   Particles do **not** block movement or vision.
    *   Particles are automatically despawned when `lifetime` reaches zero.

## Consequences

### Positive
*   **Performance**: Particles are extremely cheap to spawn and update. They do not interact with complex systems like Pathfinding or Needs.
*   **Game Feel**: Sub-grid movement allows for "bouncy" or "floating" effects that break the rigidity of the grid.
*   **Decoupling**: Visuals are strictly separated from logic. A "Hit" event spawns a particle but the combat logic doesn't care about it.

### Negative
*   **Rendering Complexity**: The renderer must handle fractional positions if we want true smooth movement (currently `ParticleAccumulator` quantizes back to grid for the TUI, but the backend supports float offsets).
*   **Entity Count**: High particle counts could still impact ECS iteration performance if not managed (though `lifetime` mitigates this).
