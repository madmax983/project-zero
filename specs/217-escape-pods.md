# 217: Escape Pods

## Overview

Implement constructible **Escape Pods** that allow colonists (Pops) to evacuate the map during catastrophic failure (e.g., reactor meltdown, overrun by fauna). This converts a "Game Over" state into a "Partial Survival" state, where the saved Pops might return in future runs or Layer 2 mechanics (Distress Signals).

This feature requires:
1.  A new `BuildingType::EscapePod`.
2.  An evacuation mechanic where Pops enter the pod.
3.  A launch mechanic that removes the Pod and Pops from the map.

## Dependencies

- `004` — Pop Entity (Pops to save)
- `006` — Building Placement (To build the pods)
- `015` — Selection/Input (To trigger launch)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/escape_pod_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::escape_pod::{EscapePod, launch_pod_system, Evacuee};

    #[test]
    fn test_escape_pod_capacity() {
        let mut world = World::new();
        let pod = world.spawn((
            Building { building_type: BuildingType::EscapePod },
            EscapePod { capacity: 3, ..Default::default() },
        )).id();

        let pop1 = world.spawn(Pop).id();
        let pop2 = world.spawn(Pop).id();
        let pop3 = world.spawn(Pop).id();
        let pop4 = world.spawn(Pop).id();

        // Simulate assigning pops to pod (logic usually in a system, but helper here)
        assert!(crate::layer1::escape_pod::try_assign_pop(&mut world, pod, pop1));
        assert!(crate::layer1::escape_pod::try_assign_pop(&mut world, pod, pop2));
        assert!(crate::layer1::escape_pod::try_assign_pop(&mut world, pod, pop3));
        assert!(!crate::layer1::escape_pod::try_assign_pop(&mut world, pod, pop4), "Should fail when full");
    }

    #[test]
    fn test_launch_removes_pod_and_occupants() {
        let mut world = World::new();
        let pod = world.spawn((
            Building { building_type: BuildingType::EscapePod },
            EscapePod { capacity: 3, launched: false, ..Default::default() },
        )).id();

        let pop = world.spawn((
            Pop,
            Evacuee { pod_entity: pod }, // Component indicating they are inside
        )).id();

        // Mark for launch
        world.get_mut::<EscapePod>(pod).unwrap().launched = true;

        // Run launch system
        let mut schedule = Schedule::default();
        schedule.add_systems(launch_pod_system);
        schedule.run(&mut world);

        // Verify entities are despawned (or marked dead/safe)
        assert!(world.get_entity(pod).is_none(), "Pod should be despawned (launched)");
        assert!(world.get_entity(pop).is_none(), "Pop should be despawned (safe)");
    }

    #[test]
    fn test_launch_event_generation() {
        // Verify that launching generates a 'DistressSignal' or log event
        // (Implementation detail: check Events<LaunchEvent> or Chronicle)
        // let mut world = World::new();
        // ... setup world ...
        // ... assert event ...
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `EscapePod` Component

```rust
// src/layer1/escape_pod.rs
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct EscapePod {
    pub capacity: usize,
    pub launched: bool,
    pub occupants: Vec<Entity>,
}

#[derive(Component)]
pub struct Evacuee {
    pub pod_entity: Entity,
}

pub fn try_assign_pop(world: &mut World, pod_entity: Entity, pop_entity: Entity) -> bool {
    let mut pod = world.get_mut::<EscapePod>(pod_entity)?;
    if pod.occupants.len() < pod.capacity {
        pod.occupants.push(pop_entity);
        world.entity_mut(pop_entity).insert(Evacuee { pod_entity });
        return true;
    }
    false
}
```

### 2. `launch_pod_system`

```rust
// src/layer1/escape_pod.rs

pub fn launch_pod_system(
    mut commands: Commands,
    query: Query<(Entity, &EscapePod)>,
) {
    for (pod_entity, pod) in query.iter() {
        if pod.launched {
            // Log success (Chronicle event 217)
            // ...

            // Despawn occupants (Safely!)
            for &occupant in &pod.occupants {
                commands.entity(occupant).despawn_recursive();
            }

            // Despawn pod
            commands.entity(pod_entity).despawn_recursive();
        }
    }
}
```

### 3. Register `BuildingType::EscapePod`

Update `src/layer1/building.rs`:
- Add `EscapePod` variant.
- Set `char()` to something distinct (e.g. `'^'`).
- Set `cost()` (High metal/fuel).
- Add `EscapePod` component in `spawn_building`.

## REFACTOR Phase: Quality & Design

- **Animation**: Instead of instant despawn, spawn a `VisualEffect` (Rocket trail) moving upwards (z-index).
- **Survival Data**: Store the "Saved" pops in a `RunHistory` resource or file so they can appear in future games.
- **Panic AI**: Update `UtilityAI` to highly prioritize `EnterEscapePod` actions when `AlertLevel::Red` is active.
- **UI**: Add a "Launch" button to the Inspector panel when an Escape Pod is selected.

## Acceptance Criteria (Testable!)

- [ ] `EscapePod` component tracks capacity and occupants.
- [ ] `try_assign_pop` respects capacity limits.
- [ ] `launch_pod_system` removes the Pod and all Occupants from the World.
- [ ] `BuildingType::EscapePod` is buildable and has a cost.
- [ ] Tests pass.

## Technical Guidance

- **Despawn Recursive**: Use `despawn_recursive` to ensure any attached children (sprites, inventories) are also cleaned up.
- **Safety**: Ensure pops inside the pod are removed from `OccupiedTiles` or other spatial grids *before* launch to prevent ghost blockers. Actually, `despawn` handles entity removal, but if they had a `GridPosition`, they might still be in `EntityMap` until next update. `update_entity_map_system` usually handles this.

## Questions

*Builder: add questions here if spec is unclear.*
