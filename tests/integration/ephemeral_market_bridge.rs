use bevy::prelude::*;
use scale::layer3::market::ephemeral_market::{
    fulfill_market_trade_system, process_market_despawn_system, spawn_ephemeral_market_system,
    EphemeralMarket, MarketSpawnEvent, MarketTradeEvent, MarketTradeFailedEvent,
};
use scale::layer1::chronicle::Chronicle;
use scale::prelude::SimulationTime;
use scale::layer1::resources::ColonyResources;

#[test]
fn test_ephemeral_market_systems_registered_in_schedule() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.init_resource::<bevy_ecs::event::Events<MarketSpawnEvent>>();
    app.init_resource::<bevy_ecs::event::Events<MarketTradeEvent>>();
    app.init_resource::<bevy_ecs::event::Events<MarketTradeFailedEvent>>();
    app.init_resource::<Chronicle>();
    app.init_resource::<SimulationTime>();
    app.init_resource::<ColonyResources>();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        spawn_ephemeral_market_system,
        process_market_despawn_system,
        fulfill_market_trade_system,
    ));

    let system_entity = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(MarketSpawnEvent {
        system_entity,
        duration_ticks: 10,
    });

    schedule.run(app.world_mut());

    let market_count = app
        .world_mut()
        .query::<&EphemeralMarket>()
        .iter(app.world())
        .count();

    assert_eq!(market_count, 1, "Market should be spawned");
}
