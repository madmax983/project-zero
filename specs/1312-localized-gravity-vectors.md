# 1312: Localized Gravity Vectors

## Overview

"Gravity Plates" define "Down" for adjacent tiles allowing building rooms on walls or ceilings. Pops transition orientation when walking over curved plates. It provides high density options but failures can cause 500 people to fall into parked starships, bringing a tension between Density vs. Complexity.

## Dependencies

- `008` — Map Grid
- `012` — Pathfinding

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_gravity_plate_defines_down() {
        let mut app = App::new();
        // Setup systems

        // Act: place gravity plate with UP orientation
        let plate = app.world.spawn((GravityPlate { down_vector: Vec3::Y })).id();
        let tile = app.world.spawn((GridPosition { x: 0, y: 1 }, MapTile)).id();

        // Assert: tile 'down' matches plate orientation
    }

    #[test]
    fn test_pop_transitions_orientation() {
        let mut app = App::new();
        // Setup systems

        let pop = app.world.spawn((Pop, GridPosition { x: 0, y: 0 }, Orientation { up: Vec3::Y })).id();

        // Act: pop walks over a curved plate that shifts 'down' to X

        // Assert: Pop's Orientation changes smoothly
    }

    #[test]
    fn test_gravity_plate_failure_causes_falling() {
        let mut app = App::new();
        // Setup systems

        let pop = app.world.spawn((Pop, GridPosition { x: 5, y: 10 }, Orientation { up: Vec3::NEG_Y })).id();
        let plate = app.world.spawn((GravityPlate { down_vector: Vec3::Y, powered: false })).id();

        // Act: plate loses power

        // Assert: pop falls down towards Vec3::NEG_Y (global down)
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct GravityPlate {
    pub down_vector: Vec3,
    pub powered: bool,
}

#[derive(Component)]
pub struct Orientation {
    pub up: Vec3,
}

pub fn update_tile_gravity(
    // Query for GravityPlates
    // Query for MapTiles
    // Update MapTile down_vector based on nearest powered GravityPlate
) {
    // Implementation
}

pub fn update_pop_orientation(
    // Query for Pops with GridPosition and Orientation
    // Query for MapTiles to get local gravity
    // Update Pop's Orientation to align with tile's gravity
) {
    // Implementation
}

pub fn apply_falling_mechanics(
    // Query for Pops
    // If pop is not supported by floor relative to their local/global gravity
    // Apply falling translation
) {
    // Implementation
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Caching the gravity vector on MapTiles to avoid recalculating per Pop per frame.
- **Integration**: Pathfinding needs to handle navigating through varying gravity orientations.
- **Design**: Visual indication for gravity transitions and power failures.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Map tiles inherit gravity from plates.
- [ ] Pops adjust orientation based on local gravity.
- [ ] Pops fall globally when gravity plates fail.

## Technical Guidance

- Modifying the pathfinding A* algorithm to consider 'up' orientation as part of the state might be necessary to avoid impossible paths.
- Ensure the falling mechanic integrates with the Health system (injury upon impact).

## Questions

*Builder: add questions here if spec is unclear.*
