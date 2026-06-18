# 1308: The Blackout Bazaars

## Overview

Implementing "The Blackout Bazaars" mechanic. When power fails or a "Blackout Protocol" is initiated, unique temporary "Bazaar" zones spontaneously form in unpowered social areas. Pops trade rare items, secrets, and contraband that they refuse to sell when the lights are on and the cameras are watching. These Bazaars disappear instantly when power is restored. It creates a tension: intentionally destabilize your power grid to access the black market, risking safety, or maintain a secure but mundane economy.

## Dependencies

- Existing Layer 1 Pop mechanics, `PowerConsumer` components, and the power grid systems.
- Existing `BlackoutState` from Spec 803 or generic grid failure mechanics.
- `Item` and inventory components.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_blackout_spawns_bazaar_in_social_area() {
        let mut app = App::new();
        app.add_systems(Update, spawn_blackout_bazaars_system);

        let social_area = app.world_mut().spawn((
            SocialArea,
            PowerConsumer { demand: 5.0, active: false, is_powered: false },
        )).id();

        // Act
        app.update();

        // Assert: A BlackoutBazaar should be attached to the unpowered social area
        assert!(app.world().get::<BlackoutBazaar>(social_area).is_some());
    }

    #[test]
    fn test_power_restoration_despawns_bazaar() {
        let mut app = App::new();
        app.add_systems(Update, despawn_blackout_bazaars_system);

        let social_area = app.world_mut().spawn((
            SocialArea,
            PowerConsumer { demand: 5.0, active: true, is_powered: true },
            BlackoutBazaar { active: true },
        )).id();

        // Act
        app.update();

        // Assert: The BlackoutBazaar component should be removed since power is restored
        assert!(app.world().get::<BlackoutBazaar>(social_area).is_none());
    }

    #[test]
    fn test_bazaar_trade_opportunity() {
        let mut app = App::new();
        app.add_systems(Update, bazaar_trading_system);

        let bazaar = app.world_mut().spawn((
            BlackoutBazaar { active: true },
            BazaarInventory {
                rare_items: vec![RareItem::FounderRifle],
                required_trade: RareItem::FuelCell,
            }
        )).id();

        let player_trade_event = PlayerTradeEvent {
            bazaar_entity: bazaar,
            offered_item: RareItem::FuelCell,
        };
        app.world_mut().send_event(player_trade_event);

        // Act
        app.update();

        // Assert: The trade should complete successfully, removing the required item
        let inventory = app.world().get::<BazaarInventory>(bazaar).unwrap();
        assert!(inventory.rare_items.is_empty(), "Item should be traded");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SocialArea;

#[derive(Component)]
pub struct PowerConsumer {
    pub demand: f32,
    pub active: bool,
    pub is_powered: bool,
}

#[derive(Component)]
pub struct BlackoutBazaar {
    pub active: bool,
}

#[derive(Component)]
pub struct BazaarInventory {
    pub rare_items: Vec<RareItem>,
    pub required_trade: RareItem,
}

#[derive(PartialEq, Clone, Debug)]
pub enum RareItem {
    FounderRifle,
    FuelCell,
    Contraband,
}

#[derive(Event)]
pub struct PlayerTradeEvent {
    pub bazaar_entity: Entity,
    pub offered_item: RareItem,
}

pub fn spawn_blackout_bazaars_system(
    mut commands: Commands,
    query: Query<(Entity, &PowerConsumer), (With<SocialArea>, Without<BlackoutBazaar>)>,
) {
    for (entity, power) in query.iter() {
        // If unpowered, a bazaar forms
        if !power.is_powered {
            commands.entity(entity).insert(BlackoutBazaar { active: true });
        }
    }
}

pub fn despawn_blackout_bazaars_system(
    mut commands: Commands,
    query: Query<(Entity, &PowerConsumer), With<BlackoutBazaar>>,
) {
    for (entity, power) in query.iter() {
        // If powered, the bazaar scatters
        if power.is_powered {
            commands.entity(entity).remove::<BlackoutBazaar>();
            commands.entity(entity).remove::<BazaarInventory>();
        }
    }
}

pub fn bazaar_trading_system(
    mut events: EventReader<PlayerTradeEvent>,
    mut query: Query<&mut BazaarInventory, With<BlackoutBazaar>>,
) {
    for event in events.read() {
        if let Ok(mut inventory) = query.get_mut(event.bazaar_entity) {
            if inventory.required_trade == event.offered_item {
                inventory.rare_items.clear(); // Trade successful
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **RNG Inventory Generation**: The items spawned in `BazaarInventory` should be drawn from a weighted loot table based on the colony's current deficits or historical events, rather than being hardcoded.
- **Risk Mechanic**: Add a chance for a "Raid" by Enforcer Pops during a Blackout Bazaar, which could confiscate traded items or cause the bazaar to close prematurely.
- **Duration Delay**: Instead of spawning instantly, bazaars should perhaps take a few ticks to "setup" after power fails, representing the time it takes for Pops to gather.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Bazaars only spawn in unpowered `SocialArea` entities.
- [ ] Bazaars disappear instantly when power is restored.
- [ ] Player can execute trades if offering the correctly requested item.

## Technical Guidance

### Components
- `BlackoutBazaar`: Marks a location currently hosting an illicit market.
- `BazaarInventory`: Stores what the bazaar is selling and what it wants in return.

### Systems
- `spawn_blackout_bazaars_system`: Listens for unpowered social areas and creates bazaars.
- `despawn_blackout_bazaars_system`: Cleans up the market when the lights come back on.
- `bazaar_trading_system`: Handles the exchange of `RareItem`s.

### Integration Points
- This connects deeply with existing power grid and grid failure logic (e.g., from Spec 803 Blackout Protocol).
- UI needs to draw a specific icon (maybe a lantern or eye) over Social Areas when a bazaar is active so the player knows where to trade.

## Questions

*Builder: add questions here if spec is unclear.*
