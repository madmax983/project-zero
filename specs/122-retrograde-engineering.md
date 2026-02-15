# 122: Retrograde Engineering

## Overview

Implements **Retrograde Engineering**, allowing players to deconstruct **Heirloom** buildings (e.g., Ancient Reactor) to gain massive amounts of **Knowledge** instead of resources. This represents "breaking it to understand how it works."

Currently, demolishing a building simply destroys it. This feature adds a strategic choice: keep the decaying Heirloom for its utility, or sacrifice it to jump-start research into its technology.

## Dependencies

- `070` — Heirloom Tech (for `Heirloom` component)
- `029` — Knowledge System (for `ColonyResources.knowledge`)
- `020` — Construction Costs (for resource structures)

## RED Phase: Tests First

Write these tests in `src/layer1/retrograde_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::heirloom::Heirloom;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::map::GridPosition;
    use crate::layer1::execution::execute_demolish; // Assuming execute_demolish is public or testable

    #[test]
    fn test_demolish_normal_building_gives_no_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::map::OccupiedTiles::default());

        // Mock dependencies for execute_demolish (ScreenShake, etc) if needed
        world.insert_resource(crate::layer1::map::ScreenShake::default());

        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
        )).id();

        let designation = world.spawn((
            crate::layer1::designation::Designation {
                designation_type: crate::layer1::designation::DesignationType::Demolish
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Execute
        let success = execute_demolish(&mut world, designation);
        assert!(success);

        // Verify no knowledge gain
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.knowledge, 0.0);
    }

    #[test]
    fn test_demolish_heirloom_gives_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::map::OccupiedTiles::default());
        world.insert_resource(crate::layer1::map::ScreenShake::default());

        let heirloom = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Heirloom,
            GridPosition { x: 5, y: 5 },
        )).id();

        let designation = world.spawn((
            crate::layer1::designation::Designation {
                designation_type: crate::layer1::designation::DesignationType::Demolish
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Execute
        execute_demolish(&mut world, designation);

        // Verify massive knowledge gain
        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge >= 500.0); // Tweak value as needed
    }

    #[test]
    fn test_demolish_heirloom_notification() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::map::OccupiedTiles::default());
        world.insert_resource(crate::layer1::map::ScreenShake::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let heirloom = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Heirloom,
            GridPosition { x: 5, y: 5 },
        )).id();

        let designation = world.spawn((
            crate::layer1::designation::Designation {
                designation_type: crate::layer1::designation::DesignationType::Demolish
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        execute_demolish(&mut world, designation);

        let log = world.resource::<crate::shared::log::MessageLog>();
        assert!(log.messages.last().unwrap().text.contains("Retrograde Engineering"));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `execute_demolish`

In `src/layer1/execution.rs`, modify `execute_demolish` to check for `Heirloom`.

```rust
fn execute_demolish(world: &mut World, designation_entity: Entity) -> bool {
    // ... existing setup ...

    // Find building
    if let Some(entity) = building_entity {
        // Check for Heirloom BEFORE despawn
        let is_heirloom = world.get::<Heirloom>(entity).is_some();
        let building_label = world.get::<Building>(entity).unwrap().building_type.label();

        if is_heirloom {
            // Award Knowledge
            let knowledge_gain = 500.0; // Balance this value
            if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
                res.knowledge += knowledge_gain;
            }

            // Log it
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add_colored(
                    format!("Retrograde Engineering complete on {}. Gained {} Knowledge.", building_label, knowledge_gain),
                    Color::Cyan
                );
            }
        }

        // Proceed with destruction
        world.despawn(entity);
        // ... existing juice/particles ...
    }

    // ... existing cleanup ...
}
```

## REFACTOR Phase: Quality & Design

- **Balance**: The amount of knowledge should probably depend on the specific Heirloom type (Reactor > Fabricator).
- **Juice**: Different particle effect for Heirloom destruction (maybe Cyan "data" sparks instead of Red debris).
- **Warning**: The UI should probably warn the player before they designate an Heirloom for demolition, but that's a UI task.
- **Refunds**: If we ever implement resource refunds for normal buildings, ensure Heirlooms *don't* get them (only Knowledge).

## Acceptance Criteria

- [ ] Demolishing a normal building works as before (no knowledge).
- [ ] Demolishing an Heirloom grants significant Knowledge.
- [ ] A notification is logged upon Heirloom destruction.
- [ ] The Heirloom is removed from the world.
- [ ] Tests pass.

## Technical Guidance

- `Heirloom` component is in `crate::layer1::heirloom`.
- `ColonyResources` is in `crate::layer1::resources`.
- Ensure you check for `Heirloom` *before* calling `world.despawn(entity)`.
