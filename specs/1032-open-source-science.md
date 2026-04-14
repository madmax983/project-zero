# 1032: Open Source Science

## 1. Overview
The "Open Source Science" mechanic allows players to publish their colony's discovery data (Maps, Alien Biology, Tech Specs) to the Galactic Layer (Layer 3). Publishing grants massive Prestige and Relations with peaceful factions, but simultaneously provides enemies with exploitable bonuses (e.g., known shield frequencies, revealed backdoor routes).

## 2. Dependencies
- Layer 1/2 `Research` and `Discovery` tracking.
- Layer 3 `Diplomacy` (Prestige, Relations).
- Layer 3 `Factions` (Hostile vs Peaceful).
- Layer 1/2 `Combat` or `Pathfinding` (Enemy bonuses).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::research::Discovery;
    use crate::layer3::diplomacy::{Faction, Relations, Prestige};
    use crate::layer1::combat::CombatStats;

    #[test]
    fn test_publishing_discovery_increases_prestige() {
        let mut app = App::new();
        app.insert_resource(Prestige { value: 100 });
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_publication_system);

        let discovery = app.world_mut().spawn(Discovery {
            data_type: "ShieldFrequency".to_string(),
            value: 50
        }).id();

        app.world_mut().resource_mut::<Events<PublishDiscoveryEvent>>().send(PublishDiscoveryEvent {
            discovery,
        });

        app.update();

        let prestige = app.world().resource::<Prestige>();
        assert_eq!(prestige.value, 150, "Publishing a discovery should increase the colony's global prestige.");
    }

    #[test]
    fn test_published_discovery_gives_enemies_combat_bonus() {
        let mut app = App::new();
        app.add_event::<PublishDiscoveryEvent>();
        app.add_systems(Update, process_enemy_exploits_system);

        let discovery = app.world_mut().spawn(Discovery {
            data_type: "ShieldFrequency".to_string(),
            value: 50
        }).id();

        // Hostile Pirate Faction
        let pirate = app.world_mut().spawn((
            Faction { is_hostile: true },
            CombatStats { attack_bonus: 0.0 },
        )).id();

        app.world_mut().resource_mut::<Events<PublishDiscoveryEvent>>().send(PublishDiscoveryEvent {
            discovery,
        });

        app.update();

        let stats = app.world().get::<CombatStats>(pirate).unwrap();
        assert!(stats.attack_bonus > 0.0, "Hostile factions should receive an attack bonus when tactical data is published.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer3/open_source_science.rs
use bevy::prelude::*;
use crate::layer1::research::Discovery;
use crate::layer3::diplomacy::{Faction, Prestige};
use crate::layer1::combat::CombatStats;

#[derive(Event)]
pub struct PublishDiscoveryEvent {
    pub discovery: Entity,
}

pub fn process_publication_system(
    mut events: EventReader<PublishDiscoveryEvent>,
    query: Query<&Discovery>,
    mut prestige: ResMut<Prestige>,
) {
    for event in events.read() {
        if let Ok(discovery) = query.get(event.discovery) {
            // Add value to prestige
            prestige.value += discovery.value;
        }
    }
}

pub fn process_enemy_exploits_system(
    mut events: EventReader<PublishDiscoveryEvent>,
    discovery_query: Query<&Discovery>,
    mut hostile_query: Query<(&Faction, &mut CombatStats)>,
) {
    for event in events.read() {
        if let Ok(discovery) = discovery_query.get(event.discovery) {
            // MVP: If it's a tactical discovery, give all hostiles a buff
            if discovery.data_type == "ShieldFrequency" || discovery.data_type == "MapData" {
                for (faction, mut stats) in hostile_query.iter_mut() {
                    if faction.is_hostile {
                        stats.attack_bonus += 10.0;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Data Types:** A published "MapData" shouldn't grant an `attack_bonus`; it should allow enemy fleets in Layer 2/3 to bypass standard pathfinding chokepoints or reveal hidden systems.
- **Relations:** Publishing should also modify `Relations` with specific factions. A faction of scholars might love you for publishing biology data, while a militant faction might view you as naive.
- **Irreversible:** Once an event is published, the entity should probably be marked `Published: true` to prevent spamming the event for infinite prestige.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_publishing_discovery_increases_prestige` passes.
- [ ] Test `test_published_discovery_gives_enemies_combat_bonus` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Integration across layers is key. The `PublishDiscoveryEvent` likely originates in the Layer 1 UI but impacts Layer 3 `Prestige` and Layer 1/2 `CombatStats`.
- Ensure `CombatStats` changes are persistent for the specific hostile entities or applied globally via a resource to all future hostile spawns.

## 8. Questions
*Builder: add questions here if spec is unclear.*
