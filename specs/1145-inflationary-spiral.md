# 1145: Inflationary Spiral

## 1. Overview
The "Inflationary Spiral" feature implements a dynamic galactic market where the value of "Credits" fluctuates relative to goods. During a market crash, Credits lose their value, rendering cash stockpiles useless and forcing empires to rely on barter (trading goods for goods). This creates a strategic tension between holding liquid assets (Credits, which are flexible but volatile) versus hard assets (Alloys/Goods, which are static but stable).

## 2. Dependencies
- `bevy_ecs` for managing empire resources and market state.
- `MarketState` resource that tracks the current conversion rate of Credits to standard goods.
- `EmpireResources` component or resource that holds stockpiles of Credits and Alloys.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_market_crash_reduces_credit_value() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(MarketState {
            credit_value_multiplier: 1.0, // 1 Credit = 1 Unit of purchasing power
        });

        // Act: Trigger a market crash
        world.run_system_once(trigger_market_crash);

        // Assert: Credit value multiplier should drop significantly
        let market = world.resource::<MarketState>();
        assert_eq!(market.credit_value_multiplier, 0.1); // Value plummeted
    }

    #[test]
    fn test_barter_trade_bypasses_credit_value() {
        // Arrange
        let mut world = World::new();
        // A crashed market where credits are worthless
        world.insert_resource(MarketState {
            credit_value_multiplier: 0.1,
        });

        // Setup two entities (empires/traders) to perform a barter
        let empire_a = world.spawn(EmpireResources {
            credits: 1_000_000.0,
            alloys: 100.0,
        }).id();

        let empire_b = world.spawn(EmpireResources {
            credits: 10.0,
            alloys: 500.0,
        }).id();

        // Let's say Empire A wants 100 Alloys from Empire B, using barter (100 Alloys for 100 Alloys)
        world.insert_resource(PendingBarter {
            initiator: empire_a,
            target: empire_b,
            offer_alloys: 100.0,
            request_alloys: 100.0,
        });

        // Act: Process the barter
        world.run_system_once(process_barter_trade);

        // Assert: Goods are exchanged regardless of credit value
        let a_res = world.get::<EmpireResources>(empire_a).unwrap();
        let b_res = world.get::<EmpireResources>(empire_b).unwrap();

        assert_eq!(a_res.alloys, 100.0); // 100 - 100 + 100 = 100
        assert_eq!(b_res.alloys, 500.0); // 500 - 100 + 100 = 500
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct MarketState {
    pub credit_value_multiplier: f32,
}

#[derive(Component)]
pub struct EmpireResources {
    pub credits: f32,
    pub alloys: f32,
}

#[derive(Resource)]
pub struct PendingBarter {
    pub initiator: Entity,
    pub target: Entity,
    pub offer_alloys: f32,
    pub request_alloys: f32,
}

pub fn trigger_market_crash(mut market: ResMut<MarketState>) {
    market.credit_value_multiplier = 0.1;
}

pub fn process_barter_trade(
    mut commands: Commands,
    mut query: Query<&mut EmpireResources>,
    barter: Option<Res<PendingBarter>>,
) {
    if let Some(trade) = barter {
        if let Ok([mut init_res, mut target_res]) = query.get_many_mut([trade.initiator, trade.target]) {
            // Check if both parties have enough alloys
            if init_res.alloys >= trade.offer_alloys && target_res.alloys >= trade.request_alloys {
                // Execute the barter
                init_res.alloys -= trade.offer_alloys;
                init_res.alloys += trade.request_alloys;

                target_res.alloys -= trade.request_alloys;
                target_res.alloys += trade.offer_alloys;
            }
        }
        commands.remove_resource::<PendingBarter>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded `0.1` crash multiplier. This should be an event or driven by an economic simulation rather than a static drop.
- **Performance**: Resource-based pending barter only handles one trade at a time. This should ideally be an Event-based system (`EventReader<BarterRequest>`) to handle multiple trades per tick.
- **API Improvements**: `EmpireResources` currently uses `f32` for discrete goods, which can lead to precision errors. Consider `u32` or `i32` if fractions are not needed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `trigger_market_crash` reduces `credit_value_multiplier` to 0.1.
- [ ] `process_barter_trade` successfully swaps alloys between two empires based on the pending barter.

## 7. Technical Guidance
- Ensure that UI systems correctly display the current purchasing power of Credits by multiplying display prices by `1.0 / credit_value_multiplier`.
- Bevy's `get_many_mut` is the correct way to mutate two components of the same type from the same query to avoid aliasing rules.

## 8. Questions
*Builder: add questions here if spec is unclear.*
