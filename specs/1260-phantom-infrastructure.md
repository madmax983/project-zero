# Specification: Phantom Infrastructure (1260)

## 1. Overview
The base has a history that can easily be forgotten as new buildings are placed over old networks. The "Phantom Infrastructure" feature models the "spaghetti code" of city planning. Pipes and Cables built in the early game become "Occluded" (invisible) under floors and walls over time. When a player deconstructs a wall or floor, there is a risk of severing a forgotten "pass-through" line powering a distant sector, leading to unexpected outages.

This creates a tension between the desire for renovation (a clean, modern layout) and inertia (the fear of breaking legacy systems that are quietly keeping the colony alive).

## 2. Dependencies
- Base grid system and construction/deconstruction logic.
- Power and/or fluid networking systems (cables, pipes).
- Component modeling visibility (or lack thereof) for infrastructure, such as `Occluded` or similar mechanic over time.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::infrastructure::{Cable, Wall, GridPosition, DeconstructEvent, NetworkNode, PowerGrid};

    #[test]
    fn test_infrastructure_becomes_occluded_over_time() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, occlude_aging_infrastructure_system);

        let entity = app.world.spawn((
            Cable,
            GridPosition { x: 5, y: 5 },
            Age { ticks: 0 },
        )).id();

        // Act: Fast forward time to the occlusion threshold
        let mut age = app.world.get_mut::<Age>(entity).unwrap();
        age.ticks = OCCLUSION_THRESHOLD_TICKS + 1;

        app.update();

        // Assert: The cable should now have the Occluded component
        assert!(app.world.get::<Occluded>(entity).is_some());
    }

    #[test]
    fn test_deconstructing_wall_over_occluded_infrastructure_damages_it() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DeconstructEvent>();
        app.add_systems(Update, handle_deconstruction_system);

        // Spawn an occluded cable
        let cable_entity = app.world.spawn((
            Cable,
            Occluded,
            NetworkNode,
            GridPosition { x: 10, y: 10 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Spawn a wall on the same tile
        let wall_entity = app.world.spawn((
            Wall,
            GridPosition { x: 10, y: 10 },
        )).id();

        // Act: Issue a deconstruct event for the wall
        app.world.resource_mut::<Events<DeconstructEvent>>().send(DeconstructEvent {
            entity: wall_entity,
        });

        app.update();

        // Assert: Wall is despawned, and the occluded cable underneath takes damage or is severed
        assert!(app.world.get_entity(wall_entity).is_none());

        let cable_health = app.world.get::<Health>(cable_entity).unwrap();
        assert!(cable_health.current < cable_health.max, "Occluded cable should be damaged by deconstruction above it");
    }

    #[test]
    fn test_network_sever_causes_downstream_power_loss() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, handle_deconstruction_system);
        app.add_event::<DeconstructEvent>();

        // Setup a simple power grid: Generator -> Occluded Cable -> Consumer
        let generator_entity = app.world.spawn((PowerNode { generates: true }, PowerNetworkId(1))).id();
        let cable_entity = app.world.spawn((
            Cable,
            Occluded,
            GridPosition { x: 10, y: 10 },
            Health { current: 100.0, max: 100.0 },
            PowerNetworkId(1)
        )).id();
        let consumer_entity = app.world.spawn((PowerNode { generates: false }, PowerPowered(true), PowerNetworkId(1))).id();

        // Spawn a wall on the same tile
        let wall_entity = app.world.spawn((
            Wall,
            GridPosition { x: 10, y: 10 },
        )).id();

        // Act: Deconstruct the wall to sever the occluded cable
        app.world.resource_mut::<Events<DeconstructEvent>>().send(DeconstructEvent {
            entity: wall_entity,
        });
        app.update();

        // We'll simulate the graph recalculation that a Builder must implement
        // which would cause the consumer to lose power if it relies on that cable.
        app.update();

        // Assert: The cable is destroyed, so the consumer should no longer receive power.
        // Assuming the Builder implements the recalculation logic properly.
        let is_powered = app.world.get::<PowerPowered>(consumer_entity).unwrap().0;
        assert!(!is_powered, "Consumer should lose power when occluded cable is severed");
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
- **Improvements:** Make the chance of severing an occluded cable depend on worker skill or specific tools used for deconstruction. Add an "Underground Scanner" tool/overlay that lets the player temporarily see occluded pipes, costing energy or research points.
- **Visuals:** Add a visual update system that hides the sprites of entities with the `Occluded` component, or changes their z-index so they render strictly below floors/walls and are excluded from the default view overlay.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Cables and pipes older than `OCCLUSION_THRESHOLD_TICKS` receive the `Occluded` component.
- [ ] Deconstructing a Wall over an `Occluded` entity deals damage to or severs the hidden entity.

## 7. Technical Guidance
- **Integration Points:** You will need to hook into the existing building/deconstruction systems and the age/tick tracking systems.
- **Gotchas:** Ensure that when an `Occluded` cable is destroyed, the power/fluid network graph is correctly recalculated. Simply destroying the entity might leave ghost references in a `PowerGrid` resource.
- **Rendering:** `Occluded` should probably tie into the rendering system to ensure these items are hidden from the standard view but might be visible in a specific "Infrastructure View" if they are not fully occluded yet.

## 8. Questions
*Builder: add questions here if spec is unclear.*
