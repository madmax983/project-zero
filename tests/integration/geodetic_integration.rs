use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::economy::items::{Item, ItemType};
use scale::layer1::geodetic::{form_golem_system, GolemFormedEvent, LivingStone};
use scale::layer1::integration::golem_formed_chronicle_bridge_system;
use scale::layer1::map::GridPosition;

#[test]
fn test_geodetic_sentience_chronicle_integration() {
    let mut app = bevy_app::App::new();

    // Add required events
    app.add_event::<GolemFormedEvent>();
    app.add_event::<AddChronicleEvent>();

    // Add systems
    app.add_systems(
        bevy_app::Update,
        (
            form_golem_system,
            golem_formed_chronicle_bridge_system.after(form_golem_system),
        ),
    );

    // Spawn 5 Living Stones to trigger Golem formation
    for _ in 0..5 {
        app.world_mut().spawn((
            Item {
                item_type: ItemType::LivingStone,
                ..Default::default()
            },
            LivingStone { last_move_tick: 0 },
            GridPosition { x: 5, y: 5 },
        ));
    }

    // Run the simulation
    app.update();

    // Verify that the GolemFormedEvent was emitted
    let golem_events = app.world().resource::<Events<GolemFormedEvent>>();
    let mut reader = golem_events.get_cursor();
    assert_eq!(
        reader.read(golem_events).len(),
        1,
        "A GolemFormedEvent should be emitted when a Golem is formed."
    );

    // Verify that the AddChronicleEvent was emitted and bridges correctly
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut chron_reader = chronicle_events.get_cursor();
    let events: Vec<_> = chron_reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "An AddChronicleEvent should be emitted by the bridge system."
    );

    assert_eq!(
        events[0].importance,
        EventImportance::Major,
        "The chronicle event should have Major importance."
    );

    assert!(
        events[0].text.contains("Awake"),
        "The chronicle event should have the correct text."
    );
}
