# 1310: The Feral Administration

## Overview

Implementing "The Feral Administration" mechanic. If administrative buildings are understaffed or lose power, they begin spawning physical "Unprocessed Forms" on adjacent tiles. These forms stack rapidly and eventually create impassable terrain that blocks pathfinding and buries nearby buildings, representing a physical avalanche of paperwork.

## Dependencies

- Existing Layer 1 Pop job mechanics (e.g. `Staffed` or `Worker` state).
- Existing Layer 1 Power/Energy systems.
- Existing Layer 1 Grid and Pathfinding systems to support impassable "Unprocessed Forms" entities.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::power::PowerStatus;
    use crate::layer1::staffing::StaffedStatus;

    #[test]
    fn test_unprocessed_forms_spawn_when_unstaffed_or_unpowered() {
        let mut app = App::new();
        app.add_systems(Update, spawn_unprocessed_forms_system);

        let admin_pos = GridPosition { x: 5, y: 5 };

        let building = app.world_mut().spawn((
            Colony,
            AdministrativeBuilding,
            PowerStatus { is_powered: false },
            StaffedStatus { is_staffed: false },
            admin_pos,
        )).id();

        // Act
        app.update();

        // Assert: Unprocessed Forms should be spawned around the building
        let mut found_forms = false;
        for (_, pos) in app.world_mut().query::<(&UnprocessedForms, &GridPosition)>().iter(app.world()) {
            // Must spawn adjacently
            let dist = (pos.x - admin_pos.x).abs() + (pos.y - admin_pos.y).abs();
            if dist == 1 {
                found_forms = true;
                break;
            }
        }
        assert!(found_forms);
    }

    #[test]
    fn test_unprocessed_forms_block_pathfinding() {
        let mut app = App::new();
        app.add_systems(Update, process_impassable_terrain_system);

        let pos = GridPosition { x: 10, y: 10 };
        let forms = app.world_mut().spawn((
            UnprocessedForms { stack_size: 10 },
            pos,
        )).id();

        // Act
        app.update();

        // Assert: The tile is now marked as impassable
        let grid = app.world().resource::<TerrainGrid>();
        assert!(!grid.is_passable(&pos));
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::{GridPosition, TerrainGrid};

#[derive(Component)]
pub struct AdministrativeBuilding;

#[derive(Component)]
pub struct PowerStatus {
    pub is_powered: bool,
}

#[derive(Component)]
pub struct StaffedStatus {
    pub is_staffed: bool,
}

#[derive(Component)]
pub struct UnprocessedForms {
    pub stack_size: i32,
}

pub fn spawn_unprocessed_forms_system(
    mut commands: Commands,
    query: Query<(&GridPosition, &PowerStatus, &StaffedStatus), With<AdministrativeBuilding>>,
) {
    for (pos, power, staffed) in query.iter() {
        if !power.is_powered || !staffed.is_staffed {
            // Spawn an unprocessed form randomly adjacent
            // Note: simplified to always spawn at x+1, y for minimal implementation
            commands.spawn((
                UnprocessedForms { stack_size: 1 },
                GridPosition { x: pos.x + 1, y: pos.y },
            ));
        }
    }
}

pub fn process_impassable_terrain_system(
    query: Query<(&GridPosition, &UnprocessedForms)>,
    mut grid: ResMut<TerrainGrid>,
) {
    for (pos, forms) in query.iter() {
        if forms.stack_size >= 10 {
            // Mark the terrain as impassable in the grid
            grid.set_passable(pos, false);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Adjacent Tile Spawning**: Update `spawn_unprocessed_forms_system` to randomly select a valid, unoccupied adjacent tile to spawn forms in. If the stack is existing on the tile, increment `stack_size`.
- **Performance**: Track form stacking directly via the `TerrainGrid` or a dedicated `ClutterGrid` rather than spawning thousands of individual entities if there are massive outbreaks.
- **Clearing mechanism**: We will need a way to clear these forms via jobs (e.g. `Excavate`) later. For now, ensure `UnprocessedForms` can be cleanly deleted and `set_passable(pos, true)` triggered.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Understaffed / unpowered administrative buildings spawn `UnprocessedForms` entities on adjacent tiles.
- [ ] When `UnprocessedForms` reach a threshold size, they block pathfinding.

## Technical Guidance

- Administrative buildings are typically identified by specific components, adapt `AdministrativeBuilding` to match existing markers if available.
- Bevy's spatial querying or the existing `TerrainGrid` component should be used carefully so we don't accidentally override non-form blockers (like walls). Ensure `is_passable` handles layer combinations correctly.

## Questions
*Builder: add questions here if spec is unclear.*
