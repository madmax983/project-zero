use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::economy::debt_of_the_dead::{
    DebtInheritedEvent, DebtSocializedEvent, SocializedDebt,
};
use scale::layer1::economy::Wallet;
use scale::layer1::entities::pop::{Pop, PopDied};
use scale::layer1::social::morale::Morale;
use scale::layer1::social::Relationships;

#[test]
fn test_debt_socialized_chronicle() {
    let mut app = bevy_app::App::new();
    app.add_event::<PopDied>();
    app.add_event::<DebtInheritedEvent>();
    app.add_event::<DebtSocializedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(SocializedDebt::default());

    app.add_systems(
        bevy_app::Update,
        (
            scale::layer1::economy::debt_of_the_dead::process_debt_of_the_dead_system,
            scale::layer1::core::integration::debt_of_the_dead_chronicle_bridge,
        )
            .chain(),
    );

    let dead_pop = app
        .world_mut()
        .spawn((
            Pop,
            Wallet { credits: -100.0 },
            Relationships {
                affinities: std::collections::HashMap::new(),
            },
        ))
        .id();

    app.world_mut().send_event(PopDied {
        entity: dead_pop,
        name: "Loner Debtor".to_string(),
        tick: 1,
        reason: "Starvation".to_string(),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let chronicle_events = reader.read(events).collect::<Vec<_>>();

    assert_eq!(chronicle_events.len(), 1);
    assert!(chronicle_events[0]
        .text
        .contains("Loner Debtor died, leaving 100 credits of debt"));
}

#[test]
fn test_debt_inherited_chronicle() {
    let mut app = bevy_app::App::new();
    app.add_event::<PopDied>();
    app.add_event::<DebtInheritedEvent>();
    app.add_event::<DebtSocializedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(SocializedDebt::default());

    app.add_systems(
        bevy_app::Update,
        (
            scale::layer1::economy::debt_of_the_dead::process_debt_of_the_dead_system,
            scale::layer1::core::integration::debt_of_the_dead_chronicle_bridge,
        )
            .chain(),
    );

    let relative = app
        .world_mut()
        .spawn((Pop, Wallet { credits: 10.0 }, Morale::default()))
        .id();

    let dead_pop = app
        .world_mut()
        .spawn((
            Pop,
            Wallet { credits: -50.0 },
            Relationships {
                affinities: vec![(relative, 50.0)].into_iter().collect(),
            },
        ))
        .id();

    app.world_mut().send_event(PopDied {
        entity: dead_pop,
        name: "Debtor".to_string(),
        tick: 1,
        reason: "Old age".to_string(),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let chronicle_events = reader.read(events).collect::<Vec<_>>();

    assert_eq!(chronicle_events.len(), 1);
    assert!(chronicle_events[0]
        .text
        .contains("Debtor died. 50 credits of debt inherited"));
}
