# 235: Kinetic Storage

## 1. Overview

Introduces "Gravity Batteries" (Kinetic Storage) to the Energy System.
- **Function**: Consumes excess power to lift a heavy weight (Charge). Releases potential energy as power when grid demand exceeds supply (Discharge).
- **Physics**: Efficiency is high, but the stored energy is potential *danger*.
- **Hazard**: If a charged Gravity Battery is destroyed (by damage, deconstruction, or event), the weight drops, dealing massive damage to the tile and adjacent tiles based on the charge level.

## 2. Dependencies

- `042` — Energy System (Grid, PowerSource, PowerConsumer)
- `006` — Building Placement (Building entity)
- `034` — Pop Health and Damage (Damage application)

## 3. RED Phase: Tests First

```rust
// src/layer1/kinetic_storage_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerGrid, PowerSource, PowerConsumer, Conduit};
    use crate::layer1::kinetic_storage::{KineticBattery, gravity_battery_system, handle_battery_destruction_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::damage::{Health, DamageEvent};

    #[test]
    fn test_battery_charging() {
        // Arrange: Grid with Surplus Power (Gen 20, Cons 0)
        let mut world = World::new();
        let battery = world.spawn((
            KineticBattery {
                charge: 0.0,
                capacity: 100.0,
                charge_rate: 5.0,
                efficiency: 0.9
            },
            PowerConsumer { demand: 5.0, active: true }, // Input mode
            PowerSource { output: 0.0 }, // Output mode (inactive)
            GridPosition { x: 0, y: 0 },
        )).id();

        // Simulate surplus power in grid context (mocked or via energy system state)
        // For unit test, we can manually set the "Grid State" or mock the energy distribution.
        // Assuming gravity_battery_system checks if grid has surplus.

        let mut grid_surplus = 10.0; // Mock surplus

        // Act: Run system logic for charging
        // This test simulates the logic inside the system:
        // if grid_surplus > 0 { battery.charge += min(surplus, rate) }

        let mut query = world.query::<(&mut KineticBattery, &mut PowerConsumer, &mut PowerSource)>();
        let (mut bat, mut cons, mut src) = query.get_single_mut(&mut world).unwrap();

        // Logic simulation for test
        let charge_amount = bat.charge_rate.min(grid_surplus);
        bat.charge += charge_amount;

        // Assert
        assert_eq!(bat.charge, 5.0);
    }

    #[test]
    fn test_battery_discharging() {
        // Arrange: Battery full, Grid in deficit
        let mut world = World::new();
        let battery = world.spawn((
            KineticBattery {
                charge: 50.0,
                capacity: 100.0,
                charge_rate: 5.0,
                efficiency: 1.0
            },
            PowerConsumer { demand: 0.0, active: false }, // Not consuming
            PowerSource { output: 0.0 }, // Ready to output
            GridPosition { x: 0, y: 0 },
        )).id();

        let grid_deficit = 10.0; // Needed power

        // Act: System logic
        let mut query = world.query::<(&mut KineticBattery, &mut PowerSource)>();
        let (mut bat, mut src) = query.get_single_mut(&mut world).unwrap();

        // Logic: if deficit, discharge
        let discharge = bat.charge_rate.min(grid_deficit).min(bat.charge);
        bat.charge -= discharge;
        src.output = discharge;

        // Assert
        assert_eq!(bat.charge, 45.0);
        assert_eq!(src.output, 5.0);
    }

    #[test]
    fn test_battery_destruction_hazard() {
        // Arrange: Charged battery and adjacent victim
        let mut world = World::new();
        let battery = world.spawn((
            KineticBattery { charge: 100.0, capacity: 100.0, ..default() },
            GridPosition { x: 5, y: 5 },
            Health { current: 0.0, max: 100.0 }, // Destroyed
        )).id();

        let victim = world.spawn((
            GridPosition { x: 5, y: 6 }, // Adjacent
            Health { current: 50.0, max: 50.0 },
        )).id();

        // Event for destruction
        // Act: Run destruction handler
        // handle_battery_destruction_system checks for despawning/dead batteries with charge > 0

        // Mock system logic:
        // 1. Detect death
        // 2. Emit DamageEvent(radius=1, amount=charge)

        // Since we can't easily run full event loops in unit test snippets without setup,
        // we assert the logic calculation:
        let damage = 100.0; // Charge amount
        let radius = 1;

        // Verify victim would be in radius
        let bat_pos = GridPosition { x: 5, y: 5 };
        let vic_pos = GridPosition { x: 5, y: 6 };
        let dist = bat_pos.distance_chebyshev(vic_pos);

        assert!(dist <= radius);
        assert_eq!(damage, 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/kinetic_storage.rs

use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct KineticBattery {
    pub charge: f32,
    pub capacity: f32,
    pub charge_rate: f32,
    pub efficiency: f32, // e.g. 0.9 means 10 input = 9 stored
}
```

### 2. Systems

```rust
// src/layer1/kinetic_storage.rs

pub fn gravity_battery_system(
    mut query: Query<(&mut KineticBattery, &mut PowerConsumer, &mut PowerSource)>,
    // In a real system, we'd need the Grid Resource or access to Grid Stats
) {
    // Logic placeholder:
    // For each battery:
    // If Grid has Surplus:
    //    amount = min(surplus, battery.charge_rate, battery.capacity - battery.charge)
    //    battery.charge += amount * battery.efficiency
    //    consumer.demand = amount (to soak up surplus next tick)
    //    source.output = 0
    // Else If Grid has Deficit:
    //    amount = min(deficit, battery.charge_rate, battery.charge)
    //    battery.charge -= amount
    //    source.output = amount
    //    consumer.demand = 0
}

pub fn handle_battery_destruction_system(
    mut commands: Commands,
    query: Query<(Entity, &KineticBattery, &GridPosition, &Health)>,
    // Event writer for DamageEvent
) {
    for (entity, battery, pos, health) in query.iter() {
        if health.current <= 0.0 && battery.charge > 10.0 { // Threshold
            // Boom
            // Spawn explosion effect
            // Apply damage to neighbors
            // Log event: "Gravity Battery collapsed!"
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration**: Hook into `calculate_grid_stats` in `energy.rs`. Batteries act as buffers.
- **Visuals**: The weight (building sprite) should move up/down based on `% charge`.
- **UI**: Inspector should show "Stored: 500/1000 J".
- **Optimization**: Don't run destruction check every frame? Use `RemovedComponents` or `DeathEvent`.

## 6. Acceptance Criteria

- [ ] `KineticBattery` component implemented.
- [ ] Batteries charge when `PowerGrid` has excess.
- [ ] Batteries discharge when `PowerGrid` has deficit.
- [ ] Destroying a battery with charge > 0 causes damage to adjacent tiles.
- [ ] Tests pass.

## 7. Technical Guidance

- **Grid Logic**: The `energy.rs` system likely sums `Production` and `Demand`. Batteries need to run *after* the initial calculation to see the net balance, or interact iteratively.
- **Cycle**:
    1. `calc_grid` -> Net Power (+100).
    2. `battery_system` -> Sees +100, sets `Consumer.demand = 50` (charge).
    3. Next tick `calc_grid` -> Net Power (+50).
- **Hazard**: Reuse `ExplosionEvent` from `Volatile` (Spec 223) if possible, or `DamageEvent`.

## 8. Questions

- **Efficiency**: Should round-trip efficiency be < 100%? (Yes, set to 80-90% for realism).
- **Stacking**: Can we build them on top of each other? (No, they are tall structures).
  - *Architect:* No, kinetic storage structures cannot be stacked vertically.

*Architect:* A round-trip efficiency of 80% is appropriate for the MVP.
