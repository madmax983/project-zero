# 1031: The Mycelial Network

## 1. Overview
The Mycelial Network treats the solar system as a single living organism. Invisible "Bio-Links" connect planetary bodies at Layer 2. Excessive industrial damage or harvesting on one node triggers "Immune Responses" (spawning aggressive Space Fauna) at connected nodes, which then follow the links back to attack the player's operations. This forces players to balance their rapid expansion against the system's interconnected ecological threshold.

## 2. Dependencies
- Layer 2 `SystemMap` (Nodes and Links).
- Layer 1/2 `Ecology`/`Harvesting` tracking (Ecological damage).
- Layer 2 `Fleet` spawning (Space Fauna).
- Layer 2 `Navigation` (Fauna following links).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system::{SystemNode, BioLink};
    use crate::layer1::ecology::{EcologicalDamage, HarvestEvent};
    use crate::layer2::fleet::{Fleet, SpaceFauna, FleetCommand};

    #[test]
    fn test_harvesting_triggers_immune_response_at_connected_node() {
        let mut app = App::new();
        app.add_event::<HarvestEvent>();
        app.add_systems(Update, evaluate_ecological_damage_system);

        // Setup Nodes
        let node_a = app.world_mut().spawn((
            SystemNode { name: "Asteroid Belt".to_string() },
            EcologicalDamage { value: 0.0, threshold: 100.0 },
        )).id();

        let node_b = app.world_mut().spawn((
            SystemNode { name: "Gas Giant".to_string() },
        )).id();

        // Connect them with a BioLink
        app.world_mut().spawn(BioLink { source: node_a, target: node_b });
        app.world_mut().spawn(BioLink { source: node_b, target: node_a });

        // Trigger massive harvest on Node A
        app.world_mut().resource_mut::<Events<HarvestEvent>>().send(HarvestEvent {
            node: node_a,
            amount: 150.0, // Exceeds threshold
        });

        app.update();

        // Verify Space Fauna spawned at Node B targeting Node A
        let mut found_fauna = false;
        let mut query = app.world_mut().query::<(&SpaceFauna, &FleetCommand)>();
        for (_fauna, command) in query.iter(app.world()) {
            if let FleetCommand::MoveTo { destination } = command {
                if *destination == node_a {
                    found_fauna = true;
                    break;
                }
            }
        }

        assert!(found_fauna, "Massive harvesting at Node A should spawn immune response Fauna at connected Node B that targets Node A.");

        let damage_a = app.world().get::<EcologicalDamage>(node_a).unwrap();
        assert_eq!(damage_a.value, 0.0, "Ecological damage should reset after triggering an immune response.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/mycelial_network.rs
use bevy::prelude::*;
use crate::layer2::system::{SystemNode, BioLink};
use crate::layer1::ecology::{EcologicalDamage, HarvestEvent};
use crate::layer2::fleet::{Fleet, SpaceFauna, FleetCommand};

pub fn evaluate_ecological_damage_system(
    mut commands: Commands,
    mut events: EventReader<HarvestEvent>,
    mut damage_query: Query<&mut EcologicalDamage>,
    link_query: Query<&BioLink>,
) {
    for event in events.read() {
        if let Ok(mut damage) = damage_query.get_mut(event.node) {
            damage.value += event.amount;

            if damage.value >= damage.threshold {
                // Trigger immune response
                // Find a connected node to spawn from
                let mut spawn_node = None;
                for link in link_query.iter() {
                    if link.source == event.node {
                        spawn_node = Some(link.target);
                        break;
                    }
                }

                if let Some(spawner) = spawn_node {
                    // Spawn Fauna
                    commands.spawn((
                        Fleet,
                        SpaceFauna { strength: 100 }, // MVP strength
                        FleetCommand::MoveTo { destination: event.node },
                    ));

                    // Reset threshold
                    damage.value = 0.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Link Discovery:** `BioLink` entities should initially be invisible to the player. They need a "Scan" or "Science Ship" mechanic to reveal them so the player can plan around them.
- **Fauna Behavior:** When the `SpaceFauna` arrives at the targeted node, it needs specific combat logic to attack mining infrastructure rather than just sitting there.
- **Ecological Recovery:** The `EcologicalDamage` value should naturally decay over time if the player halts harvesting, allowing for sustainable "pulse" mining strategies.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_harvesting_triggers_immune_response_at_connected_node` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `HarvestEvent` must be tied directly to Layer 2 orbital mining or deep-crust planetary extraction systems.
- The `FleetCommand::MoveTo` enum variant is assumed; adapt it to match whatever navigation command struct Layer 2 fleets currently use.

## 8. Questions
*Builder: add questions here if spec is unclear.*
