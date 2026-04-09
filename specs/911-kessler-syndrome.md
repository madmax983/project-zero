# 911: Kessler Syndrome

## 1. Overview
Every ship launch, destruction, or orbital construction generates a "debris" probability in orbit. High debris concentration risks destroying new launches and locks the colony on the surface, preventing trade or evacuation until cleared by specialized "janitor ships". This introduces a tension between cheap, dirty boosters for rapid expansion and expensive, clean launches for sustainability.

## 2. Dependencies
- `layer2::orbit::OrbitalZone`
- `layer2::ships::LaunchEvent`
- `layer2::ships::ShipDestructionEvent`
- `layer2::construction::OrbitalConstructionEvent`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::orbit::{OrbitalZone, DebrisConcentration};
    use crate::layer2::ships::{LaunchEvent, ShipDestructionEvent, ShipStatus};

    #[test]
    fn test_launch_generates_debris() {
        let mut app = App::new();
        app.add_systems(Update, process_launch_debris);

        let orbit_id = app.world_mut().spawn((
            OrbitalZone::default(),
            DebrisConcentration(0.0),
        )).id();

        app.world_mut().send_event(LaunchEvent {
            orbit_id,
            booster_type: BoosterType::Cheap,
        });

        app.update();

        let debris = app.world().get::<DebrisConcentration>(orbit_id).expect("Zone should have debris");
        assert!(debris.0 > 0.0, "Cheap launch should generate orbital debris");
    }

    #[test]
    fn test_high_debris_destroys_launch() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_launch_risk);

        let orbit_id = app.world_mut().spawn((
            OrbitalZone::default(),
            DebrisConcentration(0.9), // Critical debris
        )).id();

        let ship_id = app.world_mut().spawn(ShipStatus::Launching).id();

        app.world_mut().send_event(LaunchEvent {
            orbit_id,
            ship_id: Some(ship_id),
            booster_type: BoosterType::Clean,
        });

        app.update();

        let status = app.world().get::<ShipStatus>(ship_id).unwrap();
        assert_eq!(*status, ShipStatus::Destroyed, "Ship should be destroyed by Kessler Syndrome debris");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct DebrisConcentration(pub f32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoosterType {
    Cheap,
    Clean,
}

#[derive(Event)]
pub struct LaunchEvent {
    pub orbit_id: Entity,
    pub ship_id: Option<Entity>,
    pub booster_type: BoosterType,
}

#[derive(Component, PartialEq, Eq)]
pub enum ShipStatus {
    Launching,
    Destroyed,
    InOrbit,
}

pub fn process_launch_debris(
    mut launch_events: EventReader<LaunchEvent>,
    mut orbit_query: Query<&mut DebrisConcentration>,
) {
    for event in launch_events.read() {
        if let Ok(mut debris) = orbit_query.get_mut(event.orbit_id) {
            match event.booster_type {
                BoosterType::Cheap => debris.0 += 0.05,
                BoosterType::Clean => debris.0 += 0.01,
            }
        }
    }
}

pub fn evaluate_launch_risk(
    mut launch_events: EventReader<LaunchEvent>,
    orbit_query: Query<&DebrisConcentration>,
    mut ship_query: Query<&mut ShipStatus>,
) {
    for event in launch_events.read() {
        if let Ok(debris) = orbit_query.get(event.orbit_id) {
            if debris.0 > 0.8 { // Kessler threshold
                if let Some(ship_id) = event.ship_id {
                    if let Ok(mut status) = ship_query.get_mut(ship_id) {
                        *status = ShipStatus::Destroyed;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Extract the debris thresholds into a `DebrisConfig` resource so it can be tuned easily without recompiling.
- **Code Smells:** Hardcoding `0.05` and `0.01` debris generation rates. These should be defined per `BoosterType` or within a resource.
- **Performance:** If there are many launches per tick, batching debris generation might be required, though `EventReader` already handles this nicely.
- **API Improvements:** Add a `DebrisJanitor` trait or component to encapsulate ships capable of reducing the `DebrisConcentration`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Cheap boosters increase debris significantly more than clean boosters
- [ ] Launches through critical debris concentrations are destroyed

## 7. Technical Guidance
- **Code Structure:** Place the systems in `src/layer2/orbit/debris.rs`. Register them in the simulation schedule to run after launch events are generated.
- **Integration Points:** You will need to emit a `ShipDestructionEvent` when a ship is destroyed by debris to integrate with the colony's memorial and morale systems.
- **Gotchas:** Make sure debris generation is deterministic to prevent save/load discrepancies or desyncs in multiplayer.

## 8. Questions
*Builder: add questions here if spec is unclear.*
