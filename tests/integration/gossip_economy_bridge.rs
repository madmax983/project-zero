use bevy::prelude::*;
use scale::layer1::social::gossip_economy::{BrokerItem, BrokerPurchaseEvent, IntelTokens};
use scale::layer1::resources::ColonyResources;
use scale::layer1::social::factions::{Factions, FactionState, FactionId, FactionData};
use scale::layer1::core::integration::gossip_broker_purchase_bridge;

#[test]
fn test_broker_rare_tech() {
    let mut app = App::new();
    app.add_event::<BrokerPurchaseEvent>();
    app.insert_resource(IntelTokens(10));
    app.insert_resource(ColonyResources::default());
    app.insert_resource(Factions::default());
    app.add_systems(Update, gossip_broker_purchase_bridge);

    app.world_mut().resource_mut::<Events<BrokerPurchaseEvent>>().send(BrokerPurchaseEvent {
        item: BrokerItem::RareTech,
        cost: 5,
    });

    app.update();

    let res = app.world().resource::<ColonyResources>();
    assert_eq!(res.knowledge, 100.0);
    assert_eq!(res.credits, 10.0);
    assert_eq!(app.world().resource::<IntelTokens>().0, 5);
}

#[test]
fn test_broker_relations_boost() {
    let mut app = App::new();
    app.add_event::<BrokerPurchaseEvent>();
    app.insert_resource(IntelTokens(10));
    app.insert_resource(ColonyResources::default());

    let mut factions = Factions::default();
    factions.map.insert(FactionId::MinersGuild, FactionData {
        name: "MinersGuild".to_string(),
        state: FactionState::Unhappy,
        satisfaction: 0.1,
        members_count: 5,
        active_demand: None,
    });
    app.insert_resource(factions);

    app.add_systems(Update, gossip_broker_purchase_bridge);

    app.world_mut().resource_mut::<Events<BrokerPurchaseEvent>>().send(BrokerPurchaseEvent {
        item: BrokerItem::RelationsBoost,
        cost: 5,
    });

    app.update();

    let factions = app.world().resource::<Factions>();
    let data = factions.map.get(&FactionId::MinersGuild).unwrap();
    assert_eq!(data.state, FactionState::Loyal);
    assert_eq!(data.satisfaction, 1.0);
    assert_eq!(app.world().resource::<IntelTokens>().0, 5);
}
