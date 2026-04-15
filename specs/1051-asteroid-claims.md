# 1051: Asteroid Claims

**Layer:** 2
**Fantasy:** The gold rush in the sky. Selling shovels instead of digging.

## 1. Overview
The colony can survey asteroids and claim "Mining Rights". Rather than dedicating internal colony resources to build a mine and maintain it, players can "sell" these rights to external factions or independent orbital prospectors. In return, the colony receives a continuous royalty stream of extracted resources but fully cedes control over the asteroid itself. This forces a tension: guaranteed immediate but slow royalty trickle, versus the full control but resource-heavy direct mining approach. It also creates a vector for emergent betrayal if the group claiming the asteroid turns out to be hostile and uses the extracted resources against the player.

## 2. Dependencies
- Must have basic Asteroid entity definitions (e.g., `Asteroid` component).
- Requires a representation of Factions or external entities (e.g., `Faction` component).
- Needs the Layer 1 resource stockpile system to receive royalties.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_selling_asteroid_claim_assigns_faction_and_generates_royalties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (sell_asteroid_claim_system, process_royalties_system));
        app.init_resource::<ColonyStockpile>();

        let external_faction = app.world_mut().spawn(Faction {
            name: "Independent Belters".to_string(),
            hostility_level: 0,
        }).id();

        let asteroid = app.world_mut().spawn((
            Asteroid,
            ResourceYield { resource_type: ResourceType::Uranium, amount_per_cycle: 100 },
        )).id();

        // Act - Player sells the claim
        app.world_mut().send_event(SellClaimEvent {
            asteroid_entity: asteroid,
            buyer_entity: external_faction,
            royalty_percentage: 0.10, // 10%
        });

        app.update();

        // Assert - Asteroid now belongs to the faction
        let claim = app.world().get::<AsteroidClaim>(asteroid).expect("Asteroid should have a claim");
        assert_eq!(claim.owner, external_faction);
        assert_eq!(claim.royalty_percentage, 0.10);

        // Act - Simulate a production cycle
        app.world_mut().send_event(ProductionCycleEvent);
        app.update();

        // Assert - Colony receives 10% of 100 Uranium (10)
        let stockpile = app.world().get_resource::<ColonyStockpile>().unwrap();
        assert_eq!(stockpile.get_amount(ResourceType::Uranium), 10);
    }

    #[test]
    fn test_hostile_faction_uses_asteroid_resources_for_attack() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (process_faction_behavior_system));
        app.add_event::<AttackColonyEvent>();

        let hostile_faction = app.world_mut().spawn((
            Faction { name: "Terror Front".to_string(), hostility_level: 100 },
            FactionInventory { uranium_stored: 90 }, // 10 went to royalties
        )).id();

        // Act - Faction processes its inventory
        app.update();

        // Assert - Faction has enough uranium to trigger an attack event
        let events = app.world().resource::<Events<AttackColonyEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).any(|e| e.attacker == hostile_faction));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Asteroid;

#[derive(Component, Clone)]
pub struct ResourceYield {
    pub resource_type: ResourceType,
    pub amount_per_cycle: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResourceType {
    Uranium,
    Iron,
}

#[derive(Component)]
pub struct Faction {
    pub name: String,
    pub hostility_level: u32,
}

#[derive(Component)]
pub struct FactionInventory {
    pub uranium_stored: u32,
}

#[derive(Component)]
pub struct AsteroidClaim {
    pub owner: Entity,
    pub royalty_percentage: f32,
}

#[derive(Event)]
pub struct SellClaimEvent {
    pub asteroid_entity: Entity,
    pub buyer_entity: Entity,
    pub royalty_percentage: f32,
}

#[derive(Event)]
pub struct ProductionCycleEvent;

#[derive(Event)]
pub struct AttackColonyEvent {
    pub attacker: Entity,
}

#[derive(Resource, Default)]
pub struct ColonyStockpile {
    uranium: u32,
    iron: u32,
}

impl ColonyStockpile {
    pub fn get_amount(&self, res: ResourceType) -> u32 {
        match res {
            ResourceType::Uranium => self.uranium,
            ResourceType::Iron => self.iron,
        }
    }
    pub fn add(&mut self, res: ResourceType, amount: u32) {
        match res {
            ResourceType::Uranium => self.uranium += amount,
            ResourceType::Iron => self.iron += amount,
        }
    }
}

pub fn sell_asteroid_claim_system(
    mut commands: Commands,
    mut events: EventReader<SellClaimEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.asteroid_entity).insert(AsteroidClaim {
            owner: ev.buyer_entity,
            royalty_percentage: ev.royalty_percentage,
        });
    }
}

pub fn process_royalties_system(
    mut events: EventReader<ProductionCycleEvent>,
    query: Query<(&AsteroidClaim, &ResourceYield)>,
    mut stockpile: ResMut<ColonyStockpile>,
) {
    for _ in events.read() {
        for (claim, yield_data) in query.iter() {
            let royalty_amount = (yield_data.amount_per_cycle as f32 * claim.royalty_percentage).round() as u32;
            stockpile.add(yield_data.resource_type, royalty_amount);
            // In a fuller implementation, we'd add the remaining 90% to the faction's inventory
        }
    }
}

pub fn process_faction_behavior_system(
    mut factions: Query<(Entity, &Faction, &mut FactionInventory)>,
    mut attack_events: EventWriter<AttackColonyEvent>,
) {
    for (entity, faction, mut inventory) in factions.iter_mut() {
        if faction.hostility_level > 50 && inventory.uranium_stored >= 50 {
            // Faction builds a weapon and attacks
            inventory.uranium_stored -= 50;
            attack_events.send(AttackColonyEvent { attacker: entity });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extrapolate faction intent logic into a broader `Utility AI` system so attacks are not hardcoded to "Uranium >= 50".
- Make `ColonyStockpile` a map instead of hardcoded fields so it scales with new resource types.
- Move the event cleanup into `cleanup.rs` as per SCALE architecture rules.
- Add an integration system in `core/integration.rs` to pipe these claims into the trade and diplomatic UI.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Asteroid ownership safely transfers and colony properly accrues partial royalty per cycle.
- [ ] Hostile factions can militarize gathered resources against the player.

## 7. Technical Guidance
- When modifying the stockpile, check if `ColonyStockpile` already exists in `layer1::economy` or similar. If so, reuse it.
- Ensure the `Asteroid` entity is fully unregistered from colony logic (like colony auto-miners) when `AsteroidClaim` is attached.
- Faction identities and intentions should be robust enough that selling to the wrong people genuinely feels like a consequence of bad intelligence.

## 8. Questions
*Builder: add questions here if spec is unclear.*
