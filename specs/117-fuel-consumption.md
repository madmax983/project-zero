# 117: Fuel Consumption

## 1. Overview

Currently, power generators produce infinite energy for free. This spec introduces a requirement for **Fuel** (produced by Refineries) to operate generators. This creates a critical supply chain dependency: Mining -> Refining -> Power -> Industry.

We introduce a `FuelConsumer` component. Systems will check this component, consume fuel from the global stockpile (`ColonyResources`), and disable the `PowerSource` if fuel is missing.

## 2. Dependencies

- `specs/042-energy-system.md` (Energy System)
- `specs/104-fuel-industry.md` (Fuel Resource)

## 3. RED Phase: Tests First

```rust
// src/layer1/energy_tests.rs or fuel_consumption_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerSource, FuelConsumer, process_fuel_consumption_system};
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_power_source_default_active() {
        // PowerSource should default to active (or we add an active field)
        let source = PowerSource { output: 10.0, active: true };
        assert!(source.active);
    }

    #[test]
    fn test_fuel_consumption_success() {
        let mut world = World::new();
        // Setup resources with fuel
        world.insert_resource(ColonyResources {
            fuel: 5.0,
            ..ColonyResources::zeroed()
        });

        // Spawn Generator with FuelConsumer
        let gen = world.spawn((
            PowerSource { output: 10.0, active: true },
            FuelConsumer { amount: 1.0 },
        )).id();

        // Run system
        process_fuel_consumption_system(&mut world);

        // Verify fuel consumed
        let res = world.resource::<ColonyResources>();
        assert!((res.fuel - 4.0).abs() < f32::EPSILON);

        // Verify generator still active
        let source = world.get::<PowerSource>(gen).unwrap();
        assert!(source.active);
    }

    #[test]
    fn test_fuel_consumption_failure_no_fuel() {
        let mut world = World::new();
        // Setup resources with NO fuel
        world.insert_resource(ColonyResources {
            fuel: 0.0,
            ..ColonyResources::zeroed()
        });

        // Spawn Generator
        let gen = world.spawn((
            PowerSource { output: 10.0, active: true },
            FuelConsumer { amount: 1.0 },
        )).id();

        // Run system
        process_fuel_consumption_system(&mut world);

        // Verify fuel unchanged (0)
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.fuel, 0.0);

        // Verify generator INACTIVE
        let source = world.get::<PowerSource>(gen).unwrap();
        assert!(!source.active);
    }

    #[test]
    fn test_fuel_consumption_reactivation() {
        let mut world = World::new();
        // Start with no fuel
        world.insert_resource(ColonyResources {
            fuel: 0.0,
            ..ColonyResources::zeroed()
        });

        let gen = world.spawn((
            PowerSource { output: 10.0, active: false }, // Previously disabled
            FuelConsumer { amount: 1.0 },
        )).id();

        // Add fuel
        world.resource_mut::<ColonyResources>().fuel = 10.0;

        // Run system
        process_fuel_consumption_system(&mut world);

        // Verify active
        let source = world.get::<PowerSource>(gen).unwrap();
        assert!(source.active);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Update `PowerSource`

Modify `src/layer1/energy.rs`:

```rust
#[derive(Component, Debug, Clone)]
pub struct PowerSource {
    pub output: f32,
    pub active: bool, // New field
}

impl Default for PowerSource {
    fn default() -> Self {
        Self { output: 10.0, active: true }
    }
}
```

### 2. Update `calculate_grid_stats` and `bfs_grid`

Update `bfs_grid` in `src/layer1/energy.rs` to only sum output if `source.active` is true.

```rust
if let Some(source) = world.get::<PowerSource>(entity) {
    if source.active {
        total_production += source.output;
    }
}
```

### 3. Add `FuelConsumer` Component

```rust
#[derive(Component, Debug, Clone)]
pub struct FuelConsumer {
    pub amount: f32, // Amount of fuel consumed per tick
}
```

### 4. Implement `process_fuel_consumption_system`

```rust
pub fn process_fuel_consumption_system(world: &mut World) {
    let mut query = world.query::<(Entity, &mut PowerSource, &FuelConsumer)>();
    let mut consumed_fuel = 0.0;

    // We need to collect entities to modify resources separately or use a command buffer,
    // but typically we can iterate and modify provided we handle the resource access correctly.
    // However, iterating query yields mutable reference to components, so we can't borrow World resource mutable at same time easily inside loop if using `iter_mut(world)`.
    // Best approach: Collect updates, then apply.

    let mut updates = Vec::new();

    // Check available fuel first?
    // Actually, we process sequentially or aggregate demand.
    // For MVP, simplistic sequential processing.

    // We can't access ColonyResources inside the query loop if we use `world` for the query.
    // So:

    let available_fuel = world.get_resource::<ColonyResources>().map(|r| r.fuel).unwrap_or(0.0);
    let mut fuel_spent = 0.0;

    for (entity, mut source, consumer) in query.iter_mut(world) {
        if available_fuel - fuel_spent >= consumer.amount {
            // Have fuel
            if !source.active {
                source.active = true;
            }
            fuel_spent += consumer.amount;
        } else {
            // Out of fuel
            if source.active {
                source.active = false;
                // Log outage?
            }
        }
    }

    // Deduct total
    if fuel_spent > 0.0 {
        if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
            res.fuel -= fuel_spent;
        }
    }
}
```

### 5. Update `spawn_building`

In `src/layer1/building.rs`, update `BuildingType::Generator` to include `FuelConsumer`.

```rust
BuildingType::Generator => {
    entity.insert((
        PowerSource { output: 10.0, active: true },
        FuelConsumer { amount: 0.05 }, // 1 fuel lasts 20 ticks
    ));
}
```

## 5. REFACTOR Phase: Quality & Design

-   **Refactor `PowerSource`**: The addition of `active` field is a breaking change for existing tests that initialize `PowerSource`. They will need `active: true` added or use `..Default::default()`.
-   **Fuel Types**: Currently `Fuel` is a generic resource. Future refactor could allow burning Wood or Coal directly if `Fuel` (refined) is missing, but for now stick to `Refinery` output.
-   **Logging**: Add a notification when power goes out due to lack of fuel.
-   **Optimization**: If we have 100 generators, iterating them is fine.

## 6. Acceptance Criteria (Testable!)

- [ ] `PowerSource` has `active` field.
- [ ] `Generator` entity has `FuelConsumer` component.
- [ ] Generators consume fuel from global stockpile.
- [ ] Generators stop producing power when fuel runs out.
- [ ] Generators restart when fuel is added.
- [ ] `cargo test` passes.

## 7. Technical Guidance

-   Run `process_fuel_consumption_system` **before** `power_grid_system` in the schedule.
-   Be careful with floating point comparisons for fuel (use epsilon or ensure strictly `>=`).
-   Update `spawn_building` for `AncientReactor` to NOT have `FuelConsumer` (it's infinite/magic for now).

## 8. Questions

-   *Builder:* How much fuel does a generator consume?
    -   *Architect:* Start with `0.05` per tick. A refinery produces `1.0` fuel from `2.0` ore. If refining takes ~10 ticks, one refinery can support maybe 2 generators. Tune `RefiningProgress.max` and consumption rate to balance.
