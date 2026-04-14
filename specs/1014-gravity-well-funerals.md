# 1014: Gravity-Well Funerals

## 1. Overview
High-status Pops demand expensive "Sun-Burial" launches upon death, disposing of them by launching them into the local sun. This requires massive fuel expenditure from Layer 1 to Layer 2. Failing to do so causes enormous Unrest among their surviving lineage. This creates a political vs. logistical tension, as a plague could cause a backlog of Sun-Burials, bankrupting fuel reserves and grounding trade fleets.

## 2. Dependencies
- Layer 1 `Pop` death events.
- Layer 1 `Social Status` or `Lineage` mechanics.
- Layer 1/2 `Logistics` and `Fuel` tracking.
- Layer 1 `Unrest` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, StatusLevel, DeathEvent};
    use crate::layer1::economy::FuelReserves;
    use crate::layer1::social::{Lineage, Unrest};
    use crate::layer1::funerals::SunBurialRequestEvent;

    #[test]
    fn test_high_status_death_triggers_sun_burial_request() {
        let mut app = App::new();
        app.add_event::<DeathEvent>();
        app.add_event::<SunBurialRequestEvent>();
        app.add_systems(Update, process_high_status_deaths_system);

        let high_status_pop = app.world_mut().spawn((
            Pop,
            StatusLevel::Elite,
        )).id();

        app.world_mut().resource_mut::<Events<DeathEvent>>().send(DeathEvent {
            pop: high_status_pop,
            cause: "Old Age".to_string(),
        });

        app.update();

        // Verify request generated
        let requests = app.world().resource::<Events<SunBurialRequestEvent>>();
        let mut reader = requests.get_reader();
        let mut found = false;
        for event in reader.read(requests) {
            if event.deceased == high_status_pop {
                found = true;
            }
        }

        assert!(found, "A high-status pop death should trigger a Sun Burial request.");
    }

    #[test]
    fn test_denied_sun_burial_causes_lineage_unrest() {
        let mut app = App::new();
        app.add_event::<SunBurialRequestEvent>();
        app.insert_resource(FuelReserves { amount: 0 }); // No fuel available
        app.add_systems(Update, evaluate_sun_burial_requests_system);

        let deceased = app.world_mut().spawn(Pop).id();

        // Spawn family members
        let child1 = app.world_mut().spawn((Pop, Lineage { ancestor: deceased }, Unrest { value: 0.0 })).id();
        let child2 = app.world_mut().spawn((Pop, Lineage { ancestor: deceased }, Unrest { value: 0.0 })).id();

        app.world_mut().resource_mut::<Events<SunBurialRequestEvent>>().send(SunBurialRequestEvent {
            deceased,
            fuel_cost: 500,
        });

        app.update();

        // Fuel is 0, so the request is denied. Check unrest on children.
        assert!(app.world().get::<Unrest>(child1).unwrap().value > 0.0, "Denied sun burial should increase unrest for surviving lineage.");
        assert!(app.world().get::<Unrest>(child2).unwrap().value > 0.0, "Denied sun burial should increase unrest for surviving lineage.");
    }

    #[test]
    fn test_fulfilled_sun_burial_consumes_fuel() {
        let mut app = App::new();
        app.add_event::<SunBurialRequestEvent>();
        app.insert_resource(FuelReserves { amount: 1000 }); // Plenty of fuel
        app.add_systems(Update, evaluate_sun_burial_requests_system);

        let deceased = app.world_mut().spawn(Pop).id();

        app.world_mut().resource_mut::<Events<SunBurialRequestEvent>>().send(SunBurialRequestEvent {
            deceased,
            fuel_cost: 500,
        });

        app.update();

        let fuel = app.world().resource::<FuelReserves>();
        assert_eq!(fuel.amount, 500, "Fulfilled sun burial should consume fuel.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/gravity_funerals.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, StatusLevel, DeathEvent};
use crate::layer1::economy::FuelReserves;
use crate::layer1::social::{Lineage, Unrest};

#[derive(Event)]
pub struct SunBurialRequestEvent {
    pub deceased: Entity,
    pub fuel_cost: u32,
}

pub fn process_high_status_deaths_system(
    query: Query<&StatusLevel>,
    mut death_events: EventReader<DeathEvent>,
    mut burial_requests: EventWriter<SunBurialRequestEvent>,
) {
    for event in death_events.read() {
        if let Ok(status) = query.get(event.pop) {
            if *status == StatusLevel::Elite {
                burial_requests.send(SunBurialRequestEvent {
                    deceased: event.pop,
                    fuel_cost: 500, // MVP cost
                });
            }
        }
    }
}

pub fn evaluate_sun_burial_requests_system(
    mut burial_requests: EventReader<SunBurialRequestEvent>,
    mut fuel: ResMut<FuelReserves>,
    mut lineage_query: Query<(&Lineage, &mut Unrest)>,
) {
    for request in burial_requests.read() {
        if fuel.amount >= request.fuel_cost {
            // Fulfill
            fuel.amount -= request.fuel_cost;
        } else {
            // Deny: Punish the lineage
            for (lineage, mut unrest) in lineage_query.iter_mut() {
                if lineage.ancestor == request.deceased {
                    unrest.value += 50.0; // Massive penalty
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Lineage Tree:** `Lineage` shouldn't just be direct children; it should trace back up through family trees to apply unrest to a wider political faction.
- **Backlogging:** Instead of instant deny, requests should enter a queue. If fuel runs out, bodies pile up in morgues (creating health hazards) while the family waits impatiently, building unrest slowly until either launched or forcibly cremated.
- **Fleet Impact:** Fuel consumption shouldn't just be a number dropping; it needs to tie directly into the Layer 2 economy, visibly grounding ships.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_high_status_death_triggers_sun_burial_request` passes.
- [ ] Test `test_denied_sun_burial_causes_lineage_unrest` passes.
- [ ] Test `test_fulfilled_sun_burial_consumes_fuel` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- The queue/backlog system is crucial for the intended emergence described in the idea (plague causing a backlog that grounds the fleet).
- Be careful with entity lifetime; if the `deceased` entity is despawned immediately on death, queries requiring it might fail. Consider keeping "Corpse" entities around.

## 8. Questions
*Builder: add questions here if spec is unclear.*
