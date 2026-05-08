use bevy::prelude::*;
use scale::layer1::culture::nostalgia::{
    nostalgia_spread_system, nostalgia_trigger_system, Nostalgia, RumorSpreadEvent,
};
use scale::layer1::lifecycle::Age;
use scale::layer1::pop::Pop;
use scale::layer1::rumor::{exchange_rumors_system, Knowledge};
use scale::layer1::social::morale::Morale;
use scale::layer1::social::Tavern;
use scale::layer1::core::integration::nostalgia_rumor_generation_bridge;

#[test]
fn test_nostalgia_plague_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<Events<RumorSpreadEvent>>();
    app.init_resource::<Events<scale::layer1::social::gossip_economy::GossipEvent>>();
    app.init_resource::<Events<scale::layer1::social::AffinityChange>>();
    app.insert_resource(scale::shared::time::SimulationTime {
        tick: 1,
        ..Default::default()
    });

    app.add_systems(
        Update,
        (
            nostalgia_trigger_system,
            nostalgia_rumor_generation_bridge,
            exchange_rumors_system,
            nostalgia_spread_system,
        )
            .chain(),
    );

    let old_pop = app
        .world_mut()
        .spawn((
            Pop,
            Age {
                ticks_alive: 65 * scale::layer1::balance::TICKS_PER_YEAR,
                stage: scale::layer1::lifecycle::LifeStage::Elder,
            },
            Morale {
                value: 10.0,
                modifiers: vec![],
            },
            Knowledge::default(),
        ))
        .id();

    let young_pop = app
        .world_mut()
        .spawn((
            Pop,
            Age {
                ticks_alive: 20 * scale::layer1::balance::TICKS_PER_YEAR,
                stage: scale::layer1::lifecycle::LifeStage::Adult,
            },
            Knowledge::default(),
        ))
        .id();

    // Put them in a tavern together
    app.world_mut().spawn(Tavern {
        visitors: vec![old_pop, young_pop],
    });

    // Run enough updates to guarantee the 25% spread chance triggers
    for _ in 0..50 {
        app.update();
    }

    assert!(
        app.world().get::<Nostalgia>(old_pop).is_some(),
        "Old pop should have Nostalgia"
    );

    assert!(
        app.world().get::<Nostalgia>(young_pop).is_some(),
        "Young pop should have caught Nostalgia through rumors"
    );
}
