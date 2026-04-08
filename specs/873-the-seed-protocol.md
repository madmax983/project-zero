# Specification 873: The Seed Protocol

## 1. Overview
**Layer:** 2 -> 3
**Feature:** The Seed Protocol
**Fantasy:** Ensuring the survival of the species, even if it's not *you*.
**Mechanic:** Construct and launch automated "Ark Pods". They leave the system to spawn AI-controlled allied colonies.
**Emergence:** You are wiped out, but your "child" colony returns 100 years later as a Fallen Empire to avenge you.
**Tension:** Spend massive resources on a ship you can't control?

## 2. Dependencies
- Layer 2 Ships / Fleets.
- Layer 3 Galaxy Map / Factions.
- Layer 1 Construction (Megaproject/Great Work scale).

## 3. RED Phase: Tests First

```rust
// tests/seed_protocol_tests.rs
use bevy::prelude::*;
// Dummy imports for spec clarity - Builders should use actual paths
use scale::layer2::fleet::{Fleet, ShipClass, ArkPod};
use scale::layer3::faction::{Faction, Alliance};
use scale::layer3::galaxy::{SystemNode, spawn_allied_colony_system};

#[test]
fn test_ark_pod_spawns_allied_colony() {
    let mut app = App::new();
    app.add_systems(Update, spawn_allied_colony_system);

    let origin_faction = app.world_mut().spawn((
        Faction { name: "Earth".into() },
    )).id();

    // Pod successfully arrives at an empty node
    let destination_node = app.world_mut().spawn((
        SystemNode { is_empty: true },
    )).id();

    let ark_pod = app.world_mut().spawn((
        Fleet { faction_id: origin_faction },
        ShipClass::ArkPod,
        ArkPod { destination: destination_node, has_landed: true },
    )).id();

    app.update();

    // Ark Pod should be consumed to form a new colony
    assert!(app.world().get_entity(ark_pod).is_err());

    // A new allied colony should now exist
    let new_factions = app.world_mut().query::<&Faction>().iter(&app.world()).count();
    assert_eq!(new_factions, 2); // Origin + New Child

    // Find the alliance
    let alliances = app.world_mut().query::<&Alliance>().iter(&app.world()).count();
    assert_eq!(alliances, 1);
}

#[test]
fn test_ark_pod_fails_if_node_occupied() {
    let mut app = App::new();
    app.add_systems(Update, spawn_allied_colony_system);

    let origin_faction = app.world_mut().spawn((
        Faction { name: "Earth".into() },
    )).id();

    // Pod attempts to arrive at an occupied node
    let destination_node = app.world_mut().spawn((
        SystemNode { is_empty: false },
    )).id();

    let ark_pod = app.world_mut().spawn((
        Fleet { faction_id: origin_faction },
        ShipClass::ArkPod,
        ArkPod { destination: destination_node, has_landed: true },
    )).id();

    app.update();

    // Ark Pod should STILL be consumed (it crashed or was destroyed)
    assert!(app.world().get_entity(ark_pod).is_err());

    // But no new faction or alliance was formed
    let new_factions = app.world_mut().query::<&Faction>().iter(&app.world()).count();
    assert_eq!(new_factions, 1);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/seed.rs
use bevy::prelude::*;
use crate::layer2::fleet::{Fleet, ShipClass, ArkPod};
use crate::layer3::faction::{Faction, Alliance};
use crate::layer3::galaxy::SystemNode;

#[derive(Component)]
pub struct ArkPod {
    pub destination: Entity,
    pub has_landed: bool,
}

pub fn spawn_allied_colony_system(
    mut commands: Commands,
    ark_query: Query<(Entity, &Fleet, &ArkPod)>,
    mut node_query: Query<&mut SystemNode>,
) {
    for (entity, fleet, ark) in ark_query.iter() {
        if !ark.has_landed {
            continue;
        }

        if let Ok(mut node) = node_query.get_mut(ark.destination) {
            if node.is_empty {
                node.is_empty = false;

                let new_faction = commands.spawn(Faction {
                    name: "Seed Child".into(),
                }).id();

                commands.spawn(Alliance {
                    faction_a: fleet.faction_id,
                    faction_b: new_faction,
                });
            }
        }

        // Consume the ark pod whether it succeeds or fails
        commands.entity(entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Naming**: Generate a procedural name for the new faction based on the origin faction (e.g., "New Earth", "Earth Remnant").
- **Chronicle**: Hook into `ChronicleEvent` to document the departure and the eventual founding (or loss) of the Seed colony.
- **Tech Tree**: Make the `ArkPod` an expensive endgame megastructure-tier project, requiring immense resource hoarding.
- **Avenge Behavior**: In a deeper layer 3 system, have child factions prioritize attacking factions that destroyed their parent `faction_id`.

## 6. Acceptance Criteria
- [ ] Tests pass in RED phase.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes cleanly.
- [ ] Test coverage >= 85%.
- [ ] Allied faction correctly spawns and an `Alliance` component is created.

## 7. Technical Guidance
- Spawning an entirely new `Faction` dynamically might require careful indexing depending on how `Faction` states are stored in UI dashboards.
- Ensure the `Alliance` system logic (if already implemented) properly recognizes the new entity.

## 8. Questions
*Builder: add questions here if spec is unclear.*
