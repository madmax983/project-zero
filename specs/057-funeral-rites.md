# 057: Funeral Rites

## Overview

Death is not the end of the story. When a colonist dies, they leave behind a **Corpse**. Unburied corpses cause **Grief** and **Haunted** moods in survivors. Players must build **Graves** and perform **Funeral** rites to provide closure.

This feature adds emotional weight to death and a sanitation requirement.

## Dependencies

- `034` — Pop Health (Death system modification)
- `031` — Pop Morale (Grief/Closure effects)
- `036` — Pop Memory (MemoryType)
- `006` — Building Placement (Grave)
- `016` — Utility AI (Funeral Action)

## RED Phase: Tests First

Write these tests in `src/layer1/funeral_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::health::{Health, death_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::funeral::{Corpse, Grave, funeral_service_system};
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::actions::{ActionType, AssignmentType};

    #[test]
    fn test_death_spawns_corpse() {
        let mut world = World::new();
        // Setup time/log resources needed by death_system
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Spawn a dying pop
        let pop_pos = GridPosition { x: 5, y: 5 };
        let pop = world.spawn((
            Health { current: -10.0, max: 100.0 }, // Dead
            pop_pos,
            crate::layer1::pop::Pop,
            crate::layer1::pop::Name("John Doe".to_string()),
        )).id();

        // Run death system
        death_system(&mut world);

        // Pop should be despawned
        assert!(world.get_entity(pop).is_err());

        // Corpse should exist at same position
        let mut corpses = world.query::<(&Corpse, &GridPosition)>();
        let (corpse, pos) = corpses.single(&world);

        assert_eq!(pos, &pop_pos);
        assert_eq!(corpse.name, "John Doe");
        assert!(corpse.decay > 0.0);
    }

    #[test]
    fn test_grave_component_defaults() {
        let grave = Grave::default();
        assert!(grave.occupied_by.is_none());
    }

    #[test]
    fn test_funeral_service_buries_corpse() {
        let mut world = World::new();

        // Spawn Corpse
        let corpse = world.spawn((
            Corpse { name: "Jane".to_string(), decay: 100.0 },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Spawn Grave
        let grave = world.spawn((
            Building { building_type: BuildingType::Grave },
            Grave::default(),
            GridPosition { x: 10, y: 10 }
        )).id();

        // Spawn Undertaker (Pop performing funeral)
        let undertaker = world.spawn((
            crate::layer1::pop::Pop,
            crate::layer1::actions::PopAction {
                current: ActionType::Funeral,
                ..Default::default()
            },
            crate::layer1::actions::AssignedTo {
                entity: grave,
                assignment_type: AssignmentType::Undertaker,
            },
            // Start carrying corpse (simulate haul complete)
            crate::layer1::hauling::Carrying {
                item: corpse,
            }
        )).id();

        // Run funeral system
        funeral_service_system(&mut world);

        // Corpse entity should be despawned (or hidden/attached to grave)
        // For MVP: Despawn corpse entity, mark grave occupied.
        assert!(world.get_entity(corpse).is_err());

        let grave_comp = world.get::<Grave>(grave).unwrap();
        assert_eq!(grave_comp.occupied_by, Some("Jane".to_string()));
    }

    #[test]
    fn test_burial_grants_closure() {
        let mut world = World::new();
        let time = crate::shared::time::SimulationTime { tick: 100 };
        world.insert_resource(time);

        // Mourner with Grief
        let mourner = world.spawn(Memories::default()).id();
        world.get_mut::<Memories>(mourner).unwrap().add(MemoryType::Grief, 50);

        // Trigger burial event (could be done via event or direct system call)
        // Here we test the helper function `apply_closure`
        crate::layer1::funeral::apply_closure(&mut world, "Jane");

        // Check memories
        let memories = world.get::<Memories>(mourner).unwrap();
        // Grief should be removed or suppressed?
        // Spec: Adds "Closure" memory.
        assert!(memories.items.iter().any(|m| matches!(m.mtype, MemoryType::Closure)));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/funeral.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::memory::{Memories, MemoryType};

#[derive(Component)]
pub struct Corpse {
    pub name: String,
    pub decay: f32, // 100.0 -> 0.0
}

#[derive(Component, Default)]
pub struct Grave {
    pub occupied_by: Option<String>,
}

pub fn funeral_service_system(
    mut commands: Commands,
    mut undertakers: Query<(Entity, &crate::layer1::hauling::Carrying, &crate::layer1::actions::AssignedTo)>,
    mut graves: Query<&mut Grave>,
    corpses: Query<&Corpse>,
) {
    for (worker, carrying, assignment) in undertakers.iter_mut() {
        if assignment.assignment_type == crate::layer1::actions::AssignmentType::Undertaker {
            // Assume worker has arrived at grave with corpse
            // (In reality, we need check distance/state, but for MVP/Test assume completion)

            if let Ok(corpse) = corpses.get(carrying.item) {
                if let Ok(mut grave) = graves.get_mut(assignment.entity) {
                    // Bury
                    grave.occupied_by = Some(corpse.name.clone());

                    // Despawn corpse
                    commands.entity(carrying.item).despawn_recursive();

                    // Remove carrying component
                    commands.entity(worker).remove::<crate::layer1::hauling::Carrying>();

                    // Apply closure (global for now, or to friends)
                    // We can't access World here easily to iterate all pops for memories.
                    // Send event? Or use a separate system?
                }
            }
        }
    }
}

pub fn apply_closure(world: &mut World, _deceased_name: &str) {
    let tick = world.resource::<crate::shared::time::SimulationTime>().tick;

    // Add Closure memory to all pops (simplification)
    // Real implementation: Filter by relationship.
    let mut query = world.query::<&mut Memories>();
    for mut memories in query.iter_mut(world) {
        memories.add(MemoryType::Closure, tick);
    }
}
```

### 2. Update `death_system` in `src/layer1/health.rs`

Modify `death_system` to spawn `Corpse` before despawning.

```rust
// ...
if let Ok((pos, name)) = world.query::<(&GridPosition, &Name)>().get(world, entity) {
    world.spawn((
        Corpse {
            name: name.0.clone(),
            decay: 100.0,
        },
        *pos,
        // Add item tag so it can be hauled?
        // ResourceItem? Or special "Haulable" tag?
        // Corpse is a special item.
    ));
}
world.despawn(entity);
// ...
```

### 3. Update Enums

- **BuildingType**: Add `Grave`.
- **ActionType**: Add `Funeral`.
- **AssignmentType**: Add `Undertaker`.
- **MemoryType**: Add `Grief`, `Closure`, `Haunted`.

## REFACTOR Phase: Quality & Design

- **Decay**: Corpses rot (Health hazard). Unburied corpses spawn "Rot" or "Vermin".
- **Grave Types**: Sarcophagus (Stone) vs Simple Grave (Dirt).
- **Ceremony**: Needs a priest?
- **Relationship**: Only friends should get Grief/Closure.

## Acceptance Criteria

- [ ] `Corpse` entity spawns on death.
- [ ] `Grave` building exists.
- [ ] `Undertaker` assignment buries the corpse.
- [ ] `Closure` memory is applied.
- [ ] Tests pass.

## Technical Guidance

- Ensure `Corpse` is haulable. Might need to implement `HaulTarget` trait or similar if the hauling system is generic.
- `funeral_service_system` might need to run *after* hauling arrives.
- Be careful with `world.despawn` in `death_system` while borrowing components; extract data first.

## Questions

*Builder: add questions here if spec is unclear.*
