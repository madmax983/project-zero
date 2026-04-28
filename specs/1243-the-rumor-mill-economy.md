# 1243: The Rumor Mill Economy

## 1. Overview
**Layer:** Cross-layer (1, 3)
**Fantasy:** Trading secrets and gossip as a tangible commodity.
**Mechanic:** If a colony has a high "Rumor" density but low physical trade, an underground "Information Broker" faction emerges. They can sell secrets to rival Layer 3 empires for massive payouts, but doing so drastically increases your vulnerability to espionage and sabotage.

## 2. Dependencies
- Layer 1 Population/Rumor System
- Layer 3 Faction Diplomacy System
- Layer 3 Trade System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_information_broker_faction_emerges() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<ColonyStats>();
        app.world_mut().resource_mut::<ColonyStats>().rumor_density = 100.0;
        app.world_mut().resource_mut::<ColonyStats>().trade_volume = 10.0; // Low trade

        // Act
        app.add_systems(Update, spawn_information_brokers_system);
        app.update();

        // Assert
        let brokers_exist = app.world().query::<&InformationBrokerFaction>().iter(app.world()).count() > 0;
        assert!(brokers_exist, "High rumors and low trade should spawn Information Brokers");
    }

    #[test]
    fn test_selling_secrets_increases_vulnerability() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<ColonyStats>();
        app.add_event::<SellSecretsEvent>();

        let _faction = app.world_mut().spawn(InformationBrokerFaction).id();

        app.world_mut().send_event(SellSecretsEvent { amount: 50.0 });

        // Act
        app.add_systems(Update, resolve_secret_sales_system);
        app.update();

        // Assert
        let stats = app.world().resource::<ColonyStats>();
        assert!(stats.credits > 0.0, "Selling secrets should yield credits");
        assert!(stats.espionage_vulnerability > 0.0, "Selling secrets should increase espionage vulnerability");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ColonyStats {
    pub rumor_density: f32,
    pub trade_volume: f32,
    pub credits: f32,
    pub espionage_vulnerability: f32,
}

#[derive(Component)]
pub struct InformationBrokerFaction;

#[derive(Event)]
pub struct SellSecretsEvent {
    pub amount: f32,
}

pub fn spawn_information_brokers_system(
    mut commands: Commands,
    stats: Res<ColonyStats>,
    query: Query<(), With<InformationBrokerFaction>>,
) {
    if stats.rumor_density > 80.0 && stats.trade_volume < 20.0 {
        if query.is_empty() {
            commands.spawn(InformationBrokerFaction);
        }
    }
}

pub fn resolve_secret_sales_system(
    mut events: EventReader<SellSecretsEvent>,
    mut stats: ResMut<ColonyStats>,
) {
    for ev in events.read() {
        // Massive payout
        stats.credits += ev.amount * 10.0;
        // Massive vulnerability increase
        stats.espionage_vulnerability += ev.amount * 2.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Ensure the `InformationBrokerFaction` acts as a real Layer 1 entity or faction that pops can join.
- `SellSecretsEvent` should be tied to specific rival empires, increasing their specific intel on you rather than a global vulnerability score.
- Introduce events for rival empires utilizing the purchased secrets to perform targeted assassinations or sabotage.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Information Brokers spawn when conditions are met.
- [ ] Selling secrets generates large amounts of credits and increases vulnerability.

## 7. Technical Guidance
- **Faction Integration:** It may be beneficial to integrate `InformationBrokerFaction` directly into the existing `factions` system if it's robust enough to handle underground or emergent factions.
- **Balancing:** The payout vs vulnerability needs to feel like a deal with the devil. Ensure the consequences of `espionage_vulnerability` are felt by the player.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
