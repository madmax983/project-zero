# Company Scrip

## 1. Overview
**Layer:** 1
**Fantasy:** You owe your soul to the company store.
**Mechanic:** Pops earn "Credits" (Scrip) for work. Food/Luxuries have prices. Pops buy what they can afford.
**Emergence:** High-skill miners live like kings; haulers starve. A black market emerges for cheap, low-quality goods.
**Tension:** Equitable pay (socialist harmony) vs. Merit-based pay (productivity incentive).

## 2. Dependencies
- `009-job-system` (Pops working jobs)
- `005-pop-needs` (Pops satisfying hunger/leisure)
- `039-trade-system` (Item values/pricing concepts)

## 3. RED Phase: Tests First

```rust
// specs/415-company-scrip.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Needs};
    use crate::layer1::job::{JobEvent, JobType};
    use crate::layer1::economy::{Wallet, Store, ScripSystem};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Store>();
        app.add_systems(Update, (
            crate::layer1::economy::pay_wages_system,
            crate::layer1::economy::purchase_goods_system,
        ));
        app
    }

    #[test]
    fn test_pop_earns_scrip_from_work() {
        // Arrange
        let mut app = setup_app();
        let pop_id = app.world_mut().spawn((
            Pop,
            Wallet { balance: 0 },
        )).id();

        // Act
        app.world_mut().send_event(JobEvent::Completed {
            worker: pop_id,
            job_type: JobType::Mining,
        });
        app.update();

        // Assert: Pop should have earned standard mining wages
        let wallet = app.world().get::<Wallet>(pop_id).unwrap();
        assert_eq!(wallet.balance, 10); // Example base wage
    }

    #[test]
    fn test_pop_spends_scrip_for_food() {
        let mut app = setup_app();

        // Setup store with food priced at 5
        let mut store = app.world_mut().resource_mut::<Store>();
        store.set_price("food_ration", 5);

        let pop_id = app.world_mut().spawn((
            Pop,
            Wallet { balance: 10 }, // Has enough to buy 1 ration
            Needs { hunger: 0.1, ..Default::default() },
        )).id();

        // Act
        app.world_mut().send_event(crate::layer1::economy::PurchaseAttemptEvent {
            buyer: pop_id,
            item: "food_ration".to_string(),
        });
        app.update();

        // Assert: Wallet reduced, Need fulfilled
        let wallet = app.world().get::<Wallet>(pop_id).unwrap();
        assert_eq!(wallet.balance, 5);
        let needs = app.world().get::<Needs>(pop_id).unwrap();
        assert_eq!(needs.hunger, 1.0); // Hunger filled
    }

    #[test]
    fn test_pop_starves_without_scrip() {
        let mut app = setup_app();

        let mut store = app.world_mut().resource_mut::<Store>();
        store.set_price("food_ration", 5);

        let pop_id = app.world_mut().spawn((
            Pop,
            Wallet { balance: 0 }, // Cannot afford!
            Needs { hunger: 0.1, ..Default::default() },
        )).id();

        // Act
        app.world_mut().send_event(crate::layer1::economy::PurchaseAttemptEvent {
            buyer: pop_id,
            item: "food_ration".to_string(),
        });
        app.update();

        // Assert: Wallet 0, Hunger unchanged (purchase failed)
        let wallet = app.world().get::<Wallet>(pop_id).unwrap();
        assert_eq!(wallet.balance, 0);
        let needs = app.world().get::<Needs>(pop_id).unwrap();
        assert_eq!(needs.hunger, 0.1); // Still starving
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/economy.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Needs};
use crate::layer1::job::{JobEvent, JobType};
use std::collections::HashMap;

#[derive(Component, Default)]
pub struct Wallet {
    pub balance: u32,
}

#[derive(Resource, Default)]
pub struct Store {
    pub prices: HashMap<String, u32>,
}

impl Store {
    pub fn set_price(&mut self, item: &str, price: u32) {
        self.prices.insert(item.to_string(), price);
    }
}

#[derive(Event)]
pub struct PurchaseAttemptEvent {
    pub buyer: Entity,
    pub item: String,
}

pub fn pay_wages_system(
    mut events: EventReader<JobEvent>,
    mut wallets: Query<&mut Wallet>,
) {
    for event in events.read() {
        if let JobEvent::Completed { worker, job_type } = event {
            if let Ok(mut wallet) = wallets.get_mut(*worker) {
                // Determine wage based on job type. Magic numbers for MVP.
                let wage = match job_type {
                    JobType::Mining => 10,
                    JobType::Farming => 5,
                    _ => 1,
                };
                wallet.balance += wage;
            }
        }
    }
}

pub fn purchase_goods_system(
    mut events: EventReader<PurchaseAttemptEvent>,
    mut query: Query<(&mut Wallet, &mut Needs)>,
    store: Res<Store>,
) {
    for event in events.read() {
        if let Ok((mut wallet, mut needs)) = query.get_mut(event.buyer) {
            if let Some(&price) = store.prices.get(&event.item) {
                if wallet.balance >= price {
                    wallet.balance -= price;

                    // Specific logic for applying the item's benefit
                    if event.item == "food_ration" {
                        needs.hunger = 1.0;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Prices and Inventory:** The `Store` currently just has infinite stock and arbitrary prices. It should be linked to the colony's actual stockpiles (Layer 1 Logistics).
- **Utility AI Integration:** The `evaluate_actions_system` needs to be updated. If a Pop cannot afford food, the utility score for "Eat" should plummet, and "Find Work" or "Steal" should skyrocket.
- **Inflation/Policy:** Add a `ColonyEdict` (from Spec 054) allowing the player to adjust the base wage or set price caps.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops successfully earn credits when `JobEvent::Completed` triggers.
- [ ] Pops can purchase goods if they have sufficient credits, satisfying their needs.
- [ ] Purchasing fails if the Pop lacks funds.

## 7. Technical Guidance
- Create a new module `src/layer1/economy.rs`.
- Ensure the Utility AI properly considers `Wallet` balance before committing to a purchase action to prevent a loop of failed attempts.
- In a real implementation, the item string should probably be replaced with the actual `ItemType` enum or ID system used elsewhere in the codebase.

## 8. Questions
*Builder: add questions here if spec is unclear.*
