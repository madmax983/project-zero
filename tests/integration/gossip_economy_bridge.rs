use bevy::app::App;
use bevy::prelude::*;
use scale::layer1::social::gossip_economy::{
    intel_decay_system, process_gossip, spend_intel, Broker, BrokerItem, BrokerPurchaseEvent,
    GossipEvent, IntelTokens, MAX_INTEL_TOKENS,
};
use scale::layer1::social::morale::Morale;
use scale::layer1::social::rumor::RumorTopic;
use scale::shared::time::SimulationTime;
use scale::layer1::utility_types::{ActionType, PopAction};
use scale::layer1::social::rumor::{Knowledge, Rumor};
use scale::layer1::social::Tavern;
use scale::layer1::actions::AssignedTo;
use scale::layer1::utility_types::AssignmentType;
use scale::layer1::social::handle_socialize;
use scale::layer1::core::integration::gossip_economy_action_bridge;

#[test]
fn test_gossip_economy_end_to_end() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<GossipEvent>();
    app.add_event::<BrokerPurchaseEvent>();
    app.insert_resource(IntelTokens(0));
    app.insert_resource(SimulationTime::default());

    app.add_systems(
        Update,
        (gossip_economy_action_bridge, process_gossip, spend_intel, intel_decay_system).chain(),
    );

    // Spawn a pop with Morale and a Gossip action
    let pop = app
        .world_mut()
        .spawn((
            scale::layer1::pop::Pop,
            Morale {
                value: 50.0,
                modifiers: vec![],
            },
            PopAction {
                current: ActionType::Gossip,
                current_utility: 0.8,
                ticks_committed: 1,
            },
        ))
        .id();

    // Give the pop some knowledge
    let mut knowledge = Knowledge::default();
    knowledge.add_rumor(Rumor {
        topic: RumorTopic::DoomProphecy,
        source: pop,
        timestamp: 0,
        strength: 1.0,
    });
    app.world_mut().entity_mut(pop).insert(knowledge);

    app.update();

    // Pop should have acted on Gossip, lowering morale and producing IntelTokens
    assert_eq!(app.world().resource::<IntelTokens>().0, 2);
    assert!(app.world().get::<Morale>(pop).unwrap().value < 50.0);

    // Action should be reset
    assert_eq!(app.world().get::<PopAction>(pop).unwrap().current, ActionType::Idle);
}
