use bevy::prelude::*;
use scale::layer1::entities::pop::Pop;
use scale::layer1::psychology::needs::Needs;
use scale::layer1::psychology::teleport_psychosis::{
    handle_teleport_system, hunger_decay_system, process_psychosis_system, Dissociation,
    TeleportEvent,
};
use scale::layer1::psychology::traits::{Trait, Traits};

#[test]
fn test_teleporter_use_adds_dissociation() {
    let mut app = App::new();
    app.add_systems(Update, handle_teleport_system);
    app.add_event::<TeleportEvent>();

    let pop_id = app
        .world_mut()
        .spawn((Pop, Dissociation { level: 0.0 }))
        .id();

    app.world_mut().send_event(TeleportEvent { entity: pop_id });
    app.update();

    let dissoc = app.world().get::<Dissociation>(pop_id).unwrap();
    assert!(dissoc.level > 0.0);
}

#[test]
fn test_high_dissociation_grants_phantom_trait() {
    let mut app = App::new();
    app.add_systems(Update, process_psychosis_system);

    let pop_id = app
        .world_mut()
        .spawn((Pop, Traits::default(), Dissociation { level: 90.0 }))
        .id();

    app.update();

    let traits = app.world().get::<Traits>(pop_id).unwrap();
    assert!(traits.has(Trait::Phantom));
}

#[test]
fn test_phantom_trait_ignores_hunger() {
    let mut app = App::new();
    app.add_systems(Update, hunger_decay_system);

    let normal_pop = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                hunger: 100.0,
                ..default()
            },
            Traits::default(),
        ))
        .id();

    let mut ghost_traits = Traits::default();
    ghost_traits.add(Trait::Phantom);

    let ghost_pop = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                hunger: 100.0,
                ..default()
            },
            ghost_traits,
        ))
        .id();

    app.update();

    assert!(app.world().get::<Needs>(normal_pop).unwrap().hunger < 100.0);
    assert_eq!(app.world().get::<Needs>(ghost_pop).unwrap().hunger, 100.0);
}

#[test]
fn test_max_dissociation_despawns_pop() {
    let mut app = App::new();
    app.add_systems(Update, process_psychosis_system);

    let pop_id = app
        .world_mut()
        .spawn((Pop, Traits::default(), Dissociation { level: 100.0 }))
        .id();

    app.update();

    assert!(app.world().get::<Pop>(pop_id).is_none());
}
