# 1130: Volatile Intermediates

## 1. Overview

Advanced crafting requires "Unstable" items (e.g., Antimatter Containment Cells, Plasma Gel) that degrade rapidly into "Explosion" or "Hazard" tiles if not used in a recipe within a given timeframe. This requires "Just-In-Time" logistics.

The danger is in the process, not just the enemy. A hauler carrying a Plasma Gel canister getting stuck in a crowded hallway could cause the canister to destabilize, venting the entire corridor. This introduces a tension between stockpiling for efficiency versus on-demand production for safety.

## 2. Dependencies

- `022` — Resource Stockpiles
- `049` — Industrial Waste

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::inventory::Item;
    use crate::layer1::map::GridPosition;
    use super::*;

    #[test]
    fn test_volatile_item_degrades_over_time() {
        // Arrange
        let mut world = World::new();

        let volatile_entity = world.spawn((
            Item::new("Plasma Gel"),
            Volatile { current_stability: 100.0, max_stability: 100.0, degradation_rate: 10.0 },
        )).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(volatile_degradation_system);

        // Simulate a tick representing 1.0 second elapsed
        // (Assuming a DeltaTime resource or similar is present and set to 1.0)
        schedule.run(&mut world);

        // Assert
        let volatile_comp = world.get::<Volatile>(volatile_entity).unwrap();
        assert!(volatile_comp.current_stability < 100.0, "Stability should decrease over time");
        assert_eq!(volatile_comp.current_stability, 90.0);
    }

    #[test]
    fn test_volatile_item_explodes_at_zero_stability() {
        // Arrange
        let mut world = World::new();

        let volatile_entity = world.spawn((
            Item::new("Plasma Gel"),
            GridPosition { x: 10, y: 10 },
            Volatile { current_stability: 5.0, max_stability: 100.0, degradation_rate: 10.0 },
        )).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(volatile_degradation_system);
        schedule.run(&mut world);

        // Assert
        // The original item should be despawned
        assert!(world.get_entity(volatile_entity).is_none(), "Exploded item should be despawned");

        // A hazard or waste should be spawned at the same location.
        // This test assumes an ExplosionEvent or similar hazard creation occurred.
        // We will assert by querying for hazard components at that position.
        let mut found_hazard = false;
        let mut query = world.query::<(&crate::layer1::waste::IndustrialWaste, &GridPosition)>();
        for (_, pos) in query.iter(&world) {
            if pos.x == 10 && pos.y == 10 {
                found_hazard = true;
                break;
            }
        }

        assert!(found_hazard, "Industrial Waste/Hazard should be spawned at explosion location");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::waste::IndustrialWaste;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Volatile {
    pub current_stability: f32,
    pub max_stability: f32,
    pub degradation_rate: f32, // Stability lost per second
}

// We assume a resource `DeltaTime` or use fixed tick time
pub fn volatile_degradation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Volatile, Option<&GridPosition>)>,
) {
    let delta_time = 1.0; // Mocked for minimal implementation

    for (entity, mut volatile, grid_pos) in query.iter_mut() {
        volatile.current_stability -= volatile.degradation_rate * delta_time;

        if volatile.current_stability <= 0.0 {
            // Explode!
            commands.entity(entity).despawn_recursive();

            // Spawn hazard if it has a grid position
            if let Some(pos) = grid_pos {
                commands.spawn((
                    IndustrialWaste { amount: 10 }, // Just a basic mess
                    *pos,
                ));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Tie degradation to Bevy's actual `Time` resource instead of hardcoded or mock values.
- **Explosion Yield**: Scale the explosion damage and waste radius based on the `current_stack_size` or max_stability of the volatile item.
- **UI Integration**: Volatile items should visually flash or display a countdown bar so the player knows disaster is imminent.
- **Containment Units**: Introduce specific inventory types (e.g., "Stasis Hauler") that halt or slow down the degradation rate while the item is stored inside.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Volatile items correctly degrade over time when spawned in the world or in standard inventories.
- [ ] Reaching 0 stability despawns the item and generates `IndustrialWaste` at its location.

## 7. Technical Guidance

- Ensure `DespawnRecursive` is used so any attached child entities (UI elements, specific sub-components) are correctly cleaned up.
- Be careful with `GridPosition` references; items in a pop's inventory might not have a direct `GridPosition` but rather be a child of an entity with a `GridPosition`. You'll need to resolve the global position during explosion.
- Fire an `ExplosionEvent` (if one exists or will be added) to trigger sound effects and visuals.

## 8. Questions

*Builder: add questions here if spec is unclear.*
