# 363 - Architectural Spite

## 1. Overview
**Layer:** 1
**Fantasy:** Seeing pops express their social grievances not through riots, but through petty, inefficient modifications to the colony's layout and buildings.
**Mechanic:** Pops with low social relationships to their neighbors or the administration will occasionally build "spite walls", alter pathfinding routing to block a neighbor's window, or sabotage specific amenity access points targeting individuals they hate.

## 2. Dependencies
- Building Placement System (`006-building-placement`)
- Pop Relationships (`047-pop-relationships`)
- Spontaneous Architecture (`110-spontaneous-architecture`)

## 3. RED Phase: Tests First

```rust
// tests/layer1/construction/spite_architecture_tests.rs

#[test]
fn test_pop_builds_spite_wall_to_block_rival() {
    // Arrange: Create Pop A and Pop B with deeply negative relationship (rivalry). Pop B's house has a window/door.
    // Act: Advance simulation enough for Pop A to trigger spontaneous architecture.
    // Assert: Pop A constructs a `SpiteWall` adjacent to Pop B's house, blocking the entrance or window.
}

#[test]
fn test_spite_structures_reduce_efficiency_and_cause_pathfinding_detours() {
    // Arrange: Place a SpiteWall in a heavily trafficked corridor.
    // Act: Run pathfinding for hauling jobs.
    // Assert: Pathfinding algorithm correctly identifies the SpiteWall as an obstacle and routes around it, increasing travel time.
}

#[test]
fn test_tearing_down_spite_wall_angers_builder() {
    // Arrange: SpiteWall built by Pop A.
    // Act: Player issues a Demolish designation on the SpiteWall. Pop C demolishes it.
    // Assert: Pop A receives a significant negative mood modifier ("My Work Destroyed").
}

#[test]
fn test_spite_amenity_sabotage_restricts_access() {
    // Arrange: Pop A sabotages an amenity (e.g., Tavern seat) specifically targeting Pop B.
    // Act: Pop B attempts to use the amenity.
    // Assert: Pop B is denied access and incurs a minor stress penalty.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/construction/spite.rs

use bevy_ecs::prelude::*;
// Implement SpiteWall component, AI logic to target rivals, and demolition mood effects.
```

## 5. REFACTOR Phase: Quality & Design
- Extend the `SpontaneousArchitecture` AI module to check for `Relationship` components to prioritize spite building targets.
- Ensure `SpiteWall` inherits from standard obstacle properties so that the existing pathfinding (`A*` in `src/layer1/pathfinding.rs`) automatically routes around it.
- Store the entity ID of the builder on the `SpiteWall` component so `DemolishEvent` can correctly trigger the mood penalty.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] Test coverage ≥85% for `spite.rs`.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Pops correctly identify rivals' spaces and build obstacles adjacent to them.
- [ ] Pathfinding naturally updates to treat spite walls as obstacles.

## 7. Technical Guidance
- `SpiteWall` shouldn't cost the player's colony resources; it uses "junk" or personal resources similar to `SpontaneousArchitecture`.
- Add a specific AI Action `ActionType::BuildSpite` that scores highly when a Pop is idle and has a rival nearby.
- Ensure that spite walls cannot completely seal off a crucial base component (e.g., the only path to the command center). Implement a "valid path" check before finalizing the spite wall construction.

## 8. Questions
*Builder: add questions here if spec is unclear.*
