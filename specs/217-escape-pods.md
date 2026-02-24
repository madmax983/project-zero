# 217: Escape Pods

## Overview

The colony must have a way to evacuate personnel in case of catastrophic failure. Escape Pods provide a "lifeboat" mechanism, allowing Pops to be launched into orbit (Layer 2) as distress signals. This is a critical safety valve for the "game over" state, allowing some legacy to survive.

## Dependencies

- `006` — Building Placement (Implemented)
- `004` — Pop Entity (Implemented)
- `099` — Fleet Movement (Layer 2) (Implemented)

## RED Phase: Tests First

```rust
// src/layer1/escape_pod_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_pod_assignment() {
        // Arrange
        let mut world = setup_world();
        let pod = spawn_escape_pod(&mut world);
        let pop = spawn_pop(&mut world);

        // Act
        let result = try_assign_pop_to_pod(&mut world, pop, pod);

        // Assert
        assert!(result.is_ok());
        assert_eq!(world.get::<Occupant>(pod).unwrap().entity, pop);
    }

    #[test]
    fn test_escape_pod_launch() {
        // Arrange
        let mut world = setup_world();
        let pod = spawn_escape_pod(&mut world);
        let pop = spawn_pop(&mut world);
        assign_pop(&mut world, pop, pod);

        // Act
        launch_pod(&mut world, pod);

        // Assert
        // Pod should be despawned from Layer 1
        assert!(world.get_entity(pod).is_none());
        // Pop should be despawned from Layer 1
        assert!(world.get_entity(pop).is_none());
        // Distress signal should exist in Layer 2 (mock check)
        assert!(layer2_signal_exists(&world));
    }
}
```

## GREEN Phase: Minimal Implementation

### Components

```rust
// src/layer1/escape_pod.rs

#[derive(Component)]
pub struct EscapePod;

#[derive(Component)]
pub struct Occupant {
    pub entity: Entity,
}

pub fn try_assign_pop(world: &mut World, pop: Entity, pod: Entity) -> Result<(), &'static str> {
    if world.get::<Occupant>(pod).is_some() {
        return Err("Pod full");
    }
    world.entity_mut(pod).insert(Occupant { entity: pop });
    Ok(())
}

pub fn launch_pod_system(
    mut commands: Commands,
    query: Query<(Entity, &Occupant), With<EscapePod>>,
    mut layer2_events: EventWriter<Layer2DistressSignal>,
) {
    for (pod, occupant) in query.iter() {
        // Despawn L1 entities
        commands.entity(occupant.entity).despawn_recursive();
        commands.entity(pod).despawn_recursive();

        // Emit L2 event
        layer2_events.send(Layer2DistressSignal::new());
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Ensure `BuildingType::EscapePod` is registered in `building.rs`.
- **UI**: Add a "Launch" button to the building inspector.
- **Lore**: Add log message "Pop X has escaped to orbit."

## Acceptance Criteria

- [x] Escape Pod building exists.
- [x] Pops can enter pods.
- [x] Launching removes pod/pop from map.
- [x] Launching triggers Layer 2 event.
