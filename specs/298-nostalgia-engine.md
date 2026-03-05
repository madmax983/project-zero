# Specification: 298 - The Nostalgia Engine

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Selling the feeling of Earth to a galaxy that has lost its way.
**Mechanic:** Construct a massive broadcast dish that transmits "Earth-Normal" frequencies and pre-collapse media. It drains enormous power but attracts wealthy "Pilgrim" ships from Layer 3 who pay exorbitant fees just to park in orbit and listen.

## 2. Dependencies
- `286-great-works` (Multi-phase construction framework)
- `152-orbital-stations` (Layer 2 orbit presence and ships)
- `039-trade-system` (Credits and Economy)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::construction::great_works::{GreatWork, OperationalGreatWork};
    use crate::layer2::orbit::{Orbit, PilgrimShip};
    use crate::layer1::economy::Credits;
    use crate::layer1::power::{PowerGrid, PowerConsumer};

    #[test]
    fn test_nostalgia_engine_drains_power() {
        // Arrange
        let mut world = World::new();
        let engine = world.spawn((
            OperationalGreatWork { id: "nostalgia_engine".to_string() },
            PowerConsumer { consumption: 500.0, active: true },
        )).id();

        let mut power_grid = PowerGrid { total_power: 1000.0, ..Default::default() };
        world.insert_resource(power_grid);

        // Act
        process_nostalgia_engine_power(&mut world);

        // Assert
        let grid = world.resource::<PowerGrid>();
        assert_eq!(grid.total_power, 500.0, "Engine should drain massive power");
    }

    #[test]
    fn test_nostalgia_engine_spawns_pilgrims() {
        // Arrange
        let mut world = World::new();
        let engine = world.spawn((
            OperationalGreatWork { id: "nostalgia_engine".to_string() },
            PowerConsumer { consumption: 500.0, active: true },
        )).id();

        // Act
        // Tick engine logic to spawn ships
        tick_nostalgia_engine(&mut world, 100);

        // Assert
        let mut query = world.query::<&PilgrimShip>();
        let pilgrim_count = query.iter(&world).count();
        assert!(pilgrim_count > 0, "Active engine should attract pilgrims");
    }

    #[test]
    fn test_pilgrims_pay_parking_fees() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(Credits { amount: 0 });
        let ship = world.spawn(PilgrimShip { parked: true, fee_rate: 100 }).id();

        // Act
        process_pilgrim_fees(&mut world);

        // Assert
        let credits = world.resource::<Credits>();
        assert_eq!(credits.amount, 100, "Parked pilgrims should pay fees");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/tech/nostalgia_engine.rs

use bevy::prelude::*;
use crate::layer1::construction::great_works::OperationalGreatWork;
use crate::layer1::power::{PowerGrid, PowerConsumer};
use crate::layer2::orbit::PilgrimShip;
use crate::layer1::economy::Credits;

pub fn process_nostalgia_engine_power(world: &mut World) {
    let mut query = world.query::<(&OperationalGreatWork, &PowerConsumer)>();
    let total_drain: f32 = query.iter(world)
        .filter(|(work, consumer)| work.id == "nostalgia_engine" && consumer.active)
        .map(|(_, consumer)| consumer.consumption)
        .sum();

    if let Some(mut grid) = world.get_resource_mut::<PowerGrid>() {
        grid.total_power -= total_drain;
    }
}

pub fn tick_nostalgia_engine(world: &mut World, tick: u64) {
    let has_active_engine = {
        let mut query = world.query::<(&OperationalGreatWork, &PowerConsumer)>();
        query.iter(world).any(|(work, consumer)| work.id == "nostalgia_engine" && consumer.active)
    };

    if has_active_engine && tick % 100 == 0 {
        world.spawn(PilgrimShip { parked: true, fee_rate: 100 });
    }
}

pub fn process_pilgrim_fees(world: &mut World) {
    let mut query = world.query::<&PilgrimShip>();
    let total_fees: u32 = query.iter(world)
        .filter(|ship| ship.parked)
        .map(|ship| ship.fee_rate)
        .sum();

    if let Some(mut credits) = world.get_resource_mut::<Credits>() {
        credits.amount += total_fees;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **System Registration**: Move the ticking logic to Bevy `FixedUpdate` schedule instead of passing raw tick counts.
- **Power Coupling**: Instead of manually subtracting power from the `PowerGrid` resource in a separate function, rely on the main `power_consumption_system` in the power module. The `Nostalgia Engine` should just toggle its `active` state if the grid goes negative.
- **Debris Shadow Generation**: Add logic where dense clusters of Pilgrim Ships cast Debris Shadows over Layer 1 tiles, affecting solar arrays.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Nostalgia Engine functions as an `OperationalGreatWork`.
- [ ] It drains massive power.
- [ ] Active engine generates `PilgrimShip` entities in orbit which pay passive Credits.

## 7. Technical Guidance
- The Nostalgia Engine should be implemented as an extension of the `GreatWork` system, built via `ConstructionPhase`.
- Pilgrim Ships should exist in Layer 2, so they require mapping logic if interacting with ground features (like generating shadows).

## 8. Questions
*Builder: add questions here if spec is unclear.*
