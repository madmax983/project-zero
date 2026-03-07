# 387 - Workplace Hazards

## 1. Overview
**Layer:** 1
**Fantasy:** The cost of progress is blood. The frontier is dangerous and OSHA is light-years away.
**Mechanic:** Buildings have an "Accident Risk" based on condition and workload. Accidents cause injuries (temporary debuffs) or permanent disabilities. "Safety Protocols" policy reduces risk but slows work.
**Emergence:** Your master engineer loses an arm in a rush job, becoming a slow but wise mentor. A series of mine collapses leads to a strike.
**Tension:** Push for quota (risk injury) or work safely (slow production)?

## 2. Dependencies
- `Pop` component
- `Health` or `Injury` system for Pops
- `Building` and `Workplace` components
- RNG utility (e.g. `fastrand` or `rand`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Health;
    use crate::layer1::buildings::Workplace;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<WorkplaceAccidentEvent>();
        app.add_systems(Update, process_workplace_hazards_system);
        app
    }

    #[test]
    fn test_high_risk_triggers_accident() {
        let mut app = setup_app();

        let worker = app.world_mut().spawn(Health::default()).id();

        // Spawn workplace with 100% risk
        app.world_mut().spawn(Workplace {
            workers: vec![worker],
            accident_risk: 1.0,
        });

        app.update();

        let events = app.world().resource::<Events<WorkplaceAccidentEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).len() > 0, "Accident should have triggered");
    }

    #[test]
    fn test_zero_risk_no_accident() {
        let mut app = setup_app();

        let worker = app.world_mut().spawn(Health::default()).id();

        // Spawn workplace with 0% risk
        app.world_mut().spawn(Workplace {
            workers: vec![worker],
            accident_risk: 0.0,
        });

        app.update();

        let events = app.world().resource::<Events<WorkplaceAccidentEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).len(), 0, "Accident should not trigger");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::buildings::Workplace;

#[derive(Event)]
pub struct WorkplaceAccidentEvent {
    pub worker: Entity,
    pub workplace: Entity,
}

pub fn process_workplace_hazards_system(
    query: Query<(Entity, &Workplace)>,
    mut events: EventWriter<WorkplaceAccidentEvent>,
) {
    for (entity, workplace) in query.iter() {
        // Simple fastrand check against risk threshold
        if fastrand::f32() < workplace.accident_risk {
            if let Some(&worker) = workplace.workers.first() {
                events.send(WorkplaceAccidentEvent {
                    worker,
                    workplace: entity,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Risk should scale with machine degradation or missing maintenance.
- Accidents shouldn't trigger multiple times in the same tick; add a cooldown component `AccidentCooldown`.
- Extract injury logic into a separate system that listens to `WorkplaceAccidentEvent`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] High-risk buildings periodically generate accidents that apply damage/debuffs to workers.

## 7. Technical Guidance
- Register `WorkplaceAccidentEvent`.
- Ensure the event triggers the health/injury module correctly so that a `Pop` receives damage or an `Injury` component.

## 8. Questions
- How should the "Safety Protocols" policy be implemented? A global resource multiplier or a local building toggle?
