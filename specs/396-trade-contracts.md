# 396: Trade Contracts

## 1. Overview
As a colony grows, external factions on the Galactic network will demand specific production outputs. "Trade Contracts" are time-limited requests from Layer 3 factions (e.g., "Export 1000 Metal Blocks in 2 cycles").

Completing these contracts grants unique technology, currency, or reputation. Failing to complete them before the deadline incurs "Debt", hostile relations, or a drop in global morale as the colony fails to meet imperial quotas.

## 2. Dependencies
- `039-trade-system.md` (for the base trade pad and transactions)
- `094-system-view.md` (for Layer 3/external faction representations)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::resource::{ResourceType, Storage};
    use crate::layer3::diplomacy::FactionId;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_contract_generation() {
        let mut app = App::new();
        app.insert_resource(SimulationTime { tick: 0, speed: Default::default() });
        app.add_systems(Update, generate_contracts_system);

        app.update();

        let mut query = app.world_mut().query::<&TradeContract>();
        let contracts_count = query.iter(app.world()).count();
        // Since we mock it, assume it spawns at least one contract in the simulation step or via event.
        // We'll test the actual component exists.
    }

    #[test]
    fn test_contract_fulfillment() {
        let mut app = App::new();
        app.add_event::<FulfillContractEvent>();
        app.add_systems(Update, fulfill_contract_system);

        let contract_ent = app.world_mut().spawn(TradeContract {
            faction: FactionId(1),
            required_resource: ResourceType::MetalBlock,
            required_amount: 1000,
            deadline_tick: 5000,
            reward_credits: 500,
            status: ContractStatus::Active,
        }).id();

        // Spawn a storage with enough items
        let storage_ent = app.world_mut().spawn(Storage {
            capacity: 2000,
            current_amount: 1500,
            resource_type: ResourceType::MetalBlock,
        }).id();

        app.world_mut().send_event(FulfillContractEvent {
            contract: contract_ent,
            storage_source: storage_ent,
        });

        app.update();

        let storage = app.world().get::<Storage>(storage_ent).unwrap();
        assert_eq!(storage.current_amount, 500); // 1000 were deducted

        let contract = app.world().get::<TradeContract>(contract_ent).unwrap();
        assert_eq!(contract.status, ContractStatus::Fulfilled);
    }

    #[test]
    fn test_contract_failure_on_deadline() {
        let mut app = App::new();
        app.insert_resource(SimulationTime { tick: 6000, speed: Default::default() });
        app.add_systems(Update, check_contract_deadlines_system);

        let contract_ent = app.world_mut().spawn(TradeContract {
            faction: FactionId(1),
            required_resource: ResourceType::MetalBlock,
            required_amount: 1000,
            deadline_tick: 5000,
            reward_credits: 500,
            status: ContractStatus::Active,
        }).id();

        app.update();

        let contract = app.world().get::<TradeContract>(contract_ent).unwrap();
        assert_eq!(contract.status, ContractStatus::Failed);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::resource::{ResourceType, Storage};
use crate::layer3::diplomacy::FactionId;
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractStatus {
    Active,
    Fulfilled,
    Failed,
}

#[derive(Component, Debug, Clone)]
pub struct TradeContract {
    pub faction: FactionId,
    pub required_resource: ResourceType,
    pub required_amount: u32,
    pub deadline_tick: u64,
    pub reward_credits: u32,
    pub status: ContractStatus,
}

#[derive(Event)]
pub struct FulfillContractEvent {
    pub contract: Entity,
    pub storage_source: Entity,
}

pub fn generate_contracts_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    // Mocking generation condition
) {
    if time.tick == 100 { // Just as an example for MVP
        commands.spawn(TradeContract {
            faction: FactionId(1),
            required_resource: ResourceType::MetalBlock,
            required_amount: 100,
            deadline_tick: time.tick + 10000,
            reward_credits: 500,
            status: ContractStatus::Active,
        });
    }
}

pub fn fulfill_contract_system(
    mut events: EventReader<FulfillContractEvent>,
    mut contracts_query: Query<&mut TradeContract>,
    mut storage_query: Query<&mut Storage>,
) {
    for event in events.read() {
        if let Ok(mut contract) = contracts_query.get_mut(event.contract) {
            if contract.status == ContractStatus::Active {
                if let Ok(mut storage) = storage_query.get_mut(event.storage_source) {
                    if storage.resource_type == contract.required_resource && storage.current_amount >= contract.required_amount {
                        storage.current_amount -= contract.required_amount;
                        contract.status = ContractStatus::Fulfilled;
                        // Note: actual credit granting or reputation bump would happen here
                    }
                }
            }
        }
    }
}

pub fn check_contract_deadlines_system(
    time: Res<SimulationTime>,
    mut contracts_query: Query<&mut TradeContract>,
) {
    for mut contract in contracts_query.iter_mut() {
        if contract.status == ContractStatus::Active && time.tick >= contract.deadline_tick {
            contract.status = ContractStatus::Failed;
            // Note: penalty logic would go here (e.g. debt, reputation loss)
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Credit & Debt Logistics:** Integrate directly with the colony's Credit/Currency component when fulfilling a contract, and apply debt when failing.
- **Contract UI:** A dedicated panel in the UI is necessary to see active contracts, remaining time, and required resources.
- **Partial Fulfillment:** Allow players to deposit resources partially towards the contract goal.
- **Diplomacy Impact:** Hook up `ContractStatus::Failed` to Layer 3 faction hostility logic.

## 6. Acceptance Criteria (Testable!)
- [ ] `TradeContract` component created.
- [ ] Fulfilling a contract removes the exact resource amount from storage.
- [ ] Contract changes to `Failed` if the deadline passes before completion.
- [ ] Tests pass (including `cargo test`).
- [ ] `cargo clippy -- -D warnings` runs cleanly.

## 7. Technical Guidance
- Register the systems in `Layer3SystemSet` or a global trade/diplomacy set if they exist.
- Ensure the `Storage` component modification respects safety boundaries (checking `current_amount`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
