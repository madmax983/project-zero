# 267: The Stowaway

## 1. Overview
The **Stowaway** system adds a layer of unpredictability to ship arrivals. Instead of every arriving entity being neatly manifested, incoming ships have a chance to offload undocumented entities (e.g., refugees, criminals, or alien fauna). These entities do not immediately appear in the colony's manifest or UI. They initially act in the shadows (consuming resources secretly or causing minor sabotage) until they reveal themselves or are discovered by security.

## 2. Dependencies
- `004` Pop Entity
- `119` Airlock & Pressure (for tracking where ships arrive)
- `046` Notifications System (for mysterious events like missing food)

## 3. RED Phase: Tests First

```rust
// tests/stowaway_tests.rs

use bevy::prelude::*;
use scale::layer1::stowaway::{Stowaway, stowaway_system, StowawayStatus};
use scale::layer1::pop::Pop;
use scale::layer1::inventory::Inventory;
use scale::layer1::items::ItemType;
use scale::layer2::trade::ShipArrivalEvent;

#[test]
fn test_stowaway_spawns_hidden_on_ship_arrival() {
    let mut app = App::new();
    app.add_event::<ShipArrivalEvent>();
    app.add_systems(Update, scale::layer1::stowaway::spawn_stowaway_system);

    // Act
    app.world.send_event(ShipArrivalEvent {
        port_entity: Entity::from_raw(1),
        has_stowaway: true,
    });
    app.update();

    // Assert
    let stowaways = app.world.query::<&Stowaway>().iter(&app.world).count();
    assert_eq!(stowaways, 1, "A stowaway should spawn when a ship arrives with one.");

    // Ensure they don't have the normal `Pop` tag yet, to avoid showing up in manifests
    let hidden_pops = app.world.query_filtered::<&Stowaway, Without<Pop>>().iter(&app.world).count();
    assert_eq!(hidden_pops, 1, "Stowaways should be hidden from the main population list.");
}

#[test]
fn test_stowaway_consumes_resources_secretly() {
    let mut app = App::new();
    app.add_systems(Update, stowaway_system);

    let storage = app.world.spawn(Inventory::new(100)).id();
    app.world.get_mut::<Inventory>(storage).unwrap().add(ItemType::Food, 10).unwrap();

    let stowaway = app.world.spawn(Stowaway {
        status: StowawayStatus::Hidden,
        hunger: 100.0, // Very hungry
        ..default()
    }).id();

    // Act
    app.update();

    // Assert
    let inv = app.world.get::<Inventory>(storage).unwrap();
    assert!(inv.count(ItemType::Food) < 10, "Stowaway should consume food from storage.");
}

#[test]
fn test_stowaway_reveals_self_over_time() {
    let mut app = App::new();
    app.add_systems(Update, stowaway_system);

    let stowaway = app.world.spawn(Stowaway {
        status: StowawayStatus::Hidden,
        confidence: 100.0, // Max confidence
        ..default()
    }).id();

    // Act
    app.update();

    // Assert
    assert!(app.world.get::<Pop>(stowaway).is_some(), "Stowaway should gain the Pop component when revealed.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/stowaway.rs

use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::inventory::Inventory;
use crate::layer1::items::ItemType;
use crate::layer2::trade::ShipArrivalEvent;

#[derive(Component, Default)]
pub struct Stowaway {
    pub status: StowawayStatus,
    pub hunger: f32,
    pub confidence: f32,
}

#[derive(Default, PartialEq, Eq)]
pub enum StowawayStatus {
    #[default]
    Hidden,
    Revealed,
}

pub fn spawn_stowaway_system(
    mut commands: Commands,
    mut events: EventReader<ShipArrivalEvent>,
) {
    for event in events.read() {
        if event.has_stowaway {
            commands.spawn((
                Stowaway {
                    status: StowawayStatus::Hidden,
                    hunger: 0.0,
                    confidence: 0.0,
                },
                Transform::from_xyz(0.0, 0.0, 0.0), // Spawn at port
            ));
        }
    }
}

pub fn stowaway_system(
    mut commands: Commands,
    mut stowaways: Query<(Entity, &mut Stowaway)>,
    mut inventories: Query<&mut Inventory>,
) {
    for (entity, mut stowaway) in stowaways.iter_mut() {
        if stowaway.status == StowawayStatus::Hidden {
            stowaway.hunger += 1.0;
            stowaway.confidence += 0.5;

            // Secretly eat food if hungry
            if stowaway.hunger > 50.0 {
                for mut inv in inventories.iter_mut() {
                    if inv.count(ItemType::Food) > 0 {
                        let _ = inv.try_remove(ItemType::Food, 1);
                        stowaway.hunger = 0.0;
                        break;
                    }
                }
            }

            // Reveal self
            if stowaway.confidence >= 100.0 {
                stowaway.status = StowawayStatus::Revealed;
                commands.entity(entity).insert(Pop);
                // Trigger notification here
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Discovery Mechanic:** Instead of just a timer (`confidence`), integrate with Security forces. Guards patrolling near the stowaway's hiding spot could increase the chance of discovery.
- **Utility AI Integration:** Once revealed, the stowaway needs to properly integrate into the `Utility AI System`. They might have high stress or poor health.
- **Event Driven:** Firing "Missing Items" events when the stowaway steals from storage to create suspense for the player.

## 6. Acceptance Criteria (Testable!)
- [ ] Ships can occasionally spawn hidden Stowaways upon arrival.
- [ ] Hidden stowaways do not appear in the global population count (no `Pop` component).
- [ ] Stowaways eventually reveal themselves and integrate as a regular `Pop`.
- [ ] All tests in RED phase pass.
- [ ] Test coverage ≥85% for `stowaway.rs`.

## 7. Technical Guidance
- **ShipArrivalEvent Update:** You'll need to modify the Layer 2 trade/ship arrival systems to occasionally set `has_stowaway: true` (maybe a 5% chance, modified by security policies).
- **Hiding Spots:** Use the `GridPosition` to move the stowaway to a dark or unpopulated tile (like a maintenance shaft or empty warehouse) upon spawning.
- **UI:** Ensure the main UI specifically queries for `Pop` when counting colonists, so `Stowaway` entities are naturally excluded until revealed.

## 8. Questions
*Builder: add questions here if spec is unclear.*
