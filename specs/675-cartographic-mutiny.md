# 675 - The Cartographic Mutiny

## 1. Overview
The Cartographic Mutiny introduces a cross-layer mechanic where exploration data sent back from Layer 2 (System) scouts can be falsified. If a scout's morale is critically low or if they have been influenced by factions, the telemetry they send back about nodes (like planets or asteroid belts) can lie—presenting hostile or barren nodes as lush and resource-rich until a secondary expedition verifies the data. This creates tension around acting on unverified telemetry versus spending resources to double-check claims.

## 2. Dependencies
- `MapTelemetry` resource and exploration systems (from Task 626).
- `Pop` and `Needs` (specifically Morale/Happiness).
- `Faction` system to allow potential faction-based bribery/influence.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::exploration::{MapTelemetry, TelemetryData, ScoutShip};
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_low_morale_scout_falsifies_telemetry() {
        let mut app = App::new();
        app.add_systems(Update, process_scout_telemetry_system);

        let node_id = Entity::from_raw(1);
        app.world_mut().insert_resource(MapTelemetry::default());

        // Setup a scout with very low morale
        app.world_mut().spawn((
            ScoutShip { current_node: node_id },
            Morale { value: 10.0 }, // Critically low morale
        ));

        // Simulate true node data being an irradiated wasteland, but the scout scans it
        let true_data = TelemetryData {
            is_lush: false,
            danger_level: 90,
            resource_richness: 10,
        };

        app.world_mut().send_event(ScoutScanEvent {
            scout_entity: Entity::from_raw(2), // Just a dummy entity id for the scout
            node: node_id,
            true_data,
        });

        app.update();

        let telemetry = app.world().get_resource::<MapTelemetry>().unwrap();
        let reported_data = telemetry.get_node_data(node_id).unwrap();

        // Assert the data was falsified to look good
        assert!(reported_data.is_lush);
        assert!(reported_data.danger_level < 50);
        assert!(reported_data.resource_richness > 50);
        assert_eq!(reported_data.verified, false);
    }

    #[test]
    fn test_second_scout_verifies_telemetry() {
        let mut app = App::new();
        app.add_systems(Update, process_scout_telemetry_system);

        let node_id = Entity::from_raw(1);
        let mut telemetry = MapTelemetry::default();
        // Insert falsified data
        telemetry.insert_node_data(node_id, TelemetryData {
            is_lush: true,
            danger_level: 10,
            resource_richness: 90,
            verified: false,
        });
        app.world_mut().insert_resource(telemetry);

        // Setup a second scout with high morale
        app.world_mut().spawn((
            ScoutShip { current_node: node_id },
            Morale { value: 90.0 }, // High morale
        ));

        let true_data = TelemetryData {
            is_lush: false,
            danger_level: 90,
            resource_richness: 10,
            verified: false, // will be set to true by the system
        };

        app.world_mut().send_event(ScoutScanEvent {
            scout_entity: Entity::from_raw(3),
            node: node_id,
            true_data: true_data.clone(),
        });

        app.update();

        let telemetry = app.world().get_resource::<MapTelemetry>().unwrap();
        let reported_data = telemetry.get_node_data(node_id).unwrap();

        // Assert the data is now accurate and verified
        assert_eq!(reported_data.is_lush, false);
        assert_eq!(reported_data.danger_level, 90);
        assert_eq!(reported_data.verified, true);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ScoutShip {
    pub current_node: Entity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryData {
    pub is_lush: bool,
    pub danger_level: u8,
    pub resource_richness: u8,
    pub verified: bool,
}

#[derive(Resource, Default)]
pub struct MapTelemetry {
    nodes: std::collections::HashMap<Entity, TelemetryData>,
}

impl MapTelemetry {
    pub fn get_node_data(&self, node: Entity) -> Option<&TelemetryData> {
        self.nodes.get(&node)
    }

    pub fn insert_node_data(&mut self, node: Entity, data: TelemetryData) {
        self.nodes.insert(node, data);
    }
}

#[derive(Event)]
pub struct ScoutScanEvent {
    pub scout_entity: Entity,
    pub node: Entity,
    pub true_data: TelemetryData,
}

use crate::layer1::social::morale::Morale;

pub fn process_scout_telemetry_system(
    mut events: EventReader<ScoutScanEvent>,
    mut telemetry: ResMut<MapTelemetry>,
    scout_query: Query<&Morale>,
) {
    for event in events.read() {
        let Ok(morale) = scout_query.get(event.scout_entity) else { continue; };

        let mut reported_data = event.true_data.clone();

        // Falsify if morale is critically low (< 20.0)
        if morale.value < 20.0 {
            reported_data.is_lush = true;
            reported_data.danger_level = reported_data.danger_level / 2;
            reported_data.resource_richness = reported_data.resource_richness.max(80);
            reported_data.verified = false;
        } else {
            // High morale verifies the data
            reported_data.verified = true;
        }

        telemetry.insert_node_data(event.node, reported_data);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Extract the falsification logic into a separate strategy pattern or dedicated function so different factions or situations can dictate how the data is falsified (e.g., hiding hostiles vs hiding resources).
- **Code Smells:** Magic numbers in the `process_scout_telemetry_system` (e.g., `< 20.0`, `max(80)`). These should be pulled into a configuration resource.
- **Performance:** Hash map lookups and inserts are fast enough for the expected number of nodes, but we should consider how this scales if nodes number in the thousands.
- **API Improvements:** Include an event when telemetry is verified to be false, triggering a possible `AddChronicleEvent` or faction unrest.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Telemetry data is verifiably altered when scanned by low-morale scouts.

## 7. Technical Guidance
- **Code Structure:** Add this to `src/layer2/cartographers_curse.rs` or a new `src/layer2/mutiny.rs` depending on logical fit, extending the existing `MapTelemetry` resource.
- **Integration Points:** You will need to hook into the existing exploration logic and tie it to Layer 1 morale systems. You might need to bridge Layer 1 `Pop` morale to Layer 2 `ScoutShip` entities (e.g., averaging the morale of the crew).
- **Gotchas:** Ensure that when reading `MapTelemetry` in other systems, you account for `verified: false`. The UI should perhaps render unverified data differently (e.g., with a question mark or italics).

## 8. Questions
*Builder: add questions here if spec is unclear.*
