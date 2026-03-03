# 157: Ship Classes & Construction

## 1. Overview

Currently, **Fleets** are abstract entities with generic "Cargo Capacity" and "Speed".
To deepen the Layer 2 gameplay, we introduce distinct **Ship Classes** (e.g., *Transport*, *Miner*, *Scout*).
Fleets become collections of these ships. A fleet's stats are derived from its composition:
- **Speed**: Determined by the slowest ship.
- **Cargo**: Sum of all ships' cargo holds.
- **Mining/Combat**: Sum of respective modules (future).

This spec also introduces the **Shipyard** building (Layer 1) which constructs **Ship Items**. These items are then used by the **Launch Pad** (Spec 105) to form fleets.

## 2. Dependencies

- `specs/099-fleet-movement.md` (Fleet Entity)
- `specs/105-launch-logistics.md` (Launch Pad)
- `specs/104-fuel-industry.md` (Fuel Resource)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/ship_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetComposition};
    use crate::layer2::ship::{Ship, ShipType};
    use crate::layer1::resources::ResourceType;

    #[test]
    fn test_ship_type_stats() {
        // Scout: Fast, Low Cargo
        let scout = ShipType::Scout;
        assert_eq!(scout.base_speed(), 2.0);
        assert_eq!(scout.cargo_capacity(), 10.0);

        // Transport: Slow, High Cargo
        let transport = ShipType::Transport;
        assert_eq!(transport.base_speed(), 0.5);
        assert_eq!(transport.cargo_capacity(), 1000.0);

        // Miner: Medium Speed, Medium Cargo, Mining Ability
        let miner = ShipType::Miner;
        assert_eq!(miner.base_speed(), 0.8);
        assert_eq!(miner.cargo_capacity(), 200.0);
    }

    #[test]
    fn test_fleet_composition_calculates_total_cargo() {
        let mut comp = FleetComposition::default();
        comp.add_ship(Ship::new(ShipType::Scout));     // 10
        comp.add_ship(Ship::new(ShipType::Transport)); // 1000

        assert_eq!(comp.total_cargo_capacity(), 1010.0);
    }

    #[test]
    fn test_fleet_composition_calculates_min_speed() {
        let mut comp = FleetComposition::default();
        comp.add_ship(Ship::new(ShipType::Scout));     // 2.0
        comp.add_ship(Ship::new(ShipType::Transport)); // 0.5

        // Fleet moves at speed of slowest ship
        assert_eq!(comp.speed(), 0.5);
    }

    #[test]
    fn test_fleet_composition_empty_has_default_speed() {
        let comp = FleetComposition::default();
        // Empty fleet (e.g., just a probe?) or invalid?
        // Let's say 0.0 or 1.0.
        // Logic: An empty fleet shouldn't exist, but if it does, it has no speed constraints?
        // Or 0.0 to prevent movement.
        assert_eq!(comp.speed(), 0.0);
    }

    #[test]
    fn test_ship_construction_cost() {
        let scout = ShipType::Scout;
        let cost = scout.construction_cost();

        // Check contents
        assert!(cost.contains(&(ResourceType::Metal, 50.0)));
        assert!(cost.contains(&(ResourceType::Fuel, 20.0))); // Initial fueling? Or construction energy?
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define `ShipType` Enum (`src/layer2/ship.rs`)

```rust
use crate::layer1::resources::ResourceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShipType {
    Scout,
    Transport,
    Miner,
    Frigate,
}

impl ShipType {
    pub fn base_speed(&self) -> f32 {
        match self {
            Self::Scout => 2.0,
            Self::Frigate => 1.5,
            Self::Miner => 0.8,
            Self::Transport => 0.5,
        }
    }

    pub fn cargo_capacity(&self) -> f32 {
        match self {
            Self::Scout => 10.0,
            Self::Frigate => 50.0,
            Self::Miner => 200.0,
            Self::Transport => 1000.0,
        }
    }

    pub fn construction_cost(&self) -> Vec<(ResourceType, f32)> {
        match self {
            Self::Scout => vec![(ResourceType::Metal, 50.0), (ResourceType::Fuel, 20.0)],
            Self::Transport => vec![(ResourceType::Metal, 200.0), (ResourceType::Fuel, 50.0)],
            Self::Miner => vec![(ResourceType::Metal, 100.0), (ResourceType::Fuel, 30.0)],
            Self::Frigate => vec![(ResourceType::Metal, 150.0), (ResourceType::Fuel, 40.0)],
        }
    }
}
```

### 2. Define `Ship` Struct (`src/layer2/ship.rs`)

```rust
#[derive(Debug, Clone)]
pub struct Ship {
    pub ship_type: ShipType,
    pub health: f32,
    pub max_health: f32,
}

impl Ship {
    pub fn new(ship_type: ShipType) -> Self {
        Self {
            ship_type,
            health: 100.0, // Default for now
            max_health: 100.0,
        }
    }
}
```

### 3. Define `FleetComposition` Component (`src/layer2/fleet.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer2::ship::Ship;

#[derive(Component, Debug, Clone, Default)]
pub struct FleetComposition {
    pub ships: Vec<Ship>,
}

impl FleetComposition {
    pub fn add_ship(&mut self, ship: Ship) {
        self.ships.push(ship);
    }

    pub fn total_cargo_capacity(&self) -> f32 {
        self.ships.iter().map(|s| s.ship_type.cargo_capacity()).sum()
    }

    pub fn speed(&self) -> f32 {
        if self.ships.is_empty() {
            return 0.0;
        }
        // Minimal speed of all ships
        self.ships.iter()
            .map(|s| s.ship_type.base_speed())
            .fold(f32::INFINITY, |a, b| a.min(b))
    }
}
```

### 4. Integration with `LaunchPad`

Update `LaunchOrder` (from Spec 105) to accept `Vec<ShipType>` instead of just `fuel_cost`.

```rust
// src/layer1/launch.rs
pub struct LaunchOrder {
    pub ships: Vec<ShipType>,
    pub cargo: Vec<(ResourceType, f32)>,
}
```

The `launch_system` will then:
1.  Calculate total fuel cost based on ship types + cargo weight (future).
2.  Spawn `Fleet` entity.
3.  Add `FleetComposition` with requested ships.
4.  Add `FleetCargo` with requested resources.

## 5. REFACTOR Phase: Quality & Design

-   **Ship Names**: Add `name: String` to `Ship` struct for flavor (e.g., "The Indestructible II").
-   **Fuel Efficiency**: Different engines? (Future).
-   **Shipyard Building**: Currently assumed to be part of LaunchPad or separate.
    -   *Design Decision*: `Shipyard` (Layer 1) constructs `ShipItem`s. `LaunchPad` takes `ShipItem`s from storage to launch.
    -   *Refactor*: Move construction logic to `Shipyard` system later. For MVP, `LaunchPad` can "construct and launch" instantly if resources are available.
-   **Visuals**: Fleet icon on map should reflect composition (e.g., if mostly Combat ships, use Triangle; if Transport, use Square).

## 6. Acceptance Criteria (Testable!)

- [ ] `ShipType` enum defined with stats.
- [ ] `FleetComposition` component implemented.
- [ ] `FleetComposition::speed()` returns min speed.
- [ ] `FleetComposition::total_cargo_capacity()` returns sum.
- [ ] Tests pass.
- [ ] Integration with `Fleet` entity in Layer 2.

## 7. Technical Guidance

-   Keep `Ship` struct lightweight. If we need detailed damage models per ship later, we can turn `Ship` into an Entity and use `Children` relationship, but for thousands of ships, a `Vec<Ship>` struct inside `Fleet` component is more performant for ECS.
-   Use `f32::INFINITY` for initial fold accumulator for speed, but handle empty case.

## 8. Questions

-   *Builder: Do ships have individual fuel?*
-   *Architect:* Yes, each ship tracks its own fuel tank.
-   *Builder: Can I split a fleet?*
-   *Architect:* Yes, fleets can be split into multiple smaller fleets on the same tile.
