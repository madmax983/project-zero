# 125: Grid Instability

## Overview

Adds depth to the Energy System (Spec 042) by introducing **Grid Instability**.
Currently, power is binary: if Demand > Supply, everything shuts down.
This spec introduces:
1.  **Batteries**: Buildings that store excess power to buffer brownouts.
2.  **Brownouts**: If Supply < Demand, machines flicker (randomly active) instead of a hard shutdown.
3.  **Overload**: If Demand > 1.5x Supply, the grid becomes unstable, risking damage to conduits and generators.

## Dependencies

- `042` — Energy System (Base grid logic)
- `033` — Fire Propagation (Overload consequences)
- `071` — Structural Integrity (Overload damage)

## RED Phase: Tests First

Write these tests in `src/layer1/energy/instability_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{
        PowerGrid, PowerSource, PowerConsumer, Battery, Conduit,
        calculate_grid_stats, power_grid_system
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health; // From Spec 034/071

    // 1. Battery Logic
    #[test]
    fn test_battery_storage() {
        let mut battery = Battery {
            capacity: 100.0,
            charge: 0.0,
            max_throughput: 10.0,
        };

        // Charge
        battery.charge(50.0);
        assert_eq!(battery.charge, 50.0);

        // Overcharge
        battery.charge(100.0);
        assert_eq!(battery.charge, 100.0);

        // Discharge
        let drained = battery.discharge(20.0);
        assert_eq!(drained, 20.0);
        assert_eq!(battery.charge, 80.0);
    }

    // 2. Grid Buffering
    #[test]
    fn test_battery_buffers_shortage() {
        let mut world = World::new();

        // Generator: 10 Output
        world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // Consumer: 15 Demand
        let consumer = world.spawn((
            PowerConsumer { demand: 15.0, active: false },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Battery: 100 Charge
        world.spawn((
            Battery { capacity: 100.0, charge: 100.0, max_throughput: 10.0 },
            GridPosition { x: 0, y: 2 },
            Conduit, // Connects to grid
        ));

        // Run system
        power_grid_system(&mut world);

        // Consumer should be ACTIVE because Battery covered the 5.0 deficit
        let state = world.get::<PowerConsumer>(consumer).unwrap();
        assert!(state.active);

        // Battery should be drained by 5.0
        let battery = world.query::<&Battery>().single(&world);
        assert_eq!(battery.charge, 95.0);
    }

    // 3. Brownout (Partial Activation)
    #[test]
    fn test_brownout_flickering() {
        let mut world = World::new();

        // Generator: 10 Output
        world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
        ));

        // 10 Consumers: 2 Demand each (Total 20)
        // Deficit: 10 Supply vs 20 Demand -> 50% Supply Ratio
        let mut consumers = Vec::new();
        for i in 0..10 {
            consumers.push(world.spawn((
                PowerConsumer { demand: 2.0, active: true }, // Start active
                GridPosition { x: 0, y: i + 1 },
            )).id());
        }

        // Run system
        power_grid_system(&mut world);

        // Check activation count
        // Should be roughly 50% (5 consumers) active
        // Allow variance for RNG, but ensure SOME are off and SOME are on
        let active_count = consumers.iter()
            .filter(|&e| world.get::<PowerConsumer>(*e).unwrap().active)
            .count();

        assert!(active_count < 10, "Not all consumers should be active");
        assert!(active_count > 0, "Some consumers should be active");
    }

    // 4. Overload Damage
    #[test]
    fn test_overload_damage() {
        let mut world = World::new();

        // Generator: 10 Output
        let gen = world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
            Health { current: 100.0, max: 100.0 }, // Has Health
        )).id();

        // Consumer: 30 Demand (300% Load) -> Severe Overload
        world.spawn((
            PowerConsumer { demand: 30.0, active: true },
            GridPosition { x: 0, y: 1 },
        ));

        // Run system multiple times to trigger probability
        for _ in 0..100 {
            power_grid_system(&mut world);
        }

        // Generator should have taken damage
        let health = world.get::<Health>(gen).unwrap();
        assert!(health.current < 100.0, "Generator should take damage from 300% overload");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Battery Component

In `src/layer1/energy.rs`:

```rust
#[derive(Component, Debug, Clone)]
pub struct Battery {
    pub capacity: f32,
    pub charge: f32,
    pub max_throughput: f32,
}

impl Battery {
    pub fn charge(&mut self, amount: f32) {
        self.charge = (self.charge + amount).min(self.capacity);
    }

    pub fn discharge(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.charge).min(self.max_throughput);
        self.charge -= actual;
        actual
    }
}
```

### 2. Update `power_grid_system`

Modify `power_grid_system` in `src/layer1/energy.rs` to handle buffering and brownouts.

```rust
use rand::Rng; // Add rand dependency
use rand::seq::SliceRandom; // For choose

pub fn power_grid_system(world: &mut World) {
    let grid_map = build_grid_map(world);
    let mut visited = HashSet::new();

    // Iterate grids
    for start_pos in grid_map.keys() {
        if visited.contains(start_pos) { continue; }

        let (mut prod, demand, entities) = bfs_grid(*start_pos, &grid_map, world, &mut visited);

        // 1. Calculate Net
        let mut net = prod - demand;

        // 2. Handle Batteries
        // Need to query batteries in this grid.
        // We can collect battery entities during BFS or filter `entities`.
        let mut batteries: Vec<Entity> = entities.iter()
            .filter(|e| world.get::<Battery>(**e).is_some())
            .copied()
            .collect();

        if net > 0.0 {
            // Surplus: Charge batteries
            let charge_per_battery = net / batteries.len().max(1) as f32;
            for bat_entity in &batteries {
                if let Some(mut bat) = world.get_mut::<Battery>(*bat_entity) {
                    bat.charge(charge_per_battery);
                }
            }
        } else if net < 0.0 {
            // Deficit: Discharge batteries
            let needed = -net;
            let mut provided = 0.0;
            for bat_entity in &batteries {
                if let Some(mut bat) = world.get_mut::<Battery>(*bat_entity) {
                    provided += bat.discharge(needed - provided);
                    if provided >= needed { break; }
                }
            }
            prod += provided;
            net = prod - demand;
        }

        // 3. Handle Activation & Overload
        let supply_ratio = if demand > 0.0 { prod / demand } else { 1.0 };
        let overload_ratio = if prod > 0.0 { demand / prod } else { 1.0 };

        let mut rng = rand::thread_rng();

        // Overload Check (>150% demand)
        if overload_ratio > 1.5 {
            // Risk of damage to random entity in grid
            if rng.gen_bool(0.05 * (overload_ratio - 1.5) as f64) {
                // Pick random entity
                if let Some(victim) = entities.choose(&mut rng) {
                    // Apply damage (direct Health modification or via Event)
                    // For MVP: Direct
                    if let Some(mut health) = world.get_mut::<crate::layer1::health::Health>(*victim) {
                        health.take_damage(10.0);
                    }
                }
            }
        }

        // Activation
        for entity in entities {
            if let Some(mut consumer) = world.get_mut::<PowerConsumer>(entity) {
                if net >= 0.0 {
                    consumer.active = true;
                } else {
                    // Brownout: Probabilistic activation
                    // e.g. 80% supply -> 80% chance to run
                    consumer.active = rng.gen_bool(supply_ratio as f64);
                }
            }
        }
    }
}
```

### 3. Register Battery Building

Update `src/layer1/building.rs`:
- Add `BuildingType::Battery`.
- Configure cost/description.

## REFACTOR Phase: Quality & Design

- **Optimization**: Don't query `Health` or `Battery` inside the loop if possible. Collect mutable references carefully or use `param_set`.
- **Visuals**: Add particles for Brownouts (sparks) and Overload (smoke).
- **UI**: Show Grid Stats (Supply/Demand/Battery) when clicking a power building.

## Acceptance Criteria

- [ ] `Battery` component exists and functions.
- [ ] Surplus power charges batteries.
- [ ] Deficit power drains batteries.
- [ ] Batteries prevent brownouts until empty.
- [ ] Brownouts cause flickering (probabilistic activation) instead of total shutdown.
- [ ] Severe overload (>150%) causes damage to grid entities.
- [ ] Tests pass.

## Technical Guidance

- Use `rand::thread_rng()` for brownout/overload logic.
- Be careful with `world.get_mut` inside loops over `entities` vector. Use `Query::get_mut` if iterating via Query, or split the logic. Since `power_grid_system` has `world: &mut World`, you can iterate `entities` (Vec<Entity>) and call `world.get_mut(entity)`.
- Ensure `Health` component is available (Spec 034/071).

## Questions

*Builder: How to visualize Overload?*
*Architect: A blinking visual indicator (e.g., red warning icon) on the overloaded building in the UI.*
*Architect: In the TUI, overloaded buildings can flash red or display a specific "Overload" glyph. In a graphical renderer, they could spark or smoke.*
*Architect: For MVP, console logs or Health bar dropping is enough. Visuals later.*
