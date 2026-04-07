# 851 - Subscription Prosthetics

## 1. Overview
Advanced prosthetics give massive bonuses to Pops but require monthly "License Key" updates or "Proprietary Fluid" resources. If a payment or shipment is missed, the limb locks up, causing a severe debuff. This forces players to choose between high-tech dependency and low-tech autonomy.

## 2. Dependencies
- `src/layer1/components.rs` or `src/layer1/needs.rs` (Pop stats and Needs)
- `src/layer1/inventory.rs` or similar resource management
- `src/layer1/events.rs`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal systems
        app
    }

    #[test]
    fn test_subscription_prosthetic_grants_bonus() {
        let mut app = setup_app();
        // Arrange: Pop with active SubscriptionProsthetic
        // Act: Evaluate work efficiency
        // Assert: Efficiency has massive bonus
    }

    #[test]
    fn test_subscription_prosthetic_locks_up_on_missed_payment() {
        let mut app = setup_app();
        // Arrange: Pop with SubscriptionProsthetic, but colony lacks required resource/credits
        // Act: Run subscription update system
        // Assert: Prosthetic state changes to Locked, Pop gets severe efficiency debuff
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct SubscriptionProsthetic {
    pub is_active: bool,
    pub bonus_multiplier: f32,
    pub debuff_multiplier: f32,
    pub resource_cost_per_cycle: f32, // Or use a specific resource type
}

// Systems
pub fn subscription_prosthetic_system(
    mut prosthetics: Query<&mut SubscriptionProsthetic>,
    // Resource or credit storage access
) {
    // Deduct cost per cycle. If not enough, set is_active = false
}

pub fn apply_prosthetic_effects_system(
    prosthetics: Query<(&SubscriptionProsthetic, &mut crate::layer1::needs::Needs)>, // Adjust based on how efficiency/stats are tracked
) {
    // Apply bonus if active, apply severe debuff if locked
}
```

## 5. REFACTOR Phase: Quality & Design
- Centralize the resource deduction logic to handle partial payments or grace periods if appropriate.
- Ensure the lock-up debuff correctly impacts pathfinding speed or work action execution duration.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active prosthetics grant a bonus.
- [ ] Unpaid prosthetics lock up and apply a debuff.

## 7. Technical Guidance
- The "billing cycle" should be tied to `SimulationTime` to happen at regular intervals (e.g., once a month or every X ticks).
- Consider emitting an event when a prosthetic locks up for UI notifications.

## 8. Questions
*Builder: add questions here if spec is unclear.*
