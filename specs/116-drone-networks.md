# 116: Drone Networks

## Overview

Replacing fragile biological workers with cold, hard steel.
This feature introduces **Drone Hubs** and **Drones**.
- **Drone Hub**: A building that spawns and recharges Drones. Consumes Power.
- **Drone**: An automated worker entity. It performs simple tasks (Haul, Clean, Repair) but cannot do skilled labor (Research, Craft).
- **Behavior**: Drones have no Morale or Hunger. They have a `Battery` level. When low, they return to a Hub to recharge.

## Dependencies

- `016` — Utility AI (Action evaluation)
- `042` — Energy System (Power consumption)
- `014` — Hauling Logistics (Haul action)

## RED Phase: Tests First

Write these tests in `src/layer1/drone_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::drone::{Drone, DroneHub, Battery, evaluate_drone_actions_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::energy::{PowerConsumer, PowerSource};

    #[test]
    fn test_drone_component_initialization() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            Battery { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        let battery = world.get::<Battery>(drone).unwrap();
        assert_eq!(battery.current, 100.0);
    }

    #[test]
    fn test_drone_seeks_charge_when_low() {
        let mut world = World::new();
        // Setup Drone with low battery
        let drone = world.spawn((
            Drone,
            Battery { current: 10.0, max: 100.0 }, // 10%
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        // Setup Hub (Charger)
        let hub = world.spawn((
            Building { building_type: BuildingType::DroneHub },
            DroneHub,
            GridPosition { x: 5, y: 5 },
            PowerConsumer { demand: 10.0, active: true },
        )).id();

        // Run evaluation
        evaluate_drone_actions_system(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Charge);
        // Should target the hub
        // (Implementation detail: target might be stored in a Plan or Action data)
    }

    #[test]
    fn test_drone_idle_when_no_task() {
        let mut world = World::new();
        let drone = world.spawn((
            Drone,
            Battery { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        evaluate_drone_actions_system(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Idle);
    }

    #[test]
    fn test_drone_hauls_item() {
        let mut world = World::new();
        // Setup Drone
        let drone = world.spawn((
            Drone,
            Battery { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        // Setup Item to haul
        use crate::layer1::resources::{ResourceItem, ResourceType};
        use crate::layer1::stockpile::Stockpile;

        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Wood, amount: 10.0 },
            GridPosition { x: 2, y: 2 },
        )).id();

        // Setup Stockpile
        world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile::default(),
            GridPosition { x: 5, y: 5 },
        ));

        // Inject dependencies for `evaluate_haul` (mocked or real)
        // For unit test simplicity, we assume `evaluate_drone_actions_system` calls it.
        // We might need to setup UtilityConfig or mock time/resources depending on `evaluate_haul` internals.
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::layer1::utility_ai::UtilityAIBuffer::default());

        evaluate_drone_actions_system(&mut world);

        let action = world.get::<PopAction>(drone).unwrap();
        assert_eq!(action.current, ActionType::Haul);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/drone.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::utility_ai::{ActionType, PopAction};

#[derive(Component, Default)]
pub struct Drone;

#[derive(Component, Default)]
pub struct DroneHub;

#[derive(Component, Clone, Copy, Debug)]
pub struct Battery {
    pub current: f32,
    pub max: f32,
}
```

### 2. Implement Drone AI System

Create a simplified version of `evaluate_actions_system` specifically for Drones.

```rust
// src/layer1/drone.rs

pub fn evaluate_drone_actions_system(world: &mut World) {
    // Similar to evaluate_actions_system but filtered for Drones
    // and simplified logic (Charge > Haul > Repair > Clean > Idle)

    // 1. Collect Drones needing update
    // 2. For each drone:
    //    if Battery < 20% -> Find Hub -> Action::Charge
    //    else -> Check Haul -> Check Repair -> Check Clean -> Action::Idle
    // 3. Commit Action
}
```

### 3. Integrate with Building System

Add `DroneHub` to `BuildingType` enum in `src/layer1/building.rs`.
Update `spawn_building` to add `DroneHub` component.

### 4. Implement Charge Action

Add `ActionType::Charge` to `src/layer1/utility_types.rs`.
Implement `process_charge_system` that increases `Battery.current` when at a Hub.

## REFACTOR Phase: Quality & Design

- **Code Reuse**: `evaluate_haul`, `evaluate_repair`, etc. should be reusable functions that accept a position and return a utility/target, agnostic of whether the agent is a Pop or Drone.
- **Visuals**: Drones need a distinct ASCII char (`d` or `🤖`) and color (Cyan/Grey).
- **Spawn Logic**: `DroneHub` should periodically check if it has a Drone. If not (and power exists), spawn one.
- **Power Drain**: Drones drain battery every tick. `update_battery_system`.

## Acceptance Criteria

- [ ] `Drone` and `DroneHub` components exist.
- [ ] `Battery` component tracks energy.
- [ ] Drones seek `DroneHub` to `Charge` when battery < 20%.
- [ ] Drones perform `Haul` tasks when battery is high.
- [ ] Drones perform `Repair` tasks.
- [ ] Drones do NOT have `Needs` (Hunger/Rest/Leisure).
- [ ] Tests pass.

## Questions

*Builder: add questions here if spec is unclear.*
