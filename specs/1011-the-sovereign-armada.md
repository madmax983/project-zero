# 1011: The Sovereign Armada

## 1. Overview
A colossal fleet of heavily armed generation ships travels slowly across the galaxy map (Layer 3). When it enters a system (Layer 2), it stops to strip-mine asteroid belts and "tax" local colonies for supplies. They are too powerful to defeat early on, so the player must pay them or hide. This creates tension around predicting and navigating around this massive, unstoppable force of nature, or exploiting its overwhelming power against enemies.

## 2. Dependencies
- Layer 3 `GalaxyMap` and node-to-node travel.
- Layer 2 `System` entry events.
- Layer 2 `Fleet` entities (Sovereign Armada type).
- Layer 1/2 `Diplomacy` (Tribute demands).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::galaxy::{GalaxyNode, FleetTravelEvent};
    use crate::layer2::fleet::{Fleet, Armada};
    use crate::layer1::diplomacy::TributeDemandEvent;

    #[test]
    fn test_armada_entry_triggers_tribute_demand() {
        let mut app = App::new();
        app.add_event::<FleetTravelEvent>();
        app.add_event::<TributeDemandEvent>();
        app.add_systems(Update, armada_arrival_system);

        let system_node = app.world_mut().spawn(GalaxyNode).id();
        let armada = app.world_mut().spawn((
            Fleet,
            Armada { strength: 10000 },
        )).id();

        // Armada arrives in the system
        app.world_mut().resource_mut::<Events<FleetTravelEvent>>().send(FleetTravelEvent {
            fleet: armada,
            destination: system_node,
        });

        app.update();

        // Verify TributeDemand event was fired for the system
        let tribute_events = app.world().resource::<Events<TributeDemandEvent>>();
        let mut reader = tribute_events.get_reader();
        let mut found = false;
        for event in reader.read(tribute_events) {
            if event.system == system_node && event.aggressor == armada {
                found = true;
            }
        }

        assert!(found, "Armada entering a system should immediately trigger a Tribute Demand.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer3/sovereign_armada.rs
use bevy::prelude::*;
use crate::layer3::galaxy::{GalaxyNode, FleetTravelEvent};
use crate::layer2::fleet::Fleet;
use crate::layer1::diplomacy::TributeDemandEvent;

#[derive(Component)]
pub struct Armada {
    pub strength: u32,
}

pub fn armada_arrival_system(
    query: Query<&Armada>,
    mut travel_events: EventReader<FleetTravelEvent>,
    mut tribute_events: EventWriter<TributeDemandEvent>,
) {
    for event in travel_events.read() {
        if query.get(event.fleet).is_ok() {
            // It's the Armada arriving
            tribute_events.send(TributeDemandEvent {
                aggressor: event.fleet,
                system: event.destination,
                amount: 5000, // Hardcoded massive tribute for MVP
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Target Selection:** The Armada shouldn't just demand tribute from the abstract "System"; it needs to query the Layer 2 nodes in that system and find specific Colonies/Stations to demand from.
- **Consequences:** If tribute is not paid within a timeframe, the Armada should switch to hostile behavior and actively bombard Layer 1 colonies or destroy Layer 2 stations.
- **Resource Extraction:** Aside from tribute, the Armada should automatically strip resources from unclaimed asteroid belts in the system while it resides there.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_armada_entry_triggers_tribute_demand` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `TributeDemandEvent` needs to hook into the UI so the player is presented with a choice (Pay vs Refuse).
- For the MVP, if the player refuses, you can just apply a massive `Unrest` penalty or destroy a random station to simulate the consequence without needing a full combat simulation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
