# 134: Emergency Venting

## Overview

Adds manual control to Doors and Airlocks, allowing players to Vent rooms to vacuum. This serves as an emergency measure to extinguish fires or purge hazardous gases, at the cost of losing atmosphere and potentially damaging contents.

**Why:**
- Adds "Button mashing" panic moments (Tension).
- Connects Pressure (`119`) and Fire (`033`) systems.
- Provides a high-risk/high-reward tool for crisis management.

## Dependencies

- `119` — Airlock & Pressure (Implemented)
- `033` — Fire Propagation (Implemented)
- `007` — Housing (for Doors/Gates)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/venting_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pressure::{PressureGrid, update_pressure_system};
    use crate::layer1::fire::{Fire, fire_damage_system}; // Assumes fire_damage_system handles extinguishing
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;

    // New Component
    use crate::layer1::control::{DoorControl, DoorState};

    #[test]
    fn test_door_control_defaults_to_auto() {
        let control = DoorControl::default();
        assert_eq!(control.state, DoorState::Auto);
    }

    #[test]
    fn test_open_airlock_vents_pressure() {
        let mut world = World::new();
        // Setup Pressure Grid: (1,0) is High Pressure, (3,0) is Vacuum
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Spawn Airlock at (2,0) set to OPEN
        world.spawn((
            Building { building_type: BuildingType::Airlock },
            GridPosition { x: 2, y: 0 },
            DoorControl { state: DoorState::Open }, // Forced Open
            Structure::default(),
        ));

        // Run pressure update
        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        // Pressure should diffuse past the airlock because it is open
        assert!(grid.get(3, 0) > 0.05, "Pressure should vent through OPEN airlock");
    }

    #[test]
    fn test_locked_airlock_maintains_pressure() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Spawn Airlock at (2,0) set to LOCKED (Closed)
        world.spawn((
            Building { building_type: BuildingType::Airlock },
            GridPosition { x: 2, y: 0 },
            DoorControl { state: DoorState::Locked },
            Structure::default(),
        ));

        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(3, 0) < 0.01, "Pressure should NOT vent through LOCKED airlock");
    }

    #[test]
    fn test_fire_extinguishes_in_vacuum() {
        let mut world = World::new();

        // Setup Pressure Grid with Vacuum at (5,5)
        let mut grid = PressureGrid::new(10, 10);
        grid.set(5, 5, 0.0); // Vacuum
        world.insert_resource(grid);

        // Spawn Fire at (5,5)
        let fire_entity = world.spawn((
            Fire { lifetime: 50, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run fire damage/update system
        // Note: You may need to update fire_damage_system or create a new fire_pressure_system
        crate::layer1::fire::fire_pressure_check_system(&mut world);

        // Fire should be extinguished (despawned) immediately due to lack of oxygen/pressure
        assert!(world.get_entity(fire_entity).is_err(), "Fire should die in vacuum");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `DoorControl` (`src/layer1/control.rs`)

```rust
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DoorControl {
    pub state: DoorState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DoorState {
    #[default]
    Auto,   // Normal behavior (opens for pathing, closed otherwise)
    Open,   // Forced open (vents pressure)
    Locked, // Forced closed (blocks pathing & pressure)
}
```

### 2. Update `spawn_building` (`src/layer1/building.rs`)

Add `DoorControl` component to `BuildingType::Airlock` and `BuildingType::Gate`.

### 3. Update `PressureGrid` Logic (`src/layer1/pressure.rs`)

Modify `update_pressure_system` to check `DoorControl`.

```rust
// Inside the loop that builds `blockers` map:
for (b, pos, control) in world.query::<(&Building, &GridPosition, Option<&DoorControl>)>().iter(world) {
    let mut transmissivity = match b.building_type {
        BuildingType::Wall => 0.0,
        BuildingType::Airlock => 0.0,
        BuildingType::Gate => 0.5,
        _ => 1.0,
    };

    // Override based on control
    if let Some(ctrl) = control {
        match ctrl.state {
            DoorState::Open => transmissivity = 1.0, // Fully open
            DoorState::Locked => transmissivity = 0.0, // Fully closed
            DoorState::Auto => {}, // Default behavior
        }
    }

    // ... insert into blockers
}
```

### 4. Fire Pressure Check (`src/layer1/fire.rs`)

Add a system to kill fire in low pressure.

```rust
pub fn fire_pressure_check_system(world: &mut World) {
    let pressure = world.resource::<PressureGrid>();
    let mut to_despawn = Vec::new();

    for (entity, pos) in world.query::<(Entity, &GridPosition)>().with::<Fire>().iter(world) {
        if pressure.get(pos.x, pos.y) < 0.1 { // Threshold for combustion
            to_despawn.push(entity);
        }
    }

    for e in to_despawn {
        world.despawn(e);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Add `fire_pressure_check_system` to `SimulationSchedule`.
- **UI**: (Future) Add buttons in `Inspector` to toggle `DoorState`.
- **Visuals**: Open doors should render differently (e.g., empty space or specific char).
- **Sound**: Venting sound effect when pressure drops rapidly.

## Acceptance Criteria

- [ ] `DoorControl` component exists.
- [ ] Airlocks/Gates spawn with `DoorControl::Auto`.
- [ ] Forcing `DoorState::Open` allows pressure to equalize (venting).
- [ ] Forcing `DoorState::Locked` blocks pressure.
- [ ] Fire is extinguished when tile pressure < 0.1.
- [ ] `cargo test` passes.

## Technical Guidance

- Be careful with `Option<&DoorControl>` in queries; standard buildings won't have it.
- Ensure `fire_pressure_check_system` runs *before* `fire_spread_system` to prevent spreading from a dying fire.
