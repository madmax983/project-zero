# Specification: 299 - Temporal Smuggling

## 1. Overview
**Layer:** Cross-layer
**Fantasy:** Buying tomorrow with the collateral of yesterday. Time is just another loan shark.
**Mechanic:** A rare "Rift" anomaly allows receiving a cargo pod from the future. You get the resources *now*, but gain a "Temporal Debt". In exactly one year, you must place the exact same resources into the Rift to close the loop.

## 2. Dependencies
- `039-trade-system` (Inventory and Trade flows)
- `145-prototyping-phase` (Handling anomalies)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::inventory::{Inventory, ResourceType};
    use crate::layer1::simulation::{SimulationTime, TICKS_PER_YEAR};
    use crate::layer1::events::TemporalDebt;

    #[test]
    fn test_rift_grants_resources_and_incurs_debt() {
        // Arrange
        let mut world = World::new();
        let colony_inv = world.spawn(Inventory::default()).id();
        let mut sim_time = SimulationTime { current_tick: 0 };
        world.insert_resource(sim_time);

        let rift_entity = world.spawn(TemporalRift { active: true }).id();

        // Act
        open_rift(&mut world, rift_entity, ResourceType::AdvancedMedicine, 50);

        // Assert
        let inv = world.get::<Inventory>(colony_inv).unwrap();
        assert_eq!(inv.get_amount(ResourceType::AdvancedMedicine), 50, "Should receive future resources instantly");

        let mut debt_query = world.query::<&TemporalDebt>();
        let debt = debt_query.iter(&world).next().unwrap();
        assert_eq!(debt.resource_type, ResourceType::AdvancedMedicine);
        assert_eq!(debt.amount, 50);
        assert_eq!(debt.due_tick, TICKS_PER_YEAR, "Debt due in exactly one year");
    }

    #[test]
    fn test_rift_closes_loop_on_payment() {
        // Arrange
        let mut world = World::new();
        let colony_inv = world.spawn(Inventory::with_amount(ResourceType::AdvancedMedicine, 50)).id();
        world.spawn(TemporalDebt {
            resource_type: ResourceType::AdvancedMedicine,
            amount: 50,
            due_tick: 100
        });

        // Act
        pay_temporal_debt(&mut world, colony_inv);

        // Assert
        let inv = world.get::<Inventory>(colony_inv).unwrap();
        assert_eq!(inv.get_amount(ResourceType::AdvancedMedicine), 0, "Resources should be consumed to pay debt");

        let mut debt_query = world.query::<&TemporalDebt>();
        assert_eq!(debt_query.iter(&world).count(), 0, "Debt entity should be despawned");
    }

    #[test]
    fn test_unpaid_debt_causes_paradox() {
        // Arrange
        let mut world = World::new();
        let mut sim_time = SimulationTime { current_tick: 101 };
        world.insert_resource(sim_time);

        world.spawn(TemporalDebt {
            resource_type: ResourceType::AdvancedMedicine,
            amount: 50,
            due_tick: 100
        });

        // Act
        check_temporal_debts(&mut world);

        // Assert
        let mut paradox_query = world.query::<&ParadoxEvent>();
        assert_eq!(paradox_query.iter(&world).count(), 1, "Unpaid debt past due_tick spawns ParadoxEvent");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/tech/temporal_smuggling.rs

use bevy::prelude::*;
use crate::layer1::inventory::{Inventory, ResourceType};
use crate::layer1::simulation::{SimulationTime, TICKS_PER_YEAR};

#[derive(Component)]
pub struct TemporalRift {
    pub active: bool,
}

#[derive(Component)]
pub struct TemporalDebt {
    pub resource_type: ResourceType,
    pub amount: u32,
    pub due_tick: u64,
}

#[derive(Component)]
pub struct ParadoxEvent;

pub fn open_rift(world: &mut World, rift: Entity, res_type: ResourceType, amount: u32) {
    if let Some(mut inv) = world.query::<&mut Inventory>().iter_mut(world).next() {
        inv.add_amount(res_type.clone(), amount);
    }

    let current_tick = world.resource::<SimulationTime>().current_tick;
    world.spawn(TemporalDebt {
        resource_type: res_type,
        amount,
        due_tick: current_tick + TICKS_PER_YEAR as u64,
    });
}

pub fn pay_temporal_debt(world: &mut World, inventory_entity: Entity) {
    let mut paid_debts = Vec::new();

    for (debt_entity, debt) in world.query::<(Entity, &TemporalDebt)>().iter(world) {
        if let Some(mut inv) = world.get_mut::<Inventory>(inventory_entity) {
            if inv.get_amount(debt.resource_type.clone()) >= debt.amount {
                inv.remove_amount(debt.resource_type.clone(), debt.amount);
                paid_debts.push(debt_entity);
            }
        }
    }

    for debt_entity in paid_debts {
        world.despawn(debt_entity);
    }
}

pub fn check_temporal_debts(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().current_tick;
    let mut defaulted_debts = Vec::new();

    for (debt_entity, debt) in world.query::<(Entity, &TemporalDebt)>().iter(world) {
        if current_tick > debt.due_tick {
            defaulted_debts.push(debt_entity);
            world.spawn(ParadoxEvent);
        }
    }

    for debt_entity in defaulted_debts {
        world.despawn(debt_entity);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Handling**: Instead of despawning entities inside `check_temporal_debts`, trigger a Bevy `EventWriter<ParadoxEvent>` to handle the catastrophic paradox mechanics independently.
- **Inventory Check**: Ensure `Inventory::remove_amount` checks if enough is present before deducing, though the `if inv.get_amount(...) >= ...` check acts as a safeguard.
- **Paradox Outcome**: Hook `ParadoxEvent` into the building destruction or map scrambling systems to enact the punishment for defaulting on a temporal debt.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Temporal Rifts grant resources immediately and incur debt due in `TICKS_PER_YEAR`.
- [ ] Paying debts removes the required resources and closes the loop.
- [ ] Failing to pay a debt spawns a Paradox.

## 7. Technical Guidance
- `TICKS_PER_YEAR` is 1000 per the architect rules.
- Integrating this with `ratatui` UI requires exposing `TemporalDebt` queries to the `ui` module so players can see the countdown to their debt.

## 8. Questions
*Builder: add questions here if spec is unclear.*
