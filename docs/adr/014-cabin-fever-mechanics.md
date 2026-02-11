# 14. Cabin Fever Mechanics

Date: 2024-05-24

## Status

Accepted

## Context

As the simulation complexity grew, we noticed that Pops were perfectly content living in cramped, windowless underground cells indefinitely. This lacked realism and removed a key tension found in colony sims: the psychological toll of the environment.

We needed a system to:
1.  Penalize "dwarf style" optimized housing (1x1 cells) without completely banning it.
2.  Encourage players to build outdoor areas or invest in aesthetics (Beauty).
3.  Create emergent failure states where confinement leads to social unrest.

## Decision

We introduced the **Cabin Fever System**, centered around a new `CabinFever` component.

1.  **Component Structure:**
    *   `CabinFever` tracks two axes of stress: `confinement` (lack of nature/sky) and `crowding` (proximity to others).

2.  **Simulation Logic (`update_cabin_fever_system`):**
    *   **Confinement:** Increases if a Pop is under a roof (`RoofGrid`). Decreases if outdoors.
    *   **Mitigation:** Confinement gain is blocked if the local tile has high beauty (`BeautyGrid > 10.0`), simulating "nice interiors".
    *   **Crowding:** Checks a 3x3 area around the Pop. If more than 3 neighbors are present, crowding stress increases.

3.  **Consequences (System Linkage):**
    *   **Morale:** High fever reduces `Needs.leisure`, making Pops harder to keep happy.
    *   **Mental Breaks:** High aggregate stress pushes Pops toward mental breaks (implemented in `unrest.rs`), leading to behaviors like Vandalism or Binging.

## Consequences

### Positive
*   **Architectural Diversity:** Players must now design spacious rooms or outdoor courtyards.
*   **Emergent Storytelling:** "The Long Night" scenarios now naturally lead to social friction.
*   **System Synergy:** Makes the `BeautyGrid` mechanically relevant beyond just a stat boost.

### Negative
*   **Performance Cost:** The crowding check involves iterating potential neighbors. While currently manageable, naive implementation approaches O(N^2) behavior if not optimized with spatial hashing in the future.
*   **Player Friction:** Players accustomed to efficiency-first builds may find the morale penalties frustrating initially.
