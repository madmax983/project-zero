# 1058: Retro-Causality Contracts

## 1. Overview
In deep space logistics, time dilation and FTL communication latency mean that contracts or edicts are sometimes agreed upon *after* the goods have theoretically arrived, or actions are taken based on predictive models of future contracts. A "Retro-Causality Contract" allows the colony to receive resources immediately from a passing high-speed freighter, with the obligation to "fulfill" the contract conditions retroactively. Failing to fulfill the contract creates a temporal-legal paradox, resulting in massive fines or hostile intervention by the enforcement faction.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- `Time` resource
- `ColonyResources` or equivalent economy manager
- Faction/Diplomacy relations tracking

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_accepting_retro_contract_grants_immediate_resources() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyResources { credits: 0 });
        app.add_event::<AcceptRetroContractEvent>();
        app.add_systems(Update, handle_retro_contract_acceptance);

        // Act
        app.world_mut().send_event(AcceptRetroContractEvent {
            credit_advance: 1000,
            deadline_days: 10.0,
        });
        app.update();

        // Assert
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 1000);

        let contracts = app.world_mut().query::<&RetroContract>().iter(&app.world()).count();
        assert_eq!(contracts, 1);
    }

    #[test]
    fn test_failing_retro_contract_applies_penalty() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.insert_resource(ColonyResources { credits: 500 });
        app.add_systems(Update, evaluate_retro_contracts);

        app.world_mut().spawn(RetroContract {
            days_remaining: 1.0,
            penalty: 2000,
            is_fulfilled: false,
        });

        // Act: Advance time past the deadline
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(100)); // Assuming 100s = 1 day
        app.update();

        // Assert: The penalty should be applied
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, -1500); // 500 - 2000
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyResources {
    pub credits: i32,
}

#[derive(Event)]
pub struct AcceptRetroContractEvent {
    pub credit_advance: i32,
    pub deadline_days: f32,
}

#[derive(Component)]
pub struct RetroContract {
    pub days_remaining: f32,
    pub penalty: i32,
    pub is_fulfilled: bool,
}

pub fn handle_retro_contract_acceptance(
    mut commands: Commands,
    mut events: EventReader<AcceptRetroContractEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        // Immediate payout
        resources.credits += event.credit_advance;

        // Create the obligation
        commands.spawn(RetroContract {
            days_remaining: event.deadline_days,
            penalty: event.credit_advance * 2, // Standard 2x penalty for MVP
            is_fulfilled: false,
        });
    }
}

pub fn evaluate_retro_contracts(
    mut commands: Commands,
    time: Res<Time>,
    mut contracts: Query<(Entity, &mut RetroContract)>,
    mut resources: ResMut<ColonyResources>,
) {
    // Standard conversion: 100 seconds = 1 day (for MVP logic)
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut contract) in contracts.iter_mut() {
        if contract.is_fulfilled {
            commands.entity(entity).despawn();
            continue;
        }

        contract.days_remaining -= day_delta;

        if contract.days_remaining <= 0.0 {
            // Failure! Apply penalty
            resources.credits -= contract.penalty;
            commands.entity(entity).despawn();

            // Note: Future specs might trigger hostile faction events here instead of simple credit drain.
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Contract Fulfillment**: Add a system/event to actually set `is_fulfilled = true` based on the player completing the required task (e.g., producing 500 units of Iron).
- **Temporal Enforcement Faction**: Instead of just subtracting credits, failing the contract should spawn a "Temporal Repo Fleet" from Layer 2 to physically collect the debt.
- **Time Abstraction**: Replace the hardcoded `100.0` seconds per day with a unified simulation time constant.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] Accepting a contract provides immediate resources.
- [ ] Failing the time limit applies the severe penalty.

## 7. Technical Guidance
- The "debt" allows credits to go negative. Ensure `ColonyResources` and other UI elements gracefully handle negative balances (e.g., disabling further purchases).
- The contract entity is despawned immediately on failure or fulfillment to prevent recurring penalties.

## 8. Questions
*Builder: add questions here if spec is unclear.*
