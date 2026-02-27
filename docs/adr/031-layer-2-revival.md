# 31. Layer 2 Revival & Lightweight Simulation

Date: 2025-05-23

## Status

Accepted (Supersedes ADR 003, Refines ADR 016)

## Context

ADR 003 ("Excision of Layers 2 and 3") removed the placeholder system-scale simulation to focus on the colony (Layer 1). Later, ADR 016 ("Layer 2 Bridge Strategy") reintroduced Layer 2 as a pure data layer for "Endgame Viability," explicitly stating it would not be a simulation layer.

However, recent feature development has organically reintroduced active simulation elements to `src/layer2`:
*   **Fleets:** Have travel times, speeds, and fuel consumption (`fleet.rs`).
*   **Debris:** Accumulates from launches/combat and decays over time (`debris.rs`).
*   **Mining:** Fleets actively extract resources from `MiningTarget` entities over multiple ticks (`mining.rs`).
*   **Stations:** Construction logic consumes resources and spawns new entities (`station.rs`).

The "Data Only" constraint of ADR 016 is no longer accurate. We are simulating a system, just not on a grid.

## Decision

We officially revive **Layer 2** as a **Lightweight Simulation Layer**.

1.  **Scope**: Layer 2 manages the orbital and interplanetary scale (System View).
2.  **Abstraction**: Unlike Layer 1 (which uses detailed Tile Grids, Pathfinding, and Physics), Layer 2 uses **Abstract Entities**.
    *   **Position**: Defined by `Orbit` (Parent + Angle) or `InTransit` (Origin -> Destination + Progress).
    *   **Movement**: Timer-based (ticks) rather than vector-based physics.
    *   **Interaction**: direct entity-to-entity (e.g., Fleet mining Asteroid) without spatial collision checks.
3.  **Systems**: Layer 2 systems run in parallel with Layer 1 but are computationally cheaper (O(N) on low entity counts).

## Consequences

### Positive
*   **Clarity**: acknowledges the reality of the codebase. `src/layer2` is a first-class citizen again.
*   **Gameplay Depth**: Allows for complex mechanics like "Kessler Syndrome" (Debris cascades) and "Trade Blockades" without needing a full physics engine.
*   **Performance**: Maintains high performance by avoiding the N^2 interactions or pathfinding of Layer 1.

### Negative
*   **Cognitive Load**: Developers must switch mental models between "Grid Sim" (L1) and "Abstract Sim" (L2).
*   **Synchronization**: Transferring resources between L1 (Colony Stockpiles) and L2 (Fleet Cargo) requires careful handling to prevent duplication or loss (The "Bridge" logic).
