use bevy::prelude::*;
use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer2::piracy::Faction;

#[derive(Component)]
pub struct Asteroid;

#[derive(Component, Clone)]
pub struct ResourceYield {
    pub resource_type: ResourceType,
    pub amount_per_cycle: u32,
}

#[derive(Component)]
pub struct FactionInventory {
    pub ore_stored: u32,
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
    mut stockpile: ResMut<ColonyResources>,
) {
    for _ in events.read() {
        for (claim, yield_data) in query.iter() {
            let royalty_amount = (yield_data.amount_per_cycle as f32 * claim.royalty_percentage).round();
            stockpile.add_resource(&yield_data.resource_type, royalty_amount);
        }
    }
}

pub fn process_faction_behavior_system(
    mut factions: Query<(Entity, &Faction, &mut FactionInventory)>,
    mut attack_events: EventWriter<AttackColonyEvent>,
) {
    for (entity, faction, mut inventory) in factions.iter_mut() {
        if faction.relationship_score < -50 && inventory.ore_stored >= 50 {
            // Faction builds a weapon and attacks
            inventory.ore_stored -= 50;
            attack_events.send(AttackColonyEvent { attacker: entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_selling_asteroid_claim_assigns_faction_and_generates_royalties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, (sell_asteroid_claim_system, process_royalties_system));
        app.init_resource::<ColonyResources>();
        app.add_event::<SellClaimEvent>();
        app.add_event::<ProductionCycleEvent>();

        let external_faction = app.world_mut().spawn(Faction {
            name: "Independent Belters".to_string(),
            relationship_score: 0,
        }).id();

        let asteroid = app.world_mut().spawn((
            Asteroid,
            ResourceYield { resource_type: ResourceType::Ore, amount_per_cycle: 100 },
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

        // Assert - Colony receives 10% of 100 Ore (10)
        let stockpile = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(stockpile.get_amount(ResourceType::Ore), 10.0);
    }

    #[test]
    fn test_hostile_faction_uses_asteroid_resources_for_attack() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_faction_behavior_system);
        app.add_event::<AttackColonyEvent>();

        let hostile_faction = app.world_mut().spawn((
            Faction { name: "Terror Front".to_string(), relationship_score: -100 },
            FactionInventory { ore_stored: 90 }, // 10 went to royalties
        )).id();

        // Act - Faction processes its inventory
        app.update();

        // Assert - Faction has enough ore to trigger an attack event
        let events = app.world().resource::<Events<AttackColonyEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).any(|e| e.attacker == hostile_faction));
    }
}
