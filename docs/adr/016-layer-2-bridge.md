# 16. Layer 2 Bridge Strategy

Date: 2024-05-22

## Status

Accepted (Supersedes ADR 003 in part)

## Context

ADR 003 ("Excision of Layers 2 and 3") explicitly removed the multi-planetary simulation scope to focus on a single colony (Layer 1). However, the narrative and mechanical endgame of SCALE involves launching fleets into orbit and interacting with the system map.

We need a way to represent off-world entities (Fleets, Asteroids) without simulating a full second layer of gameplay or physics.

## Decision

We will reintroduce Layer 2 elements as a **Data Layer**, not a Simulation Layer.

1.  **The Bridge**: The `LaunchPad` building (Layer 1) acts as the interface. It consumes Layer 1 resources (Fuel, Cargo) to spawn Layer 2 entities.
2.  **Data-Only Simulation**: Layer 2 entities (Fleets) exist as Bevy entities with components like `InOrbit` and `FleetCargo`. They are updated by lightweight systems (e.g., travel timers), but do not have complex physics, pathfinding, or tile grids like Layer 1.
3.  **Scope Limit**: We are *not* simulating surface maps of other planets yet. Interaction is abstract (Menu-based or Timer-based).

## Consequences

### Positive
*   **Endgame Viability**: Allows the "Escape" or "Expansion" fantasy to be fulfilled.
*   **Performance**: Keeping Layer 2 abstract avoids the cost of simulating multiple tile grids.
*   **Separation**: Clear distinction between "Colony Sim" (Layer 1) and "Fleet Management" (Layer 2).

### Negative
*   **Data Complexity**: Introduces new components (`Fleet`, `Orbit`) that must be persisted and managed.
*   **UI Challenge**: Requires visualizing abstract data (Orbit) alongside concrete data (Colony Map).
