# 1322: Harmonic Mining

## 1. Overview
**Layer:** 1

**Fantasy:** Mining without pickaxes. Singing the stone to dust.

**Mechanic:** "Sonic Drills" disintegrate ore instantly in a radius. However, they vibrate at specific frequencies. If the frequency matches other materials (e.g., Glass, Crystal, Bone), those shatter too.

**Emergence:** You tune the drill to mine "Iron". It works great. But your "Glass Greenhouses" vibrate and shatter, venting your crops to vacuum.

**Tension:** Fast, area-of-effect mining vs. Collateral damage risk.

## 2. Dependencies
- `011` — Core Resource system (for base resources like Iron, Glass, etc)
- `004` — Buildings (for Greenhouses, Walls, etc that can be shattered)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale_layer1::resources::{Material, ResourceType};
    use scale_layer1::map::GridPosition;
    use scale_layer1::buildings::Building;
    use scale_layer1::mining::{SonicDrill, Frequency, TriggerSonicDrillEvent, harmonic_mining_system};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems/resources if needed
        world.insert_resource(Events::<TriggerSonicDrillEvent>::default());
        world
    }

    #[test]
    fn test_drill_destroys_matching_ore() {
        let mut world = setup_world();

        let ore_entity = world.spawn((
            GridPosition { x: 5, y: 5 },
            Material { resource: ResourceType::Iron, health: 100 },
        )).id();

        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Iron,
        });

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(harmonic_mining_system);
        schedule.run(&mut world);

        // The iron ore should be destroyed (health reduced to 0 or entity despawned)
        assert!(world.get_entity(ore_entity).is_none() || world.get::<Material>(ore_entity).unwrap().health == 0);
    }

    #[test]
    fn test_drill_shatters_collateral_buildings() {
        let mut world = setup_world();

        // Spawn a greenhouse nearby
        let greenhouse = world.spawn((
            GridPosition { x: 6, y: 5 },
            Building { health: 50 },
            Material { resource: ResourceType::Glass, health: 50 },
        )).id();

        // The sonic drill is tuned to the frequency of Glass
        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Glass,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(harmonic_mining_system);
        schedule.run(&mut world);

        // The greenhouse should be destroyed or severely damaged
        assert!(world.get_entity(greenhouse).is_none() || world.get::<Building>(greenhouse).unwrap().health == 0);
    }

    #[test]
    fn test_drill_ignores_non_matching_materials() {
        let mut world = setup_world();

        // Spawn a steel wall nearby
        let steel_wall = world.spawn((
            GridPosition { x: 6, y: 5 },
            Building { health: 200 },
            Material { resource: ResourceType::Steel, health: 200 },
        )).id();

        // The sonic drill is tuned to Iron
        world.send_event(TriggerSonicDrillEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            frequency: Frequency::Iron,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(harmonic_mining_system);
        schedule.run(&mut world);

        // The steel wall should be completely unaffected
        assert_eq!(world.get::<Building>(steel_wall).unwrap().health, 200);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// scale_layer1/src/mining/mod.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{Material, ResourceType};
use crate::layer1::buildings::Building;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    Iron,
    Glass,
    // Add other frequencies as needed
}

impl Frequency {
    fn matches(&self, resource: &ResourceType) -> bool {
        match (self, resource) {
            (Frequency::Iron, ResourceType::Iron) => true,
            (Frequency::Glass, ResourceType::Glass) => true,
            _ => false,
        }
    }
}

#[derive(Event)]
pub struct TriggerSonicDrillEvent {
    pub center: GridPosition,
    pub radius: i32,
    pub frequency: Frequency,
}

pub fn harmonic_mining_system(
    mut commands: Commands,
    mut events: EventReader<TriggerSonicDrillEvent>,
    mut target_query: Query<(Entity, &GridPosition, &Material, Option<&mut Building>)>,
) {
    for event in events.read() {
        for (entity, pos, material, mut opt_building) in target_query.iter_mut() {
            // Simple distance check
            let dx = (pos.x - event.center.x).abs();
            let dy = (pos.y - event.center.y).abs();
            let distance = std::cmp::max(dx, dy); // Chebyshev distance

            if distance <= event.radius {
                if event.frequency.matches(&material.resource) {
                    if let Some(ref mut building) = opt_building {
                        building.health = 0;
                    } else {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactoring Opportunities**:
  - Instead of instantaneous destruction, we might want to apply `HarmonicStress` components to entities, which deplete health over time, allowing for a "shut off the drill!" moment.
- **Code Smells**:
  - The Chebyshev distance check is basic. Consider a Euclidean check using float math `((dx*dx + dy*dy) as f32).sqrt() <= radius as f32`.
- **Performance Considerations**:
  - Iterating over all entities with a `Material` in the entire world will scale poorly. Use a spatial partition/grid lookup query to only check entities within the `radius` of the `center`.
- **API Improvements**:
  - The `Frequency` enum might need to be abstracted, or mapped directly to `ResourceType` to avoid maintaining parallel enums.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for the new logic.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] A `SonicDrill` event successfully destroys ore matching the frequency.
- [ ] A `SonicDrill` event shatters buildings that share the matching frequency/material.
- [ ] Unrelated materials are ignored completely.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- **Location:** Implement in `src/layer1/mining/harmonic_mining.rs` or similar.
- **Hooking Up Systems:** Add the `harmonic_mining_system` to the appropriate layer 1 update schedule, and ensure the event is registered via `app.add_event::<TriggerSonicDrillEvent>()`.
- **Gotchas:** When a building's health reaches 0, ensure it actually gets cleaned up by the standard `BuildingDestroyed` systems. You may need to trigger a `BuildingDamaged` event rather than setting health to 0 manually, depending on existing building logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
