# 20. Spontaneous Architecture

Date: 2024-06-05

## Status

Accepted

## Context

In many colony simulations, the world is static unless the player explicitly orders a change. This leads to a "sterile" feeling where the inhabitants are merely robots executing a master plan. We wanted to align with our design principle of "Emergence over Scripting" and make the colony feel "lived in."

Specifically, we wanted Pops to express agency by modifying their environment to suit their own needs, independent of the player's direct control.

## Decision

We implemented the **Spontaneous Architecture** system (`src/layer1/spontaneous_architecture.rs`).

1.  **Autonomous Construction:** Idle Pops have a probability (checked in `check_spontaneous_build_system`) to construct **Personal Structures** on valid, empty tiles adjacent to their assigned Housing.
2.  **Structure Types:**
    *   **Garden:** Provides Beauty.
    *   **Shrine:** Provides Spiritual/Morale benefits.
    *   **Shed:** (Cosmetic/Storage role).
3.  **Ownership Model:** The structure is linked to the Pop via an `OwnsStructure` component.
4.  **Emotional Attachment:** If the player (or disaster) demolishes a personal structure, the owner suffers a significant Morale penalty ("Personal Structure Demolished").

## Consequences

### Positive
*   **Organic Growth:** The colony map evolves over time. "Suburbs" naturally become greener or more cluttered based on the inhabitants.
*   **Narrative Generation:** A Pop building a shrine after a friend dies (if combined with other systems) or a garden in spring creates micro-stories.
*   **Visual Variety:** Breaks up the grid with non-uniform, non-optimized placements.

### Negative
*   **Player Friction:** Pops may build in locations the player intended to use for something else (e.g., a road), forcing the player to choose between efficiency and morale.
*   **Resource Drain:** Spontaneous building consumes colony resources (Wood/Stone) without explicit player approval, which could be dangerous in scarcity scenarios.
