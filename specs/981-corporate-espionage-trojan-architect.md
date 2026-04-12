# Spec 981: Corporate Espionage: The Trojan Architect

## 1. Overview
"The Trojan Architect" lets players infiltrate a rival's architectural database via Layer 3 espionage to subtly alter a common building blueprint. When the rival empire constructs that specific building, it includes a hidden backdoor. This backdoor slowly siphons resources directly to the saboteur or acts as a hidden surveillance node, providing a massive advantage until the rival notices the inefficiency and dismantles the compromised structures.

## 2. Dependencies
- Layer 3 Espionage systems.
- Layer 1 Building placement/blueprints.
- Economy/Resource processing systems.

## 3. RED Phase: Tests First

```rust
// tests/trojan_architect_tests.rs
use bevy::prelude::*;
use crate::layer1::buildings::{Building, Blueprint, ConstructionCompleteEvent};
use crate::layer1::economy::ResourceStash;
use crate::layer3::espionage::{
    TrojanBlueprint, install_trojan_blueprint_system,
    spawn_compromised_building_system, trojan_siphon_resources_system
};

#[test]
fn test_install_trojan_blueprint() {
    let mut app = App::new();
    app.add_systems(Update, install_trojan_blueprint_system);

    // Target blueprint
    let blueprint_id = "basic_farm".to_string();
    app.world_mut().insert_resource(TrojanBlueprint {
        blueprint_id: blueprint_id.clone(),
        siphon_rate: 0.02, // 2%
        saboteur_empire: Entity::PLACEHOLDER,
    });

    app.update();

    let trojan = app.world().resource::<TrojanBlueprint>();
    assert_eq!(trojan.blueprint_id, "basic_farm");
}

#[test]
fn test_building_construction_spawns_compromised_building() {
    let mut app = App::new();
    app.add_event::<ConstructionCompleteEvent>();
    app.insert_resource(TrojanBlueprint {
        blueprint_id: "basic_farm".to_string(),
        siphon_rate: 0.02,
        saboteur_empire: Entity::PLACEHOLDER,
    });
    app.add_systems(Update, spawn_compromised_building_system);

    let building_entity = app.world_mut().spawn(Building { blueprint_id: "basic_farm".to_string() }).id();

    app.world_mut().send_event(ConstructionCompleteEvent { entity: building_entity });

    app.update();

    // Check if building gained the Compromised tag/component
    use crate::layer3::espionage::CompromisedBuilding;
    assert!(app.world().get::<CompromisedBuilding>(building_entity).is_some(), "Building constructed from trojan blueprint should be compromised.");
}

#[test]
fn test_trojan_siphons_resources() {
    let mut app = App::new();
    app.add_systems(Update, trojan_siphon_resources_system);

    use crate::layer3::espionage::CompromisedBuilding;
    use crate::layer1::economy::ResourceProduction;

    let saboteur = app.world_mut().spawn(ResourceStash::default()).id();

    // Spawn a compromised building producing 100 food
    app.world_mut().spawn((
        Building { blueprint_id: "basic_farm".to_string() },
        ResourceProduction { amount: 100.0, resource_type: "food".to_string() },
        CompromisedBuilding { siphon_rate: 0.02, saboteur_empire: saboteur }
    ));

    app.update(); // Trigger siphon

    // Original production should be 98 (2% less) and saboteur stash should have 2
    let stash = app.world().get::<ResourceStash>(saboteur).unwrap();
    assert_eq!(stash.get_amount("food"), 2.0, "Saboteur should receive siphoned resources.");
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `TrojanBlueprint` resource or component (if tied to a specific rival faction).
- Create `CompromisedBuilding` component.
- `install_trojan_blueprint_system`: Registers the trojan in the targeted faction's blueprint database.
- `spawn_compromised_building_system`: Listens for `ConstructionCompleteEvent`. If the constructed building's `blueprint_id` matches the `TrojanBlueprint`, attach the `CompromisedBuilding` component to it.
- `trojan_siphon_resources_system`: During resource production calculations, query `CompromisedBuilding`s. Reduce their output by `siphon_rate` and credit that exact amount to the `saboteur_empire`'s global `ResourceStash`.

## 5. REFACTOR Phase: Quality & Design
- **Traceability:** Players dismantling a compromised building should have a chance to discover the Trojan. Create an event upon dismantling that checks for the `CompromisedBuilding` tag and alerts the victim.
- **Resource Processing:** Ensure the siphon happens strictly *after* production is generated but *before* it is added to the victim's local stash, avoiding economy desyncs.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85%.
- [ ] Trojan blueprints can be registered.
- [ ] Buildings built from trojans automatically become compromised.
- [ ] Compromised buildings secretly divert a % of output to the saboteur.

## 7. Technical Guidance
- **Discovery Mechanism:** If a player uses an "Inspect" or "Audit" tool on a building, it might reveal the `CompromisedBuilding` component.

## 8. Questions
*Builder: Add questions here regarding the exact implementation of the resource production pipeline.*
