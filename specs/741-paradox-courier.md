# 741 - The Paradox Courier

## 1. Overview
Receiving a care package from your future self, with the terrifying obligation to eventually send it back in time. A mysterious pod arrives on Layer 1 containing endgame technology or massive resources, attached to a "Temporal Debt" contract. Decades later, a Layer 2 anomaly appears. You must deposit the exact same resources/tech into the anomaly within a tight timeframe, or suffer a massive "Paradox Event" (randomized catastrophic damage).

## 2. Dependencies
- Layer 1 resource stockpiles and delivery mechanisms.
- Layer 2 anomaly/event generation.
- Temporal/Debt tracking system spanning large in-game durations.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_paradox_event_triggers_if_debt_unpaid() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimulationTime { tick: 1000 });
        app.add_event::<ParadoxEvent>();
        app.add_systems(Update, check_temporal_debts_system);

        app.world_mut().spawn(TemporalDebt {
            amount: 500,
            resource_type: ResourceType::AdvancedAlloy,
            deadline_tick: 1000,
            is_paid: false,
        });

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<ParadoxEvent>>();
        assert_eq!(events.len(), 1, "ParadoxEvent should be fired when debt deadline is reached and unpaid");
    }

    #[test]
    fn test_debt_paid_prevents_paradox() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimulationTime { tick: 1000 });
        app.add_event::<ParadoxEvent>();
        app.add_systems(Update, check_temporal_debts_system);

        app.world_mut().spawn(TemporalDebt {
            amount: 500,
            resource_type: ResourceType::AdvancedAlloy,
            deadline_tick: 1000,
            is_paid: true, // Debt is paid
        });

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<ParadoxEvent>>();
        assert!(events.is_empty(), "ParadoxEvent should NOT be fired if debt is paid");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct SimulationTime {
    pub tick: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    AdvancedAlloy,
    EndgameTech,
}

#[derive(Component)]
pub struct TemporalDebt {
    pub amount: u32,
    pub resource_type: ResourceType,
    pub deadline_tick: u64,
    pub is_paid: bool,
}

#[derive(Event)]
pub struct ParadoxEvent {
    pub debt_entity: Entity,
}

pub fn check_temporal_debts_system(
    time: Res<SimulationTime>,
    query: Query<(Entity, &TemporalDebt)>,
    mut paradox_events: EventWriter<ParadoxEvent>,
) {
    for (entity, debt) in query.iter() {
        if !debt.is_paid && time.tick >= debt.deadline_tick {
            paradox_events.send(ParadoxEvent { debt_entity: entity });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a `ParadoxConsequenceMap` to handle the randomized catastrophic damage (e.g., destroying random buildings, wiping out resources) instead of a generic event.
- Ensure the mechanism to actually *pay* the debt via a Layer 2 anomaly interaction is implemented and well-integrated with the trade/inventory systems.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A `ParadoxEvent` fires when a `TemporalDebt` reaches its deadline unpaid.

## 7. Technical Guidance
- The notification system should warn the player significantly ahead of the `deadline_tick` (e.g., "A Temporal Anomaly has opened, requesting repayment").
- The payment logic should likely involve a specific `Designation` or interaction on the Layer 2 map targeting the anomaly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
