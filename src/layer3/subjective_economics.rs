use crate::layer1::resources::ResourceType;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub enum SpeciesType {
    Lithoid,
    Human,
    Robot,
}

#[derive(Component, Debug, Clone)]
pub struct MarketValue {
    pub resource: ResourceType,
    pub price: i32,
}

#[derive(Event, Debug, Clone)]
pub struct TradeEvent {
    pub buyer: Entity,
    pub seller: Entity,
    pub resource: ResourceType,
    pub quantity: i32,
}

#[derive(Resource, Debug, Default)]
pub struct Credits {
    pub amount: i32,
}

pub fn evaluate_market_value_system(mut query: Query<(&SpeciesType, &mut MarketValue)>) {
    for (species, mut market) in query.iter_mut() {
        match species {
            SpeciesType::Lithoid => {
                if market.resource == ResourceType::Stone {
                    market.price = 50; // Premium
                } else if market.resource == ResourceType::Food {
                    market.price = 1; // Waste
                }
            }
            SpeciesType::Human => {
                if market.resource == ResourceType::Stone {
                    market.price = 5; // Base
                } else if market.resource == ResourceType::Food {
                    market.price = 10; // Base
                }
            }
            SpeciesType::Robot => {
                if market.resource == ResourceType::Metal {
                    market.price = 1; // They generate so much it's waste
                }
            }
        }
    }
}

pub fn process_trade_system(
    mut events: EventReader<TradeEvent>,
    mut credits: ResMut<Credits>,
    market_query: Query<&MarketValue>,
) {
    for event in events.read() {
        if let Ok(market) = market_query.get(event.seller) {
            if market.resource == event.resource {
                let total_cost = market.price * event.quantity;
                if credits.amount >= total_cost {
                    credits.amount -= total_cost;
                    // In a full implementation, you'd add the resource to the player's inventory here
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::diplomacy::succession::Faction;

    #[test]
    fn test_resource_value_depends_on_faction_species() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_market_value_system);

        let lithoid_faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Rock Eaters".to_string(),
                },
                SpeciesType::Lithoid,
                MarketValue {
                    resource: ResourceType::Stone,
                    price: 0,
                },
            ))
            .id();

        let human_faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Humans".to_string(),
                },
                SpeciesType::Human,
                MarketValue {
                    resource: ResourceType::Stone,
                    price: 0,
                },
            ))
            .id();

        app.update();

        let lithoid_price = app
            .world()
            .get::<MarketValue>(lithoid_faction)
            .unwrap()
            .price;
        let human_price = app.world().get::<MarketValue>(human_faction).unwrap().price;

        assert!(
            lithoid_price > human_price,
            "Lithoids should value Granite much higher than Humans."
        );
    }

    #[test]
    fn test_trading_trash_for_treasure() {
        let mut app = App::new();
        app.insert_resource(Credits { amount: 100 });
        app.add_event::<TradeEvent>();
        app.add_systems(Update, process_trade_system);

        // Setup Robot Faction selling "Scrap" (Alloys) cheaply
        let robot_faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "Robots".to_string(),
                },
                SpeciesType::Robot,
                MarketValue {
                    resource: ResourceType::Metal,
                    price: 1,
                }, // Dirt cheap for them
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<TradeEvent>>()
            .send(TradeEvent {
                buyer: Entity::PLACEHOLDER, // Player
                seller: robot_faction,
                resource: ResourceType::Metal,
                quantity: 10,
            });

        app.update();

        let credits = app.world().resource::<Credits>();
        assert_eq!(
            credits.amount, 90,
            "Player should only pay the subjective faction value for the resource."
        );
    }
}
