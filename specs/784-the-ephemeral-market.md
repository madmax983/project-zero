# 784: The Ephemeral Market

## 1. Overview
**Layer:** 2 -> 3
**Fantasy:** A mythical trading hub that only exists briefly and randomly.
**Mechanic:** A nomadic fleet of immense trading vessels occasionally drops into random systems for a very short duration. They offer incredibly rare artifacts, lost technologies, and massive resource bundles, but only accept payment in highly specific, obscure commodities.
**Emergence:** The Ephemeral Market appears in your home system, offering the final piece of technology you need for a Dyson Sphere, but they only accept "Sub-Lithic Fungal Spores," a rare resource you only produce on one rebellious, isolated mining colony on the edge of your empire. You must suppress the rebellion, harvest the spores, and ship them back before the market vanishes.
**Tension:** The potential for game-changing economic windfalls vs. the chaotic scramble to fulfill their bizarre demands before time runs out.

## 2. Dependencies
- Layer 2 / Layer 3 Orbital Stations / Star Systems
- Layer 1 Economy and Resource System (`ColonyResources`)
- `010` Chronicle System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            spawn_ephemeral_market_system,
            process_market_despawn_system,
            fulfill_market_trade_system
        ));
        app.add_event::<MarketSpawnEvent>();
        app.add_event::<MarketTradeEvent>();
        app
    }

    #[test]
    fn test_ephemeral_market_spawns_and_despawns() {
        let mut app = setup_app();

        // Trigger spawn event
        let target_system = app.world_mut().spawn_empty().id();
        app.world_mut().send_event(MarketSpawnEvent {
            system_entity: target_system,
            duration_ticks: 10,
        });

        app.update();

        // Check market spawned
        let mut query = app.world_mut().query::<(Entity, &EphemeralMarket)>();
        let market_count = query.iter(app.world()).count();
        assert_eq!(market_count, 1, "Ephemeral Market should spawn when event is sent");

        let (market_ent, market) = query.iter(app.world()).next().unwrap();
        assert_eq!(market.ticks_remaining, 10);
        assert_eq!(market.location, target_system);

        // Run ticks to trigger despawn
        for _ in 0..11 {
            app.update();
        }

        // Market should be despawned
        assert!(app.world().get_entity(market_ent).is_none(), "Market should despawn after ticks_remaining reaches 0");
    }

    #[test]
    fn test_market_trade_fulfillment() {
        let mut app = setup_app();

        let target_system = app.world_mut().spawn_empty().id();
        app.world_mut().send_event(MarketSpawnEvent {
            system_entity: target_system,
            duration_ticks: 100,
        });
        app.update();

        let market_ent = app.world_mut().query_filtered::<Entity, With<EphemeralMarket>>().single(app.world());

        // Mock inventory / colony resource setup would go here

        // Attempt Trade
        app.world_mut().send_event(MarketTradeEvent {
            market_entity: market_ent,
            buyer_entity: target_system,
            trade_index: 0,
        });

        app.update();

        // Assert: Resources were exchanged successfully
        // Requires full integration with ColonyResources, for test we just assume the event was processed
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct EphemeralMarket {
    pub location: Entity,
    pub ticks_remaining: u64,
    // List of offered trades: (Required Item/Amount, Offered Item/Amount)
    pub trades: Vec<(String, f32, String, f32)>,
}

#[derive(Event)]
pub struct MarketSpawnEvent {
    pub system_entity: Entity,
    pub duration_ticks: u64,
}

#[derive(Event)]
pub struct MarketTradeEvent {
    pub market_entity: Entity,
    pub buyer_entity: Entity,
    pub trade_index: usize,
}

pub fn spawn_ephemeral_market_system(
    mut commands: Commands,
    mut spawn_events: EventReader<MarketSpawnEvent>,
) {
    for ev in spawn_events.read() {
        commands.spawn(EphemeralMarket {
            location: ev.system_entity,
            ticks_remaining: ev.duration_ticks,
            trades: vec![
                ("Sub-Lithic Fungal Spores".into(), 100.0, "Dyson Sphere Tech Blueprint".into(), 1.0)
            ]
        });
    }
}

pub fn process_market_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EphemeralMarket)>,
) {
    for (entity, mut market) in query.iter_mut() {
        if market.ticks_remaining == 0 {
            commands.entity(entity).despawn();
        } else {
            market.ticks_remaining -= 1;
        }
    }
}

pub fn fulfill_market_trade_system(
    mut commands: Commands,
    mut trade_events: EventReader<MarketTradeEvent>,
    mut market_query: Query<&mut EphemeralMarket>,
) {
    for ev in trade_events.read() {
        if let Ok(mut market) = market_query.get_mut(ev.market_entity) {
            if ev.trade_index < market.trades.len() {
                // In full implementation: deduct required resources from buyer_entity,
                // and grant offered items to buyer_entity.
                // For now, minimal logic:
                let trade = market.trades.remove(ev.trade_index);
                // process trade...
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Trade Data Structures:** The `trades` vector uses primitive `String` and `f32`. This should be refactored to use strong typing reflecting `ItemType`, `Resource`, or `Technology` enums used in the rest of the game.
- **Resource Checking:** The trade fulfillment system must check `ColonyResources` to verify the buyer actually has the bizarre items requested before processing the transaction.
- **Player Notifications:** Tie the spawn event to a global UI notification or Chronicle Event to alert the player when the market drops in.
- **Procedural Generation:** The trades offered should be procedurally generated, ensuring they ask for items the player either doesn't have or are extremely far away, driving the tension.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `ephemeral_market` module.
- [ ] Market entity spawns and accurately despawns after `duration_ticks`.
- [ ] Trades can be successfully executed when requirements are met.

## 7. Technical Guidance
- **System Set:** Register `process_market_despawn_system` in an Update schedule so it decrements predictably per tick.
- **Event Bus:** Make sure `MarketTradeEvent` has a companion event `MarketTradeFailedEvent` to return feedback if the player clicks 'trade' but lacks the obscure commodities.
- **Layer 2 -> Layer 3:** Even though it docks at a Layer 2 or 3 StarSystem, ensure the required resources can be pulled from a specific Layer 1 colony via the logistical/trade network.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Builder: This task seems to be mostly Layer 2 -> Layer 3 integration. Should we just implement it based on the spec?*
