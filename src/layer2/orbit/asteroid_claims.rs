use bevy::prelude::*;

#[derive(Component)]
pub struct Asteroid;

#[derive(Component, Clone)]
pub struct ResourceYield {
    pub resource_type: ResourceType,
    pub amount_per_cycle: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum ResourceType {
    Uranium,
    Iron,
}

#[derive(Component)]
pub struct FactionClaim {
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
    pub map: std::collections::BTreeMap<ResourceType, u32>,
}

impl ColonyStockpile {
    pub fn get_amount(&self, res: ResourceType) -> u32 {
        self.map.get(&res).copied().unwrap_or(0)
    }
    pub fn add(&mut self, res: ResourceType, amount: u32) {
        *self.map.entry(res).or_insert(0) += amount;
    }
}

pub fn sell_asteroid_claim_system(mut commands: Commands, mut events: EventReader<SellClaimEvent>) {
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
            let royalty_amount =
                (yield_data.amount_per_cycle as f32 * claim.royalty_percentage).round() as u32;
            stockpile.add(yield_data.resource_type, royalty_amount);
            // In a fuller implementation, we'd add the remaining 90% to the faction's inventory
        }
    }
}

pub fn process_faction_behavior_system(
    mut factions: Query<(Entity, &FactionClaim, &mut FactionInventory)>,
    mut attack_events: EventWriter<AttackColonyEvent>,
) {
    for (entity, faction, mut inventory) in factions.iter_mut() {
        // More robust utility-based checking.
        // Instead of hardcoding 50 uranium, calculate a threat score based on hostility and resources.
        let resource_score = inventory.uranium_stored as f32 * 0.5;
        let threat_score = faction.hostility_level as f32 * 0.5 + resource_score;

        if threat_score > 75.0 && inventory.uranium_stored >= 50 {
            // Faction attacks if threat is high enough
            inventory.uranium_stored -= 50;
            attack_events.send(AttackColonyEvent { attacker: entity });
        }
    }
}

pub struct AsteroidClaimsPlugin;

impl Plugin for AsteroidClaimsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SellClaimEvent>();
        app.add_event::<ProductionCycleEvent>();
        app.add_event::<AttackColonyEvent>();
        app.add_systems(
            Update,
            (
                sell_asteroid_claim_system,
                process_royalties_system,
                process_faction_behavior_system,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selling_asteroid_claim_assigns_faction_and_generates_royalties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(
            Update,
            (sell_asteroid_claim_system, process_royalties_system),
        );
        app.init_resource::<ColonyStockpile>();
        app.add_event::<SellClaimEvent>();
        app.add_event::<ProductionCycleEvent>();

        let external_faction = app
            .world_mut()
            .spawn(FactionClaim {
                name: "Independent Belters".to_string(),
                hostility_level: 0,
            })
            .id();

        let asteroid = app
            .world_mut()
            .spawn((
                Asteroid,
                ResourceYield {
                    resource_type: ResourceType::Uranium,
                    amount_per_cycle: 100,
                },
            ))
            .id();

        // Act - Player sells the claim
        app.world_mut().send_event(SellClaimEvent {
            asteroid_entity: asteroid,
            buyer_entity: external_faction,
            royalty_percentage: 0.10, // 10%
        });

        app.update();

        // Assert - Asteroid now belongs to the faction
        let claim = app
            .world()
            .get::<AsteroidClaim>(asteroid)
            .expect("Asteroid should have a claim");
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
        app.add_systems(Update, process_faction_behavior_system);
        app.add_event::<AttackColonyEvent>();

        let hostile_faction = app
            .world_mut()
            .spawn((
                FactionClaim {
                    name: "Terror Front".to_string(),
                    hostility_level: 100,
                },
                FactionInventory { uranium_stored: 90 }, // 10 went to royalties
            ))
            .id();

        // Act - Faction processes its inventory
        app.update();

        // Assert - Faction has enough uranium to trigger an attack event
        let events = app.world().resource::<Events<AttackColonyEvent>>();
        let mut cursor = events.get_cursor();
        assert!(cursor.read(events).any(|e| e.attacker == hostile_faction));
    }
}
