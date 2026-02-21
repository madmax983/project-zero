# 194 Company Scrip

## 1. Overview

**Fantasy:** "You owe your soul to the company store."

Pops are no longer passive consumers of resources; they are participants in a closed economy. They earn "Company Scrip" (Credits) for completing jobs and spend it to fulfill needs (Food, Luxuries). This introduces a new tension: **Equitable Pay** vs **Merit-Based Pay**. High-value jobs pay more, leading to wealth stratification.

**Why:** Adds economic depth. Allows for "Poverty" as a failure state for individual pops, not just the colony.

## 2. Dependencies

- `005` Pop Needs (Hunger)
- `006` Job Assignment (Job completion)
- `008` Resource Stockpiles (Food consumption)

## 3. RED Phase: Tests First

Write these tests in `src/layer1/economy_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::job::Job;
    use crate::layer1::needs::Needs;
    use crate::layer2::farm::ColonyResources;

    #[test]
    fn test_wallet_default() {
        let wallet = Wallet::default();
        assert_eq!(wallet.credits, 0.0);
    }

    #[test]
    fn test_pay_wages_system() {
        let mut world = World::new();
        // Spawn a pop with a wallet
        let pop = world.spawn((Pop, Wallet { credits: 10.0 })).id();

        // Simulate job completion event
        // (In real impl, this would be an event reader. For test, we might call a function directly)
        // Let's assume a helper function `pay_wage(&mut world, pop, amount)`

        pay_wage(&mut world, pop, 5.0);

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 15.0);
    }

    #[test]
    fn test_purchase_food_success() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Wallet { credits: 10.0 },
            Needs { hunger: 0.5, rest: 0.5 }
        )).id();

        world.insert_resource(ColonyPrices { food_price: 2.0 });
        world.insert_resource(ColonyResources { food: 10.0 });

        // Call modified consumption system
        consume_food_with_payment_system(&mut world);

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 8.0, "Should deduct 2.0 credits");

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.5, "Should have eaten");
    }

    #[test]
    fn test_purchase_food_fail_poverty() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Wallet { credits: 1.0 }, // Not enough!
            Needs { hunger: 0.1, rest: 0.5 } // Starving
        )).id();

        world.insert_resource(ColonyPrices { food_price: 2.0 });
        world.insert_resource(ColonyResources { food: 10.0 });

        consume_food_with_payment_system(&mut world);

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 1.0, "Should not deduct if purchase failed");

        let needs = world.get::<Needs>(pop).unwrap();
        assert_eq!(needs.hunger, 0.1, "Should NOT have eaten (too poor)");
    }

    #[test]
    fn test_job_wage_configuration() {
        // Ensure jobs have a wage associated
        let job = JobType::Mining; // Assume enum
        assert!(get_wage_for_job(job) > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. `Wallet` Component

```rust
// src/layer1/economy.rs

use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Wallet {
    pub credits: f32,
}
```

### 2. `ColonyPrices` Resource

```rust
#[derive(Resource, Debug, Clone, Copy)]
pub struct ColonyPrices {
    pub food_price: f32,
    pub luxury_price: f32,
}

impl Default for ColonyPrices {
    fn default() -> Self {
        Self {
            food_price: 1.0,   // 1 credit per meal
            luxury_price: 5.0,
        }
    }
}
```

### 3. Wage Logic

```rust
// src/layer1/job.rs (or economy.rs)

pub fn get_wage_for_job(job_type: JobType) -> f32 {
    match job_type {
        JobType::Mining => 2.0,
        JobType::Farming => 1.5,
        JobType::Hauling => 1.0,
        JobType::Research => 3.0,
        _ => 1.0,
    }
}

pub fn pay_wage(world: &mut World, worker: Entity, amount: f32) {
    if let Some(mut wallet) = world.get_mut::<Wallet>(worker) {
        wallet.credits += amount;
    }
}
```

### 4. Update Consumption System

Update `consume_food_system` in `src/layer1/farm.rs` (or `economy.rs` if moved).

```rust
pub fn consume_food_with_payment_system(world: &mut World) {
    let price = world.resource::<ColonyPrices>().food_price;
    let mut food_store = world.resource_mut::<ColonyResources>().food;

    // We need to query Needs AND Wallet
    let mut query = world.query::<(Entity, &mut Needs, &mut Wallet)>();

    for (entity, mut needs, mut wallet) in query.iter_mut(world) {
        if needs.hunger < 0.7 { // Hungry
            if food_store >= 1.0 {
                if wallet.credits >= price {
                    // Transaction
                    wallet.credits -= price;
                    food_store -= 1.0;

                    // Eat
                    needs.hunger = (needs.hunger + 0.3).min(1.0);
                } else {
                    // Too poor to eat!
                    // Add "Poverty" thought here later
                }
            }
        }
    }

    world.resource_mut::<ColonyResources>().food = food_store;
}
```

## 5. REFACTOR Phase: Quality & Design

-   **Welfare**: Implement a "Soup Kitchen" edict that sets `food_price = 0.0` but lowers food quality (if quality exists).
-   **Debt**: Allow `Wallet` to go negative up to a limit (Credit Score), charging interest.
-   **Wage Inflation**: If labor is scarce, wages should rise (Market Logic).
-   **UI**: Show Wallet balance in Pop Inspector.

## 6. Acceptance Criteria

- [ ] `Wallet` component added to Pops.
- [ ] `ColonyPrices` resource controls cost of living.
- [ ] Pops earn credits when completing jobs.
- [ ] Pops consume credits when eating.
- [ ] Pops failing to afford food do *not* eat (starvation risk).
- [ ] Tests pass.

## 7. Technical Guidance

-   **Integration**: Hook `pay_wage` into `work_execution_system` where the job progress reaches 100%.
-   **Migration**: Existing save files need `Wallet { credits: 0.0 }` inserted for all Pops.
-   **Balance**: Ensure starting wages > starting food price, or the colony dies immediately.

## 8. Questions

-   **Q**: What about "Free" resources like air?
    -   **A**: Air is free. Water might not be.
-   **Q**: Do children earn wages?
    -   **A**: Only if they work (Child Labor). Otherwise, they need a "Dependent" allowance system (future spec).
