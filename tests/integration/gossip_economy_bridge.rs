use bevy::prelude::*;
use scale::layer1::pop::Pop;
use scale::layer1::social::gossip_economy::{
    process_gossip, spend_intel, BrokerItem, BrokerPurchaseEvent, GossipEvent, Gossiping,
    IntelTokens,
};
use scale::layer1::social::morale::Morale;
use scale::layer1::social::rumor::RumorTopic;

#[test]
fn test_gossip_economy_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<GossipEvent>();
    app.add_event::<BrokerPurchaseEvent>();
    app.insert_resource(IntelTokens(0));

    // Register the systems
    app.add_systems(Update, (process_gossip, spend_intel));

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 50.0,
                modifiers: vec![],
            },
        ))
        .id();

    // Fire gossip event
    app.world_mut()
        .resource_mut::<Events<GossipEvent>>()
        .send(GossipEvent {
            pop,
            rumor: RumorTopic::DoomProphecy, // Dark Secret gives 2 intel
        });

    app.update();

    // Verify intel generated and gossiping component added
    assert_eq!(app.world().resource::<IntelTokens>().0, 2);
    assert!(app.world().get::<Gossiping>(pop).is_some());
    assert!(app.world().get::<Morale>(pop).unwrap().value < 50.0); // DoomProphecy lowers morale

    // Fire broker purchase event
    app.world_mut()
        .resource_mut::<Events<BrokerPurchaseEvent>>()
        .send(BrokerPurchaseEvent {
            item: BrokerItem::RareTech,
            cost: 2,
        });

    app.update();

    // Verify intel spent
    assert_eq!(app.world().resource::<IntelTokens>().0, 0);
}
