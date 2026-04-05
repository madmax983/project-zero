# 782-the-void-architect

## 1. Overview
An enigmatic Layer 2 ship occasionally arrives, offering to instantly construct a highly advanced, massive Layer 1 building for free. The catch is that it demands bizarre and costly future payments (e.g., "10 tons of harvested misery", "the memories of your three best doctors"). Refusing the payment after the building is constructed causes it to violently deconstruct itself, destroying anything nearby.

This creates tension between the immense immediate benefit of impossible construction and the horrifying, unpredictable cost of honoring the debt.

## 2. Dependencies
- `010` Chronicle System (for recording the arrival, construction, and payment events)
- `152` Orbital Stations (Layer 2 entities interacting with Layer 1)
- `036` Pop Memory (for memory extraction payments)
- `034` Pop Health and Damage (for misery/injury payments)

## 3. RED Phase: Tests First

```rust
// tests/void_architect_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::pop::Pop;
    use scale::layer1::buildings::Building;
    use scale::layer1::resources::ColonyResources;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_architect_offer_acceptance() {
        let mut world = setup_world();

        let offer = VoidArchitectOffer {
            building_type: BuildingType::ThermalSpire,
            payment_type: PaymentType::HarvestedMisery(10.0),
            deadline_ticks: 5000,
        };

        // Player accepts offer
        accept_void_architect_offer(&mut world, offer.clone());

        // Building should be constructed immediately
        let building_count = world.query::<&Building>().iter(&world).count();
        assert_eq!(building_count, 1);

        // Debt should be recorded
        let debt = world.get_resource::<VoidArchitectDebt>().unwrap();
        assert_eq!(debt.amount_due, 10.0);
        assert_eq!(debt.ticks_remaining, 5000);
    }

    #[test]
    fn test_architect_payment_refusal_destruction() {
        let mut world = setup_world();

        // Setup existing debt and building
        let building_ent = world.spawn(Building { building_type: BuildingType::ThermalSpire }).id();
        world.insert_resource(VoidArchitectDebt {
            building_entity: building_ent,
            payment_type: PaymentType::Blindness(50),
            amount_due: 50.0,
            ticks_remaining: 0, // Due now
        });

        // Player refuses payment
        refuse_void_architect_payment(&mut world);

        // Building should be destroyed
        assert!(world.get_entity(building_ent).is_none());

        // Nearby entities should take damage (mocked explosion)
        // ...
    }

    #[test]
    fn test_architect_payment_fulfillment() {
        let mut world = setup_world();

        // Setup debt
        let building_ent = world.spawn(Building { building_type: BuildingType::ThermalSpire }).id();
        world.insert_resource(VoidArchitectDebt {
            building_entity: building_ent,
            payment_type: PaymentType::Memories(3),
            amount_due: 3.0,
            ticks_remaining: 100,
        });

        // Setup pops with memories
        for _ in 0..3 {
            world.spawn((Pop, PopMemory { count: 1 }));
        }

        // Fulfill payment
        fulfill_void_architect_payment(&mut world);

        // Debt should be cleared
        assert!(world.get_resource::<VoidArchitectDebt>().is_none());

        // Pops should have memories extracted
        for (_, mem) in world.query::<(&Pop, &PopMemory)>().iter(&world) {
            assert_eq!(mem.count, 0);
        }
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/void_architect.rs

use bevy_ecs::prelude::*;
use crate::layer1::buildings::{Building, BuildingType};

#[derive(Clone, Debug)]
pub enum PaymentType {
    HarvestedMisery(f32),
    Memories(u32),
    Blindness(u32),
}

#[derive(Clone, Debug)]
pub struct VoidArchitectOffer {
    pub building_type: BuildingType,
    pub payment_type: PaymentType,
    pub deadline_ticks: u64,
}

#[derive(Resource)]
pub struct VoidArchitectDebt {
    pub building_entity: Entity,
    pub payment_type: PaymentType,
    pub amount_due: f32,
    pub ticks_remaining: u64,
}

pub fn accept_void_architect_offer(world: &mut World, offer: VoidArchitectOffer) {
    let building_ent = world.spawn(Building {
        building_type: offer.building_type,
    }).id();

    let amount = match offer.payment_type {
        PaymentType::HarvestedMisery(amt) => amt,
        PaymentType::Memories(amt) => amt as f32,
        PaymentType::Blindness(amt) => amt as f32,
    };

    world.insert_resource(VoidArchitectDebt {
        building_entity: building_ent,
        payment_type: offer.payment_type,
        amount_due: amount,
        ticks_remaining: offer.deadline_ticks,
    });
}

pub fn refuse_void_architect_payment(world: &mut World) {
    if let Some(debt) = world.remove_resource::<VoidArchitectDebt>() {
        world.despawn(debt.building_entity);
        // Trigger explosion event here
    }
}

pub fn fulfill_void_architect_payment(world: &mut World) {
    if let Some(debt) = world.remove_resource::<VoidArchitectDebt>() {
        // Implement logic to extract payment from colony/pops
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract the payment fulfillment logic into distinct systems per `PaymentType` to avoid a massive `match` statement in `fulfill_void_architect_payment`.
- Introduce a UI component to track the `ticks_remaining` for the player to anticipate the demand.
- Tie the arrival of the Architect to specific Layer 2 triggers, such as low colony morale or energy crisis, to maximize the temptation of the offer.
- Ensure the explosive deconstruction generates a `ChronicleEvent` to record the tragedy.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Architect offer generates the building and records the debt correctly.
- [ ] Refusing payment destroys the building.
- [ ] Fulfilling payment clears the debt resource.

## 7. Technical Guidance
- **Explosion Logic:** Use the existing `ExplosionEvent` or environmental damage systems when the building deconstructs.
- **Payment Abstraction:** The `PaymentType` enum should be easily extensible for future bizarre demands.
- **System Scheduling:** `VoidArchitectDebt` ticks should be decremented in a system that runs every simulation tick, similar to metabolic decay.

## 8. Questions
*Builder: Add any questions here.*