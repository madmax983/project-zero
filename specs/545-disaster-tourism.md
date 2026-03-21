# 545: Disaster Tourism

## 1. Overview

**Layer:** Cross-layer (2 -> 1)
**Fantasy:** The worse your colony is doing, the more wealthy outsiders want to pay to watch it burn.

**Mechanic:** When your Layer 1 colony suffers a massive, highly visible disaster (e.g., a reactor meltdown, a catastrophic localized earthquake, a violent uprising), wealthy Layer 2 "Grief Tourists" arrive in specialized, heavily shielded luxury yachts. They offer exorbitant amounts of credits to safely observe the chaos, demanding you construct high-end "viewing platforms" dangerously close to the disaster zones.

**Emergence:** Your primary fusion reactor melts down, dooming half the colony to radiation sickness. The colony is bankrupt. Suddenly, grief tourists arrive, offering enough credits to easily buy a new reactor—but only if you build a luxury hotel right on the edge of the radioactive exclusion zone, tying up your surviving engineers and exposing the tourists (and their powerful governments) to severe risk.

**Tension:** Exploiting a horrific tragedy for massive, much-needed financial gain versus the utter moral bankruptcy of profiting off your own colonists' suffering while diverting critical emergency response efforts to cater to rich voyeurs.

## 2. Dependencies

- `042` — Energy System (or other disaster sources like Meltdowns, Rebellions)
- `110` — Spontaneous Architecture (or standard Building System for Observation Decks)
- `408` — Galactic Tourism (for Tourist Pop/Ship logic)
- `039` — Trade System (for receiving Credits)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/tourism/disaster_tourism_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::disasters::{DisasterEvent, DisasterType};
    use crate::layer2::tourism::{GriefTouristArrivalEvent, TourismModule, process_disaster_tourism_system};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_disaster_triggers_grief_tourists() {
        let mut world = World::new();
        world.insert_resource(Events::<DisasterEvent>::default());
        world.insert_resource(Events::<GriefTouristArrivalEvent>::default());

        // Trigger a major disaster
        world.send_event(DisasterEvent {
            disaster_type: DisasterType::ReactorMeltdown,
            location: GridPosition { x: 10, y: 10 },
            severity: 90.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_disaster_tourism_system);
        schedule.run(&mut world);

        let arrival_events = world.resource::<Events<GriefTouristArrivalEvent>>();
        let mut reader = arrival_events.get_reader();

        // Assert that a tourist ship arrived
        assert_eq!(reader.len(arrival_events), 1);
        let arrival = reader.read(arrival_events).next().unwrap();
        assert_eq!(arrival.target_location, GridPosition { x: 10, y: 10 });
        assert!(arrival.offered_credits > 10000.0); // Exorbitant amount
    }

    #[test]
    fn test_minor_disaster_ignored_by_tourists() {
        let mut world = World::new();
        world.insert_resource(Events::<DisasterEvent>::default());
        world.insert_resource(Events::<GriefTouristArrivalEvent>::default());

        // Trigger a minor disaster
        world.send_event(DisasterEvent {
            disaster_type: DisasterType::LocalizedFire,
            location: GridPosition { x: 5, y: 5 },
            severity: 20.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_disaster_tourism_system);
        schedule.run(&mut world);

        let arrival_events = world.resource::<Events<GriefTouristArrivalEvent>>();
        let mut reader = arrival_events.get_reader();

        // Assert that NO tourist ship arrived
        assert_eq!(reader.len(arrival_events), 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/tourism/disaster_tourism.rs

use bevy_ecs::prelude::*;
use crate::layer1::disasters::{DisasterEvent, DisasterType};
use crate::layer1::map::GridPosition;

#[derive(Event)]
pub struct GriefTouristArrivalEvent {
    pub target_location: GridPosition,
    pub offered_credits: f32,
}

pub fn process_disaster_tourism_system(
    mut disaster_events: EventReader<DisasterEvent>,
    mut tourist_events: EventWriter<GriefTouristArrivalEvent>,
) {
    for disaster in disaster_events.read() {
        // Only high severity and specific types attract tourists
        if disaster.severity >= 80.0 {
            match disaster.disaster_type {
                DisasterType::ReactorMeltdown | DisasterType::MassiveEarthquake | DisasterType::ViolentUprising => {
                    tourist_events.send(GriefTouristArrivalEvent {
                        target_location: disaster.location,
                        offered_credits: disaster.severity * 500.0, // Scale payout with tragedy
                    });
                }
                _ => {}
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Tourist Demands**: The `GriefTouristArrivalEvent` should hook into the `TradeContracts` or a new `Mission` system. They don't just give money; they offer a contract to build a `ViewingPlatform` within X tiles of the `target_location`.
- **Morale Impact**: Pops working near the viewing platform should suffer massive Morale penalties ("They are watching us suffer").
- **Political Repercussions**: If a VIP tourist dies in the disaster zone, it should trigger a massive Layer 3 consequence (e.g., Blockade, Fines).

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High-severity disasters trigger the arrival of Grief Tourists offering large credit bounties.

## 7. Technical Guidance

- Integrate with the existing `Disaster` systems (if they exist) or mock them if they are still conceptual.
- Ensure the payout is high enough to be incredibly tempting but the risks (building near a meltdown) are genuinely difficult.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
