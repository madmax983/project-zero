# Specification: 813 The FTL Sabotage

## 1. Overview
**Layer:** 3 -> 2
**Fantasy:** Weaponizing the very fabric of space travel to strand your enemies.
**Mechanic:** You can deploy stealth ships to subtly alter the physics of specific hyperlanes, causing any rival ships that use them to exit FTL travel light-years off course, often in hazardous star systems.

This feature allows factions to sabotage `Hyperlane` entities. When a rival fleet travels through a sabotaged hyperlane, they are misdirected to an unintended, potentially hazardous destination system instead of their intended target.

## 2. Dependencies
- `Hyperlane` component
- `StarSystem` component
- `FleetMovement` and `ProcessArrival` systems
- Stealth or sabotage action triggers

## 3. RED Phase: Tests First

```rust
// tests/layer3/ftl_sabotage.rs

use bevy::prelude::*;
use scale::layer3::hyperlane::{Hyperlane, SabotagedHyperlane};
use scale::layer3::fleet::{Fleet, FleetMovementEvent, FleetArrivalEvent};

#[test]
fn test_sabotage_hyperlane() {
    let mut app = App::new();
    // Setup systems and events
    app.add_event::<SabotageHyperlaneEvent>();

    let lane_entity = app.world.spawn(Hyperlane {
        source: Entity::PLACEHOLDER,
        destination: Entity::PLACEHOLDER,
    }).id();

    // Act
    app.world.resource_mut::<Events<SabotageHyperlaneEvent>>().send(SabotageHyperlaneEvent {
        hyperlane: lane_entity,
        misdirect_destination: Entity::PLACEHOLDER,
    });
    app.update();

    // Assert
    assert!(app.world.entity(lane_entity).contains::<SabotagedHyperlane>());
}

#[test]
fn test_fleet_misdirected_by_sabotaged_hyperlane() {
    let mut app = App::new();

    let bad_destination = app.world.spawn(StarSystem::default()).id();
    let lane_entity = app.world.spawn((
        Hyperlane {
            source: Entity::PLACEHOLDER,
            destination: Entity::PLACEHOLDER,
        },
        SabotagedHyperlane {
            misdirect_target: bad_destination,
        }
    )).id();

    let fleet_entity = app.world.spawn(Fleet::default()).id();

    // Act
    app.world.resource_mut::<Events<FleetMovementEvent>>().send(FleetMovementEvent {
        fleet: fleet_entity,
        hyperlane: lane_entity,
    });
    app.update();

    // Assert
    // Check that the fleet arrival event targets the bad destination, not the original one
    let events = app.world.resource::<Events<FleetArrivalEvent>>();
    let mut reader = events.get_reader();
    let arrival = reader.iter(events).next().unwrap();
    assert_eq!(arrival.destination, bad_destination);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/hyperlane.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct SabotagedHyperlane {
    pub misdirect_target: Entity,
}

#[derive(Event)]
pub struct SabotageHyperlaneEvent {
    pub hyperlane: Entity,
    pub misdirect_destination: Entity,
}

pub fn process_hyperlane_sabotage(
    mut commands: Commands,
    mut events: EventReader<SabotageHyperlaneEvent>,
) {
    for event in events.read() {
        commands.entity(event.hyperlane).insert(SabotagedHyperlane {
            misdirect_target: event.misdirect_destination,
        });
    }
}

// In your fleet movement system:
// When routing a fleet through a hyperlane, check if `SabotagedHyperlane` exists.
// If it does, use `misdirect_target` as the destination for the FleetArrivalEvent.
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: Integrate the stealth check into the `SabotageHyperlaneEvent` to ensure only valid stealth ships can trigger it.
- **Code Smells**: Ensure `misdirect_target` is a valid `StarSystem` entity to avoid crashes on arrival.
- **Performance Considerations**: Querying `SabotagedHyperlane` during every fleet movement could be slightly heavier; ensure component queries are efficient.
- **API Improvements**: Add a decay or detection mechanism so the sabotage doesn't last forever.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Fleets traveling on a sabotaged hyperlane arrive at the misdirected target

## 7. Technical Guidance
- **Code structure suggestions**: Keep `SabotagedHyperlane` isolated in the layer 3 module.
- **Integration points**: Must hook into the existing `FleetMovement` and `ProcessArrival` systems in `layer3`.
- **Gotchas**: Ensure that if the misdirect target is destroyed or invalid, the game gracefully handles the fallback (e.g., reverting to standard destination or dropping out of hyperspace in deep space).

## 8. Questions
*Builder: add questions here if spec is unclear.*
