# Specification: 448 - Grid Instability

## 1. Overview
The **Grid Instability** feature adds load limits and heat management to the power grid. Drawing too much power through a single cable or a specific circuit causes "Overload," generating heat damage and potential fires. Batteries can buffer surges but degrade. This forces players to build redundant, safer power grids rather than daisy-chaining their entire colony off a single cheap wire.

## 2. Dependencies
- `042` Energy System
- `033` Fire Propagation
- `140` Thermal Management

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::grid::{GridPos, GridMap};
    use scale::layer1::power::{PowerCable, PowerGrid, PowerConsumer};

    #[test]
    fn test_cable_overload_creates_heat() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(PowerGridPlugin);

        // Spawn a cable with max capacity 100
        let cable_pos = GridPos::new(5, 5);
        let cable_entity = app.world.spawn((
            PowerCable { capacity: 100.0, current_load: 0.0 },
            cable_pos,
        )).id();

        // Connect a consumer drawing 150 power
        app.world.spawn((
            PowerConsumer { draw: 150.0 },
            cable_pos,
        ));

        // Act: Evaluate grid
        app.update();

        // Assert: Cable is overloaded
        let cable = app.world.get::<PowerCable>(cable_entity).unwrap();
        assert!(cable.current_load > cable.capacity, "Cable load should exceed capacity");

        // Assert: Heat is generated on the tile
        let heat_map = app.world.resource::<HeatMap>();
        let tile_heat = heat_map.get_heat(cable_pos);
        assert!(tile_heat > 0.0, "Overloaded cable must generate heat");
    }

    #[test]
    fn test_extreme_overload_causes_fire() {
        let mut app = App::new();
        app.add_plugins(PowerGridPlugin);

        let pos = GridPos::new(3, 3);
        let cable = app.world.spawn((
            PowerCable { capacity: 50.0, current_load: 0.0 },
            pos,
        )).id();

        // Draw 300 power (massive overload)
        app.world.spawn((
            PowerConsumer { draw: 300.0 },
            pos,
        ));

        app.update();

        // Assert: Fire entity spawned on the cable position
        let mut fire_query = app.world.query::<&Fire>();
        assert!(fire_query.iter(&app.world).count() > 0, "Massive overload must start a fire");
    }

    #[test]
    fn test_batteries_buffer_surges() {
        let mut app = App::new();
        app.add_plugins(PowerGridPlugin);

        let pos = GridPos::new(2, 2);
        app.world.spawn((
            PowerCable { capacity: 100.0, current_load: 0.0 },
            pos,
        ));

        // Add a battery that can absorb the surge
        app.world.spawn((
            Battery { capacity: 500.0, charge: 500.0, max_discharge_rate: 100.0 },
            pos,
        ));

        app.world.spawn((
            PowerConsumer { draw: 150.0 },
            pos,
        ));

        app.update();

        // Assert: Cable is not overloaded because battery handles the local draw
        let heat_map = app.world.resource::<HeatMap>();
        let tile_heat = heat_map.get_heat(pos);
        assert_eq!(tile_heat, 0.0, "Battery should buffer surge and prevent heat");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct PowerCable {
    pub capacity: f32,
    pub current_load: f32,
}

#[derive(Component)]
pub struct Battery {
    pub capacity: f32,
    pub charge: f32,
    pub max_discharge_rate: f32,
}

pub fn evaluate_grid_load_system(
    mut cables: Query<(&mut PowerCable, &GridPos)>,
    consumers: Query<(&PowerConsumer, &GridPos)>,
    mut batteries: Query<(&mut Battery, &GridPos)>,
    mut heat_map: ResMut<HeatMap>,
    mut commands: Commands,
) {
    // Basic implementation: sum load per pos, subtract battery discharge, apply to cable
    // If cable load > capacity, add heat. If load > capacity * 2, spawn fire.

    // NOTE: This assumes point-to-point calculation for tests, actual grid tracing required for refactor.
}
```

## 5. REFACTOR Phase: Quality & Design

- **Grid Topology:** Power does not just flow point-to-point; it flows through connected networks. The `PowerGrid` resource must trace the shortest/safest paths from generators to consumers. If a path exceeds cable capacity, it should seek alternate routes. If none exist, the bottleneck cable overheats.
- **Heat Accumulation:** Heat shouldn't instantly vanish. Add to the existing thermal system where heat dissipates slowly over time.
- **Visual Feedback:** Overloaded cables should visibly spark or glow red to warn the player before a fire starts.
- **Battery Degradation:** Frequent deep discharges or buffering massive surges should lower the battery's maximum capacity permanently (durability damage).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Drawing more power than a cable's capacity generates heat.
- [ ] Massive overloads (e.g., >200% capacity) have a chance to ignite adjacent flammable tiles.

## 7. Technical Guidance
- The pathfinding for electricity (Kirchhoff's laws simplification) can be expensive. For the minimal implementation, just aggregate load on connected contiguous cable groups (sub-grids).
- Ensure integration with the existing `FirePropagation` systems (Spec 033). Spawning a `Fire` component should automatically trigger the fire spreading mechanics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
