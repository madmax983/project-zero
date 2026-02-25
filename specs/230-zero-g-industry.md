# 230: Zero-G Industry

## Overview

Orbital Stations can now be specialized as **Orbital Factories**.
These factories utilize the unique microgravity environment to produce advanced resources that are impossible to manufacture on a planet surface.
This creates a critical economic loop between Layer 1 (raw materials) and Layer 2 (advanced manufacturing).

This spec introduces:
1.  **StationType::OrbitalFactory**: A new station variant.
2.  **ZeroGFactory Component**: Logic for production cycles.
3.  **StationInventory Component**: Storage for input/output resources on stations.
4.  **New Resources**: `FoamMetal` and `PerfectCrystal`.

## Dependencies

- `152` — Orbital Stations (for `Station`, `StationType`)
- `101` — System Mining (for resource concepts)
- `018` — Mining and Resources (for `ResourceType` enum)

## RED Phase: Tests First

Write these tests in `src/layer2/industry_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ResourceType;
    use crate::layer2::station::{Station, StationType};
    use crate::layer2::industry::{ZeroGFactory, StationInventory, zero_g_production_system};

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components if needed
        world
    }

    #[test]
    fn test_zero_g_factory_production() {
        let mut world = setup_world();

        // Setup Orbital Factory with inputs
        let station = world.spawn((
            Station { station_type: StationType::OrbitalFactory },
            ZeroGFactory {
                production_rate: 1.0,
                input_type: ResourceType::Metal,
                output_type: ResourceType::FoamMetal,
                input_amount: 10.0,
                output_amount: 1.0,
                progress: 0.0,
                max_progress: 10.0,
            },
            StationInventory {
                contents: vec![
                    (ResourceType::Metal, 100.0), // Plenty of input
                ],
                capacity: 1000.0,
            }
        )).id();

        // Run system for enough ticks to complete one cycle (10.0 progress needed, 1.0 rate)
        let mut schedule = Schedule::default();
        schedule.add_systems(zero_g_production_system);

        for _ in 0..11 {
            schedule.run(&mut world);
        }

        let inventory = world.get::<StationInventory>(station).unwrap();

        // Check Output
        let foam_metal = inventory.contents.iter().find(|(r, _)| *r == ResourceType::FoamMetal);
        assert!(foam_metal.is_some());
        assert_eq!(foam_metal.unwrap().1, 1.0);

        // Check Input Consumption
        let metal = inventory.contents.iter().find(|(r, _)| *r == ResourceType::Metal);
        assert!(metal.is_some());
        assert_eq!(metal.unwrap().1, 90.0); // 100 - 10
    }

    #[test]
    fn test_zero_g_factory_halts_without_input() {
        let mut world = setup_world();

        let station = world.spawn((
            Station { station_type: StationType::OrbitalFactory },
            ZeroGFactory {
                production_rate: 1.0,
                input_type: ResourceType::Metal,
                output_type: ResourceType::FoamMetal,
                input_amount: 10.0,
                output_amount: 1.0,
                progress: 0.0,
                max_progress: 10.0,
            },
            StationInventory {
                contents: vec![], // Empty inventory
                capacity: 1000.0,
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(zero_g_production_system);
        schedule.run(&mut world);

        let factory = world.get::<ZeroGFactory>(station).unwrap();
        assert_eq!(factory.progress, 0.0); // Should not progress
    }

    #[test]
    fn test_zero_g_factory_halts_at_capacity() {
        let mut world = setup_world();

        let station = world.spawn((
            Station { station_type: StationType::OrbitalFactory },
            ZeroGFactory {
                production_rate: 1.0,
                input_type: ResourceType::Metal,
                output_type: ResourceType::FoamMetal,
                input_amount: 10.0,
                output_amount: 1.0,
                progress: 9.9, // Almost done
                max_progress: 10.0,
            },
            StationInventory {
                contents: vec![
                    (ResourceType::Metal, 20.0),
                    (ResourceType::FoamMetal, 100.0), // Full of output?
                ],
                capacity: 100.0, // Total capacity reached (20 + 100 > 100 is impossible, but if capacity is checked against sum)
            }
        )).id();

        // Let's say capacity is 110.0, current load is 120.0 (impossible state but useful for test)
        // Or better: capacity 120.0. Current load: Metal 20 + Foam 100 = 120.
        // Trying to add 1.0 Foam -> 121.0 > 120.0 -> Fail.

        // Wait, input decreases by 10, output increases by 1. Net change -9. So technically space frees up!
        // Edge case: Input 0, Output +1 (e.g. solar energy -> antimatter).
        // Let's test "Output Full" specific logic if we track per-resource limits,
        // OR total mass limit where input < output (unlikely for factory).

        // Actually, just test max capacity logic generally.
        // Case: Input (Small Mass) -> Output (Large Mass)? Unlikely.
        // Let's assume standard "Can't add if full" check happens before remove?
        // Or "Remove then Add"?
        // Better test: Capacity 100. Contents: 100 FoamMetal. Input: Infinite external source?
        // No, input must be in inventory.

        // Real test:
        // Capacity 100.
        // Inventory: Metal 10. FoamMetal 90. Total 100.
        // Production: Consumes 10 Metal, Produces 1 FoamMetal.
        // Result: Metal 0, FoamMetal 91. Total 91.
        // This is ALLOWED.

        // Test HALT:
        // Capacity 100.
        // Inventory: FoamMetal 100.
        // Input: None (magic source?) -> No, requires input.

        // Scenario where output blocks:
        // Input is "Energy" (not stored in inventory, stored in battery or infinite).
        // Let's assume input is standard resource for now.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ResourceType` (`src/layer1/resources.rs`)

```rust
pub enum ResourceType {
    // ... existing ...
    FoamMetal,
    PerfectCrystal,
}
```

### 2. Update `StationType` (`src/layer2/station.rs`)

```rust
pub enum StationType {
    Outpost,
    MiningPlatform,
    Shipyard,
    OrbitalFactory, // New
}

impl StationType {
    pub fn cost(&self) -> Vec<(ResourceType, f32)> {
        match self {
            // ...
            Self::OrbitalFactory => vec![
                (ResourceType::Metal, 300.0),
                (ResourceType::Fuel, 100.0),
                (ResourceType::Tools, 50.0),
            ],
        }
    }

    pub fn label(&self) -> &str {
        match self {
            // ...
            Self::OrbitalFactory => "Orbital Factory",
        }
    }

    pub fn char(&self) -> char {
        match self {
            // ...
            Self::OrbitalFactory => '🏭',
        }
    }
}
```

### 3. Create `src/layer2/industry.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone)]
pub struct StationInventory {
    pub contents: Vec<(ResourceType, f32)>,
    pub capacity: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ZeroGFactory {
    pub production_rate: f32, // Progress per tick
    pub input_type: ResourceType,
    pub output_type: ResourceType,
    pub input_amount: f32,
    pub output_amount: f32,
    pub progress: f32,
    pub max_progress: f32,
}

pub fn zero_g_production_system(
    mut query: Query<(&mut ZeroGFactory, &mut StationInventory)>,
) {
    for (mut factory, mut inventory) in query.iter_mut() {
        // 1. Check if we can produce (has input)
        let has_input = inventory.contents.iter().any(|(r, amt)| *r == factory.input_type && *amt >= factory.input_amount);

        if has_input {
            factory.progress += factory.production_rate;

            if factory.progress >= factory.max_progress {
                // Consume Input
                if let Some(stack) = inventory.contents.iter_mut().find(|(r, _)| *r == factory.input_type) {
                    stack.1 -= factory.input_amount;
                }

                // Produce Output
                let mut found = false;
                for (r, amt) in inventory.contents.iter_mut() {
                    if *r == factory.output_type {
                        *amt += factory.output_amount;
                        found = true;
                        break;
                    }
                }
                if !found {
                    inventory.contents.push((factory.output_type, factory.output_amount));
                }

                // Reset
                factory.progress = 0.0;

                // Cleanup empty stacks
                inventory.contents.retain(|(_, amt)| *amt > 0.0);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: `StationInventory` should act like `FleetCargo`. Consider merging logic or using a trait `HasInventory`.
- **System Registration**: Register `zero_g_production_system` in `src/simulation.rs`.
- **UI**: Update Station Inspector to show `StationInventory` and `ZeroGFactory` status.
- **Multiple Recipes**: Factory currently hardcodes one recipe. Consider a `Recipe` struct or component that can be swapped.

## Acceptance Criteria

- [ ] `StationType::OrbitalFactory` added.
- [ ] `ZeroGFactory` component creates `FoamMetal` from `Metal` (or similar).
- [ ] `StationInventory` tracks resources.
- [ ] Tests verify production consumes input and creates output.
- [ ] `cargo test` passes.
