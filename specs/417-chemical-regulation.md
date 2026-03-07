# Chemical Regulation

## 1. Overview
**Layer:** 1
**Fantasy:** Better living through chemistry. Keeping the colony running on caffeine and stims.
**Mechanic:** Consumables ("Stims", "Sedatives") modify stats. Stims = +Speed, -Health. Sedatives = +Mood, -Speed. Addiction mechanics.
**Emergence:** You issue mandatory "Wake-Up" pills to meet a deadline. The deadline is met, but the withdrawal crash next week paralyzes the colony.
**Tension:** Health vs. Productivity.

## 2. Dependencies
- `005-pop-needs` (Needs component structure and metabolism)
- `034-pop-health` (Status effects, buffs/debuffs)
- `181-chemical-regulation` (Existing chemical consumption framework if applicable)

## 3. RED Phase: Tests First

```rust
// specs/417-chemical-regulation.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Needs, Stats};
    use crate::layer1::drugs::{DrugEffect, Addiction, consume_drug_system, process_addiction_system};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (consume_drug_system, process_addiction_system));
        app
    }

    #[test]
    fn test_consume_stim_applies_buffs() {
        // Arrange
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Stats { speed_modifier: 1.0, health_drain_rate: 0.0 },
        )).id();

        // Act
        app.world_mut().send_event(crate::layer1::drugs::ConsumeDrugEvent {
            consumer: pop_id,
            drug_type: "Stim".to_string(),
        });
        app.update();

        // Assert: Speed increased, Health Drain increased
        let stats = app.world().get::<Stats>(pop_id).unwrap();
        assert!(stats.speed_modifier > 1.0);
        assert!(stats.health_drain_rate > 0.0);
    }

    #[test]
    fn test_addiction_development() {
        // Arrange
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Addiction { drug_type: "Stim".to_string(), severity: 0.0 },
        )).id();

        // Act
        // Repeated consumption
        for _ in 0..5 {
            app.world_mut().send_event(crate::layer1::drugs::ConsumeDrugEvent {
                consumer: pop_id,
                drug_type: "Stim".to_string(),
            });
            app.update();
        }

        // Assert: Addiction severity increased
        let addiction = app.world().get::<Addiction>(pop_id).unwrap();
        assert!(addiction.severity > 0.0);
    }

    #[test]
    fn test_withdrawal_symptoms() {
        // Arrange
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Stats { speed_modifier: 1.0, health_drain_rate: 0.0 },
            Addiction { drug_type: "Stim".to_string(), severity: 80.0 }, // Highly addicted
            crate::layer1::drugs::WithdrawalTimer { time_since_last_dose: 100.0 }, // Long time without dose
        )).id();

        // Act
        app.update(); // Tick withdrawal system

        // Assert: Speed debuffed due to withdrawal crash
        let stats = app.world().get::<Stats>(pop_id).unwrap();
        assert!(stats.speed_modifier < 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/drugs.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Needs, Stats};
use std::collections::HashMap;

#[derive(Component)]
pub struct DrugEffect {
    pub active_drugs: HashMap<String, f32>, // Drug name -> remaining duration
}

#[derive(Component)]
pub struct Addiction {
    pub drug_type: String,
    pub severity: f32, // 0 to 100
}

#[derive(Component)]
pub struct WithdrawalTimer {
    pub time_since_last_dose: f32,
}

#[derive(Event)]
pub struct ConsumeDrugEvent {
    pub consumer: Entity,
    pub drug_type: String,
}

pub fn consume_drug_system(
    mut events: EventReader<ConsumeDrugEvent>,
    mut pop_query: Query<(&mut Stats, Option<&mut Addiction>)>,
) {
    for event in events.read() {
        if let Ok((mut stats, opt_addiction)) = pop_query.get_mut(event.consumer) {
            match event.drug_type.as_str() {
                "Stim" => {
                    stats.speed_modifier += 0.5;
                    stats.health_drain_rate += 0.1;
                },
                "Sedative" => {
                    stats.speed_modifier -= 0.3;
                    // Apply mood buff here if Needs component was accessible
                },
                _ => {}
            }

            if let Some(mut addiction) = opt_addiction {
                if addiction.drug_type == event.drug_type {
                    addiction.severity += 10.0;
                }
            }
        }
    }
}

pub fn process_addiction_system(
    mut pop_query: Query<(&mut Stats, &Addiction, &mut WithdrawalTimer)>,
    time: Res<Time>,
) {
    for (mut stats, addiction, mut timer) in pop_query.iter_mut() {
        timer.time_since_last_dose += time.delta_secs();

        // If highly addicted and haven't had dose recently -> Withdrawal crash
        if addiction.severity > 50.0 && timer.time_since_last_dose > 60.0 {
            // Apply severe debuffs
            if addiction.drug_type == "Stim" {
                stats.speed_modifier = 0.5; // Half speed crash
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Duration/Decay:** The `DrugEffect` component should track remaining duration. When it hits 0, the buff/debuff is removed.
- **Withdrawal Scaling:** Withdrawal severity should scale with the addiction severity and time passed.
- **Utility AI:** Pops with high `Addiction` severity should have their Utility AI score for "Find Drug" override basic needs like eating or working.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Consuming Stims increases `speed_modifier` and `health_drain_rate`.
- [ ] Repeated consumption increases `Addiction` severity.
- [ ] Prolonged time without the drug (while addicted) causes severe withdrawal stat debuffs.

## 7. Technical Guidance
- Build upon existing status effect systems if they exist, rather than hardcoding stat modifications in the drug consumption event.
- Ensure the `WithdrawalTimer` is reset to 0 upon receiving a new `ConsumeDrugEvent` for that specific drug.

## 8. Questions
*Builder: add questions here if spec is unclear.*
