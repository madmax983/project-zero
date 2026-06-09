use crate::layer1::resources::ResourceType;
use crate::layer1::social::factions::FactionId;
use bevy::prelude::*;

#[derive(Component)]
pub struct TradeEmbargo {
    pub resource: ResourceType,
    pub enforcing_faction: FactionId,
}

#[derive(Component, Resource)]
pub struct Influence {
    pub amount: u32,
    pub faction: FactionId,
}

#[derive(Event)]
pub struct DeclareEmbargoEvent {
    pub faction: FactionId,
    pub resource: ResourceType,
}

pub fn apply_trade_embargo_system(
    mut market: ResMut<crate::layer3::market::GalacticMarket>,
    embargoes: Query<&TradeEmbargo>,
) {
    // Clear previous embargoes first
    market.clear_embargoes();

    for embargo in embargoes.iter() {
        market.set_embargoed(embargo.resource, true);
    }
}

pub fn declare_trade_embargo_system(
    mut commands: Commands,
    mut influence: Option<ResMut<Influence>>,
    mut events: EventReader<DeclareEmbargoEvent>,
) {
    let cost = 500;

    for event in events.read() {
        if let Some(ref mut inf) = influence {
            if inf.faction == event.faction && inf.amount >= cost {
                inf.amount -= cost;

                commands.spawn(TradeEmbargo {
                    resource: event.resource,
                    enforcing_faction: event.faction,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::market::GalacticMarket;

    #[test]
    fn test_trade_embargo_prevents_trading() {
        let mut app = App::new();
        app.add_systems(Update, apply_trade_embargo_system);

        let faction_id = FactionId::FarmersGuild;
        let mut market = GalacticMarket::default();
        market.supply_pool.insert(ResourceType::Metal, 100.0);
        app.insert_resource(market);

        // Spawn an embargo entity
        app.world_mut().spawn(TradeEmbargo {
            resource: ResourceType::Metal,
            enforcing_faction: faction_id,
        });

        // Act
        app.update();

        // Assert
        let market = app.world().resource::<GalacticMarket>();
        assert!(market.is_embargoed(ResourceType::Metal));
        assert!(!market.can_trade(ResourceType::Metal, FactionId::MinersGuild));
    }

    #[test]
    fn test_trade_embargo_costs_influence() {
        let mut app = App::new();
        app.add_systems(Update, declare_trade_embargo_system);

        let faction_id = FactionId::FarmersGuild;
        app.insert_resource(Influence {
            amount: 500,
            faction: faction_id,
        });

        // Send event to declare embargo
        app.add_event::<DeclareEmbargoEvent>();
        app.world_mut().send_event(DeclareEmbargoEvent {
            faction: faction_id,
            resource: ResourceType::Metal,
        });

        // Act
        app.update();

        // Assert
        let influence = app.world().resource::<Influence>();
        assert_eq!(influence.amount, 0); // Assuming it costs 500

        // Embargo should be created
        let mut query = app.world_mut().query::<&TradeEmbargo>();
        let embargo_count = query.iter(app.world()).count();
        assert_eq!(embargo_count, 1);
    }
}
