# 214: Customs Checkpoint

## Overview

Implements a border control system to vet new arrivals (Visitors, Migrants, Refugees). Newly spawned entities from off-world sources start with an `ImmigrationStatus::Pending` component. They are flagged to be "intercepted" and must proceed to a designated `CustomsZone` before they are allowed to wander the colony or join as citizens.

A new job, `CustomsOfficer`, is responsible for processing these pending entities. The vetting process takes time and can reveal hidden traits (e.g., `Smuggler`, `Infected`, `Spy`) or simply grant entry (`Vetted`).

This system adds tension between security (vetting everyone takes time, bottlenecks trade/migration) and efficiency (open borders are fast but risky).

## Dependencies

- `003` Pop Entity (for basic entity structure)
- `056` Designated Zones (for `CustomsZone`)
- `009` Job System (for `CustomsOfficer` assignment)
- `074` Visitor System (for `Visitor` entities)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/customs_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::customs::{ImmigrationStatus, CustomsZone, VettingResult, CustomsOfficer};
    use crate::layer1::zone::{ZoneType, ZoneGrid};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::{Pop, Name};

    #[test]
    fn test_new_visitor_is_pending() {
        // Arrange
        let mut world = World::new();
        // Simulate spawning a visitor (usually done by visitor system, but we test the component default)
        let entity = world.spawn((
            Pop::default(),
            ImmigrationStatus::default(),
        )).id();

        // Assert
        let status = world.get::<ImmigrationStatus>(entity).unwrap();
        assert_eq!(*status, ImmigrationStatus::Pending);
    }

    #[test]
    fn test_interception_logic() {
        // Arrange: Entity is Pending and not in Customs Zone
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Customs);
        world.insert_resource(zone_grid);

        let entity = world.spawn((
            Pop::default(),
            ImmigrationStatus::Pending,
            GridPosition { x: 0, y: 0 },
            // Need a Navigation component or similar to track target
            crate::layer1::navigation::NavigationTarget::default(),
        )).id();

        // Act: Run interception system
        crate::layer1::customs::immigration_interception_system(&mut world);

        // Assert: Target should be set to Customs Zone (5, 5)
        let target = world.get::<crate::layer1::navigation::NavigationTarget>(entity).unwrap();
        assert_eq!(target.destination, GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_vetting_process_success() {
        // Arrange: Officer and Pending entity in Customs Zone
        let mut world = World::new();
        let pending_pop = world.spawn((
            Pop::default(),
            ImmigrationStatus::Pending,
            GridPosition { x: 5, y: 5 },
        )).id();

        let officer = world.spawn((
            Pop::default(),
            CustomsOfficer::default(), // Marker for job
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act: Run vetting system (simulate 1 tick of work)
        // In real impl, this would likely be an ActionType::VetPop
        crate::layer1::customs::vetting_work_system(&mut world);

        // Assert: Status changed to Vetted (assuming instant success for simple test)
        let status = world.get::<ImmigrationStatus>(pending_pop).unwrap();
        assert_eq!(*status, ImmigrationStatus::Vetted);
    }

    #[test]
    fn test_vetting_reveals_traits() {
        // Arrange: Pending pop with HiddenTrait
        let mut world = World::new();
        let pending_pop = world.spawn((
            Pop::default(),
            ImmigrationStatus::Pending,
            crate::layer1::customs::HiddenTraits(vec!["Smuggler".to_string()]),
            GridPosition { x: 5, y: 5 },
        )).id();

        // Act: Vet
        crate::layer1::customs::vetting_work_system(&mut world);

        // Assert: HiddenTrait removed/converted to real Trait, Status is Rejected (if logic dictates)
        // For MVP, just check status is Rejected due to Smuggler
        let status = world.get::<ImmigrationStatus>(pending_pop).unwrap();
        assert!(matches!(*status, ImmigrationStatus::Rejected(_)));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

```rust
// src/layer1/customs.rs

use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum ImmigrationStatus {
    #[default]
    Pending,
    Processing(u32), // Progress %
    Vetted,
    Rejected(String), // Reason
    Detained,
}

#[derive(Component, Debug, Clone, Default)]
pub struct HiddenTraits(pub Vec<String>);

#[derive(Component, Default)]
pub struct CustomsOfficer;
```

### 2. Update ZoneType

Modify `src/layer1/zone.rs` to include `Customs`.

### 3. Implement Interception System

```rust
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::map::GridPosition;
use crate::layer1::navigation::NavigationTarget;

pub fn immigration_interception_system(
    zone_grid: Res<ZoneGrid>,
    mut query: Query<(&mut NavigationTarget, &GridPosition, &ImmigrationStatus), Changed<ImmigrationStatus>>,
) {
    // Find closest Customs Zone
    let customs_locs = find_zone_locations(&zone_grid, ZoneType::Customs);
    if customs_locs.is_empty() { return; } // No customs, open borders? Or blocked?

    for (mut nav, pos, status) in query.iter_mut() {
        if *status == ImmigrationStatus::Pending {
            // Simple logic: go to first customs tile
            // Real logic: find closest
            nav.destination = customs_locs[0];
        }
    }
}

fn find_zone_locations(grid: &ZoneGrid, target: ZoneType) -> Vec<GridPosition> {
    // Scan grid...
    vec![]
}
```

### 4. Implement Vetting System (Simplified)

```rust
pub fn vetting_work_system(mut commands: Commands, mut query: Query<(Entity, &mut ImmigrationStatus, Option<&HiddenTraits>)>) {
    // This system would actually be driven by the Job System (ActionType::Vet),
    // but for the GREEN phase minimal implementation, we can simulate auto-vetting
    // if an officer is nearby, or just strictly test the status transition.

    for (entity, mut status, hidden) in query.iter_mut() {
        if let ImmigrationStatus::Processing(progress) = *status {
            if progress >= 100 {
                if let Some(hidden) = hidden {
                    if hidden.0.contains(&"Smuggler".to_string()) {
                        *status = ImmigrationStatus::Rejected("Smuggler".to_string());
                    } else {
                        *status = ImmigrationStatus::Vetted;
                    }
                } else {
                    *status = ImmigrationStatus::Vetted;
                }
            } else {
                *status = ImmigrationStatus::Processing(progress + 10);
            }
        } else if *status == ImmigrationStatus::Pending {
             *status = ImmigrationStatus::Processing(0);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Job Integration**: The `CustomsOfficer` should be a real Job Assignment. The "Work" action should drive the `Processing(u32)` progress bar.
- **UI**: Add an inspector view to see `ImmigrationStatus` and "Papers" (Lore flavor).
- **Queuing**: If many migrants arrive, they need to queue up. Use `Wait spots` or `Chair` furniture in the Customs Zone.
- **Detention**: If `Rejected`, they should pathfind to `ZoneType::Jail` or the map edge (leave).

## Acceptance Criteria

- [ ] `ImmigrationStatus` component exists.
- [ ] New `Visitor` entities default to `Pending`.
- [ ] `Customs` zone type added.
- [ ] Entities with `Pending` status pathfind to `Customs` zone.
- [ ] Vetting logic converts `Pending` -> `Vetted` or `Rejected`.
- [ ] `HiddenTraits` are detected during vetting.
- [ ] Tests pass.

## Technical Guidance

- Use `crate::layer1::zone::ZoneType` for the enum variant.
- Ensure the interception system doesn't override manual player commands if we allow manual control of visitors (unlikely, but good to note).
- Use `NavigationTarget` override to force movement.
- `HiddenTraits` should be removed and applied as real `Traits` upon successful detection (or kept if undetected!).

## Questions

- *Builder*: Should `Rejected` pops turn hostile?
    - *Architect*: For MVP, no. They should just leave (path to edge). Hostility is a future "Smuggler Turn" feature.
