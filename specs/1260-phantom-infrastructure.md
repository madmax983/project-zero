# Specification: Phantom Infrastructure (1260)

## 1. Overview
The base has a history that can easily be forgotten as new buildings are placed over old networks. The "Phantom Infrastructure" feature models the "spaghetti code" of city planning. Power conduits built in the early game become "Occluded" (invisible) under floors and walls over time. When a player deconstructs a wall or floor, there is a risk of severing a forgotten "pass-through" line powering a distant sector, leading to unexpected outages.

This creates a tension between the desire for renovation (a clean, modern layout) and inertia (the fear of breaking legacy systems that are quietly keeping the colony alive).

## 2. Dependencies
- Base grid system and construction/deconstruction logic.
- Power and/or fluid networking systems (conduits).
- Component modeling visibility (or lack thereof) for infrastructure, such as `Occluded` or similar mechanic over time.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::{Conduit, PowerSource, PowerConsumer};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;

    #[test]
    fn test_infrastructure_becomes_occluded_over_time() {
        // Arrange
        let mut world = World::new();

        let entity = world.spawn((
            Conduit,
            GridPosition { x: 5, y: 5 },
            Age { ticks: OCCLUSION_THRESHOLD_TICKS + 1 },
        )).id();

        // Act
        world.run_system_once(occlude_aging_infrastructure_system).unwrap();

        // Assert: The conduit should now have the Occluded component
        assert!(world.get::<Occluded>(entity).is_some());
    }

    #[test]
    fn test_deconstructing_building_over_occluded_infrastructure_damages_it() {
        // Arrange
        let mut world = World::new();

        // Spawn an occluded conduit
        let conduit_entity = world.spawn((
            Conduit,
            Occluded,
            GridPosition { x: 10, y: 10 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Spawn a building on the same tile
        let building_entity = world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 10, y: 10 },
        )).id();

        let designation = world.spawn((
            crate::layer1::designation::Designation {
                designation_type: crate::layer1::designation::DesignationType::Demolish,
            },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Act: Execute demolish
        crate::layer1::execution::demolish::execute_demolish(&mut world, designation);

        // Assert: Building is despawned, and the occluded conduit underneath takes damage or is severed
        assert!(world.get_entity(building_entity).is_none());

        let conduit_structure = world.get::<Structure>(conduit_entity).unwrap();
        assert!(conduit_structure.current_hp < conduit_structure.max_hp, "Occluded conduit should be damaged by deconstruction above it");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

// Builder must implement the systems required by the tests above:
// 1. occlude_aging_infrastructure_system
// 2. handle_deconstruction_system
// 3. Power network recalculation system when a cable is severed.
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** Direct iteration over all occluded infrastructure might be slow on large grids. Consider using a spatial hash map or grid resource to quickly look up entities at a specific `GridPosition`.
- **Improvements:** Make the chance of severing an occluded conduit depend on worker skill or specific tools used for deconstruction. Add an "Underground Scanner" tool/overlay that lets the player temporarily see occluded pipes, costing energy or research points.
- **Visuals:** Add a visual update system that hides the sprites of entities with the `Occluded` component, or changes their z-index so they render strictly below floors/walls and are excluded from the default view overlay.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Conduits older than `OCCLUSION_THRESHOLD_TICKS` receive the `Occluded` component.
- [ ] Deconstructing a Wall over an `Occluded` entity deals damage to or severs the hidden entity.

## 7. Technical Guidance
- **Integration Points:** You will need to hook into the existing building/deconstruction systems and the age/tick tracking systems.
- **Gotchas:** Ensure that when an `Occluded` conduit is destroyed, the power/fluid network graph is correctly recalculated. Simply destroying the entity might leave ghost references in a `PowerGrid` resource.
- **Rendering:** `Occluded` should probably tie into the rendering system to ensure these items are hidden from the standard view but might be visible in a specific "Infrastructure View" if they are not fully occluded yet.

## 8. Questions
*Builder: add questions here if spec is unclear.*

