# 662 - Refugee Waves

## 1. Overview
**Layer:** 3 -> 1
**Fantasy:** The galaxy is burning, and you are the lifeboat. Wars and disasters in neighboring sectors displace vast populations. Large groups of "Refugee" pops arrive in desperate condition (injured, starving, carrying strange traits). Accepting them strains your resources but provides cheap, abundant labor. Rejecting them causes diplomatic penalties, potential combat, or massive unrest among sympathetic colonists.
**Mechanic:** A Layer 3 event triggers a "Refugee Fleet" arrival in Layer 2 orbit. They demand settlement on Layer 1. The player must choose: Accept (massive population influx, low starting health/morale, resource drain) or Reject (diplomatic incident, refugee ships turn hostile or die in orbit causing a morale penalty).
**Emergence:** You accept a wave of refugees who turn out to be the losing side of a civil war. The winning side arrives two years later demanding you hand them over or face orbital bombardment.
**Tension:** Moral duty and cheap labor vs. crippling resource strain and immense political risk.

## 2. Dependencies
- `099` Fleet Movement (Layer 2 arrival mechanics)
- `003` Population Basics (Spawning pops, assigning traits)
- `046` Notifications System (Player decision prompt)
- `469` The Galactic Council (Diplomatic consequences)

## 3. RED Phase: Tests First

```rust
// tests/layer3/refugee_waves.rs

use bevy::prelude::*;
use scale::layer3::diplomacy::DiplomaticRelations;
use scale::layer2::fleet::{Fleet, FleetState};
use scale::layer1::pops::{Pop, Health, Needs};
use scale::layer3::refugee_waves::{RefugeeWaveEvent, process_refugee_decision, Decision};
use scale::layer1::resources::ColonyResources;

#[test]
fn test_accepting_refugees_spawns_pops_with_low_health() {
    let mut app = App::new();
    app.add_systems(Update, process_refugee_decision);

    app.world_mut().insert_resource(ColonyResources::default());

    // Fire decision event to ACCEPT
    app.world_mut().send_event(RefugeeWaveEvent {
        decision: Decision::Accept,
        population_count: 50,
        origin_faction: "Rebel Alliance".to_string(),
        health_penalty: 50.0,
    });

    app.update();

    // Verify 50 new pops spawned with poor health
    let mut pop_query = app.world_mut().query::<(&Pop, &Health)>();
    let new_pops: Vec<_> = pop_query.iter(app.world()).collect();

    assert_eq!(new_pops.len(), 50);
    // Assuming max health is 100
    assert!(new_pops[0].1.current <= 50.0);
}

#[test]
fn test_rejecting_refugees_causes_diplomatic_penalty() {
    let mut app = App::new();
    app.add_systems(Update, process_refugee_decision);

    app.world_mut().insert_resource(DiplomaticRelations::default());

    // Fire decision event to REJECT
    app.world_mut().send_event(RefugeeWaveEvent {
        decision: Decision::Reject,
        population_count: 50,
        origin_faction: "Galactic Senate".to_string(),
        health_penalty: 0.0,
    });

    app.update();

    let diplomacy = app.world().resource::<DiplomaticRelations>();
    // The player's standing with "Galactic Senate" should decrease significantly
    assert!(diplomacy.get_standing("Galactic Senate") < 0.0);
}

#[test]
fn test_rejecting_refugees_turns_fleet_hostile() {
    // Test that the refugee fleet in orbit either despawns (dies) or turns Pirate/Hostile
    // based on their desperation level.
    // ...
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/refugee_waves.rs

use bevy::prelude::*;
use crate::layer1::pops::{Pop, Health, Needs, Traits};
use crate::layer3::diplomacy::DiplomaticRelations;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Accept,
    Reject,
}

#[derive(Event)]
pub struct RefugeeWaveEvent {
    pub decision: Decision,
    pub population_count: usize,
    pub origin_faction: String,
    pub health_penalty: f32,
}

pub fn process_refugee_decision(
    mut events: EventReader<RefugeeWaveEvent>,
    mut commands: Commands,
    mut diplomacy: Option<ResMut<DiplomaticRelations>>,
) {
    for event in events.read() {
        match event.decision {
            Decision::Accept => {
                // Spawn massive influx of low-health, desperate pops
                for _ in 0..event.population_count {
                    commands.spawn((
                        Pop,
                        Health { current: 100.0 - event.health_penalty, max: 100.0 },
                        Needs { hunger: 10.0, rest: 10.0, morale: 20.0, ..default() },
                        Traits { list: vec!["Refugee".to_string(), "Traumatized".to_string()] },
                    ));
                }
            }
            Decision::Reject => {
                // Apply a severe diplomatic penalty with the origin faction (or their enemies)
                if let Some(mut dip) = diplomacy.as_deref_mut() {
                    let current = dip.get_standing(&event.origin_faction);
                    dip.set_standing(&event.origin_faction, current - 50.0);
                }

                // TODO: Spawn a hostile fleet in Layer 2 or trigger a massive Morale penalty on Layer 1
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance:** Spawning 50-100 Pops instantly might cause a framerate hitch. Consider staggering the arrivals via a "Shuttle Landing" queue over several ticks.
- **Integration:** The `RefugeeWaveEvent` should be triggered by a UI dialogue (a modal popup) when a Layer 2 `Fleet` entity with the `Refugee` component enters orbit.
- **Narrative:** Add Chronicle events for accepting or denying the refugees (e.g., `REFUGEES_ACCEPTED`, `REFUGEES_TURNED_AWAY`).
- **Emergence:** Give refugees a specific faction tag. If their original oppressor faction contacts the colony later, having those refugees alive should trigger a unique hostile dialogue/demand.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer3/refugee_waves.rs`.
- [ ] Accepting refugees spawns the designated number of Pops with severe debuffs.
- [ ] Rejecting refugees applies a diplomatic penalty to the `DiplomaticRelations` resource.

## 7. Technical Guidance
- **Gotchas:** Be careful not to overwhelm the utility AI. 50 starving, tired pops will immediately flood all food storage and beds. If the colony isn't prepared, a death spiral starts immediately. This is intended, but ensure pathfinding can handle the rush.
- **Traits:** The `Refugee` trait should provide a massive work-speed buff (they are desperate to prove their worth) but a higher susceptibility to mental breaks (due to the `Traumatized` trait).

## 8. Questions
*Builder: add questions here if spec is unclear.*
