# Bureaucratic Roulette (Spec 483)

## Overview
Beggars can't be choosers. The Empire helps, but they don't care. You can request emergency aid from the Core Worlds. Instead of choosing what you get, you receive a "Care Package" based on *their* surplus. It might be food, or it might be 500 crates of "Formal Wear". This forces players to adapt to random influxes of items and dismantle unwanted items for basic resources when starving.

## Dependencies
- `039` Trade System (Implemented)
- `046` Notifications System (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_request_aid_spawns_random_care_package_event() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<Events<EmergencyAidRequestedEvent>>();
        app.init_resource::<Events<CarePackageArrivalEvent>>();
        app.add_systems(Update, handle_aid_request_system);

        // Act
        app.world_mut().send_event(EmergencyAidRequestedEvent);
        app.update();

        // Assert
        let events = app.world().resource::<Events<CarePackageArrivalEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).len(), 1, "A CarePackageArrivalEvent should be triggered.");
    }

    #[test]
    fn test_care_package_arrives_with_items() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<Events<CarePackageArrivalEvent>>();
        app.init_resource::<ColonyInventory>();
        app.add_systems(Update, process_care_package_system);

        // Act
        let items = vec![ItemType::FormalWear, ItemType::FormalWear];
        app.world_mut().send_event(CarePackageArrivalEvent { items: items.clone() });
        app.update();

        // Assert
        let inventory = app.world().resource::<ColonyInventory>();
        assert!(inventory.get_count(&ItemType::FormalWear) >= 2, "Items from the care package should be added to inventory.");
    }

    #[test]
    fn test_care_package_cooldown() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<Events<EmergencyAidRequestedEvent>>();
        app.init_resource::<Events<CarePackageArrivalEvent>>();
        app.init_resource::<AidCooldown>();
        app.add_systems(Update, handle_aid_request_system);

        // Put on cooldown
        app.world_mut().resource_mut::<AidCooldown>().ticks_remaining = 100;

        // Act
        app.world_mut().send_event(EmergencyAidRequestedEvent);
        app.update();

        // Assert
        let events = app.world().resource::<Events<CarePackageArrivalEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).len(), 0, "No care package should arrive if on cooldown.");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

// In src/layer2/trade/aid.rs or similar

#[derive(Event)]
pub struct EmergencyAidRequestedEvent;

#[derive(Event)]
pub struct CarePackageArrivalEvent {
    pub items: Vec<crate::layer1::inventory::ItemType>,
}

#[derive(Resource, Default)]
pub struct AidCooldown {
    pub ticks_remaining: u32,
}

pub fn handle_aid_request_system(
    mut request_events: EventReader<EmergencyAidRequestedEvent>,
    mut arrival_events: EventWriter<CarePackageArrivalEvent>,
    mut cooldown: ResMut<AidCooldown>,
) {
    for _ in request_events.read() {
        if cooldown.ticks_remaining > 0 {
            continue;
        }

        // Simulate random surplus - simplistic MVP logic
        let mut items = Vec::new();
        items.push(crate::layer1::inventory::ItemType::FormalWear);
        items.push(crate::layer1::inventory::ItemType::FormalWear);

        arrival_events.send(CarePackageArrivalEvent { items });

        cooldown.ticks_remaining = 10000; // Put on cooldown
    }
}

pub fn process_care_package_system(
    mut arrival_events: EventReader<CarePackageArrivalEvent>,
    mut inventory: ResMut<crate::layer1::inventory::ColonyInventory>,
) {
    for event in arrival_events.read() {
        for item in &event.items {
            inventory.add_item(item.clone(), 1);
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- Use a robust random number generator (e.g., `rand` crate or a deterministic seeded RNG) to determine the contents of the `CarePackageArrivalEvent`.
- Define a "Surplus Table" resource or config to weight certain items (e.g., `FormalWear` 40%, `Food` 10%, `Plasteel` 50%).
- Ensure `AidCooldown` decreases over time in a separate system (e.g., `update_aid_cooldown_system`).
- Hook into the Notification System to inform the player what absurd items they just received.

## Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A player can trigger an `EmergencyAidRequestedEvent`.
- [ ] Aid cannot be requested constantly (Cooldown applies).
- [ ] The received `CarePackageArrivalEvent` populates the colony inventory with semi-random, potentially useless items.

## Technical Guidance
- Ensure events are registered in `src/setup.rs` (`app.add_event::<EmergencyAidRequestedEvent>()`, etc.) or where events are initialized.
- Ensure `update_event_buffer` cleans up the new events if not using `app.add_event`. (Follow the existing codebase conventions for event cleanup).
- Integrate `process_care_package_system` and `handle_aid_request_system` into the appropriate schedule.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
