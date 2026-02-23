use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::inventory::Inventory;
use crate::layer1::trade::{MarketStatus, TradeMarket};
use bevy_ecs::prelude::*;

/// Component indicating a Pop is in possession of Contraband.
#[derive(Component)]
pub struct ContrabandPossession;

/// System to synchronize Edicts with Market Status.
///
/// Ensures items prohibited by `ColonyPolicies` are marked as `Contraband` in `TradeMarket`.
pub fn enforce_prohibition_system(edicts: Res<ColonyPolicies>, mut market: ResMut<TradeMarket>) {
    // 1. Reset all to Legal
    for item in market.items.values_mut() {
        item.status = MarketStatus::Legal;
    }

    // 2. Apply prohibitions
    for policy in &edicts.active_policies {
        if let Policy::Prohibition(resource) = policy {
            if let Some(item) = market.items.get_mut(resource) {
                item.status = MarketStatus::Contraband;
            } else {
                // If item not in market, add it? Or assume market covers all tradeable resources.
                // TradeMarket::default() should cover all if initialized properly,
                // or we insert it now.
                market.items.insert(
                    *resource,
                    crate::layer1::trade::MarketItem {
                        base_price: 10.0, // Default base price
                        status: MarketStatus::Contraband,
                    },
                );
            }
        }
    }
}

/// System to detect if Pops are carrying Contraband.
///
/// Checks inventory items against `TradeMarket` status. If a match is found,
/// the Pop is marked with `ContrabandPossession`.
pub fn detect_contraband_system(
    market: Res<TradeMarket>,
    mut commands: Commands,
    query: Query<(Entity, &Inventory), Without<ContrabandPossession>>,
) {
    for (entity, inventory) in query.iter() {
        let has_contraband = inventory.items.iter().any(|item| {
            if let Some(resource_type) = item.item_type.as_resource_type() {
                market
                    .items
                    .get(&resource_type)
                    .is_some_and(|market_item| market_item.status == MarketStatus::Contraband)
            } else {
                false
            }
        });

        if has_contraband {
            commands.entity(entity).insert(ContrabandPossession);
            // In a full implementation, we would emit a CrimeEvent here
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::ItemType;
    use crate::layer1::resources::ResourceType;
    use crate::layer1::trade::{MarketItem, MarketStatus, TradeMarket};

    #[test]
    fn test_banning_resource() {
        let mut world = World::new();
        // Setup Market
        let mut market = TradeMarket::default();
        market.items.insert(
            ResourceType::Alcohol,
            MarketItem {
                base_price: 10.0,
                status: MarketStatus::Legal,
            },
        );
        world.insert_resource(market);

        // Setup Edict Manager (ColonyPolicies)
        let mut edicts = ColonyPolicies::default();
        edicts
            .active_policies
            .insert(Policy::Prohibition(ResourceType::Alcohol));
        world.insert_resource(edicts);

        // Run system to sync edicts to market status
        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_prohibition_system);
        schedule.run(&mut world);

        let market = world.get_resource::<TradeMarket>().unwrap();
        let item = market.items.get(&ResourceType::Alcohol).unwrap();

        // This assertion will fail initially
        assert_eq!(item.status, MarketStatus::Contraband);
    }

    #[test]
    fn test_possession_crime_trigger() {
        let mut world = World::new();

        // Setup Pop with Alcohol
        let pop = world
            .spawn((
                crate::layer1::pop::Pop,
                Inventory {
                    items: vec![InventoryItem {
                        item_type: ItemType::Alcohol,
                    }],
                },
            ))
            .id();

        // Setup Market stating Alcohol is Contraband
        let mut market = TradeMarket::default();
        market.items.insert(
            ResourceType::Alcohol,
            MarketItem {
                base_price: 10.0,
                status: MarketStatus::Contraband,
            },
        );
        // We also need to map ItemType::Alcohol to ResourceType::Alcohol (which we implemented)
        // But the system needs to check the market.
        world.insert_resource(market);

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_contraband_system);
        schedule.run(&mut world);

        // This assertion will fail initially
        assert!(
            world.get::<ContrabandPossession>(pop).is_some(),
            "Pop holding contraband should have ContrabandPossession component"
        );
    }
}
