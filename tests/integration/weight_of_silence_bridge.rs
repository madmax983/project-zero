use bevy::prelude::*;
use scale::experimental::the_weight_of_silence::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer2::trade::routes::TradeRouteExecutedEvent;
use scale::layer3::integration::{reset_isolation_on_trade_system, silence_cult_chronicle_bridge};

#[test]
fn test_trade_route_resets_colony_isolation() {
    let mut app = App::new();
    app.add_event::<TradeRouteExecutedEvent>();
    app.insert_resource(scale::shared::time::SimulationTime {
        tick: 1000,
        ..Default::default()
    });

    app.add_systems(Update, reset_isolation_on_trade_system);

    let colony_ent = app
        .world_mut()
        .spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 500.0,
        })
        .id();

    app.world_mut().send_event(TradeRouteExecutedEvent {
        source: Entity::PLACEHOLDER,
        destination: colony_ent,
        item_type: "Food".to_string(),
        amount: 10,
    });

    app.update();

    let node = app.world().get::<ColonyNode>(colony_ent).unwrap();
    assert_eq!(node.last_communication_tick, 1000);
    assert_eq!(node.isolation_level, 0.0);
}

#[test]
fn test_silence_cult_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, silence_cult_chronicle_bridge);

    app.world_mut().spawn(SilenceCult {
        colony_entity: Entity::PLACEHOLDER,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.get_cursor().len(events), 1);
}
