# Maintenance Debt (Spec 721)

## 1. Overview
**Layer:** 1 (Colony)
**Fantasy:** "It'll hold together for one more shift."
**Mechanic:** Instead of repairing buildings immediately (costing resources/time), players can "Defer" maintenance. The building continues to function but accumulates "Debt". If Debt > Integrity, it suffers a Catastrophic Failure (explosion/collapse), dealing damage proportional to the debt.
**Emergence:** You defer maintenance on the fusion reactor during a raid to keep the guns firing. You win the raid, but the reactor explodes five minutes later, taking out the hospital.
**Tension:** Short-term uptime (Defer) vs. Long-term safety (Repair).

## 2. Dependencies
- `layer1::building::Building` components.
- `layer1::events::ExplosionEvent` or similar destruction mechanism.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::Building;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<CatastrophicFailureEvent>();
        app.add_systems(Update, process_maintenance_debt);
        app
    }

    #[test]
    fn test_building_accumulates_maintenance_debt() {
        let mut app = setup_app();

        let building = app.world_mut().spawn((
            Building {
                name: "Reactor".to_string(),
                integrity: 100.0,
                max_integrity: 100.0,
            },
            MaintenanceDebt {
                current_debt: 0.0,
                accumulation_rate: 5.0,
                is_deferred: true,
            },
        )).id();

        app.update(); // 1 tick

        let debt = app.world().get::<MaintenanceDebt>(building).unwrap();
        assert_eq!(debt.current_debt, 5.0);
    }

    #[test]
    fn test_building_catastrophic_failure_when_debt_exceeds_integrity() {
        let mut app = setup_app();

        let building = app.world_mut().spawn((
            Building {
                name: "Reactor".to_string(),
                integrity: 50.0,
                max_integrity: 100.0,
            },
            MaintenanceDebt {
                current_debt: 45.0, // close to failure
                accumulation_rate: 10.0,
                is_deferred: true,
            },
        )).id();

        app.update(); // 1 tick

        // Debt 55.0 > Integrity 50.0 -> Failure

        let events = app.world().resource::<Events<CatastrophicFailureEvent>>();
        let mut reader = events.get_reader();
        let evs: Vec<_> = reader.read(events).collect();

        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].building, building);
        assert_eq!(evs[0].damage_radius, 55.0); // damage proportional to debt
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::building::Building;

#[derive(Component)]
pub struct MaintenanceDebt {
    pub current_debt: f32,
    pub accumulation_rate: f32,
    pub is_deferred: bool,
}

#[derive(Event)]
pub struct CatastrophicFailureEvent {
    pub building: Entity,
    pub damage_radius: f32,
}

pub fn process_maintenance_debt(
    mut commands: Commands,
    mut query: Query<(Entity, &Building, &mut MaintenanceDebt)>,
    mut failure_events: EventWriter<CatastrophicFailureEvent>,
) {
    for (entity, building, mut debt) in query.iter_mut() {
        if debt.is_deferred {
            debt.current_debt += debt.accumulation_rate;

            if debt.current_debt > building.integrity {
                failure_events.send(CatastrophicFailureEvent {
                    building: entity,
                    damage_radius: debt.current_debt,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Repairs**: There needs to be a mechanism/job for pops to "Pay Off" or repair the maintenance debt (resetting `is_deferred` to `false` and reducing `current_debt`).
- **Explosions**: Integrate `CatastrophicFailureEvent` with environmental damage, killing nearby pops or destroying adjacent structures.
- **Chronicle**: Hook into `AddChronicleEvent` to report the disastrous outcome ("Reactor suffered catastrophic failure due to deferred maintenance!").

## 6. Acceptance Criteria
- [ ] `MaintenanceDebt` component handles the accumulation.
- [ ] `process_maintenance_debt` system triggers failures when debt exceeds integrity.
- [ ] The failed building is destroyed or severely damaged.
- [ ] Events are correctly routed and logged to the Chronicle.
- [ ] Test coverage >85% for the new module.

## 7. Technical Guidance
- Place the logic in `src/layer1/maintenance.rs`.
- `CatastrophicFailureEvent` should be initialized and buffered correctly in setup and cleanup routines.
- If an explosion system exists, route the failure event into it.

## 8. Questions
- *Builder: Should deferred maintenance lower building output efficiency?*
- *Architect:* Not for the MVP. The building continues to function at 100% efficiency until it suffers Catastrophic Failure.
- *Builder: How fast should debt accumulate relative to standard simulation ticks?*
- *Architect:* The accumulation rate is defined by the `accumulation_rate` field, which should be balanced during gameplay testing, but for tests, using a static float like 5.0 per tick is sufficient.
