use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::energy::PowerSource;
use scale::layer1::lighting::LightSource;
use scale::layer1::pop::PopBundle;
use scale::layer1::tech::martyrs_engine::{
    attune_engine_system, process_martyrs_engine, AttuneEngineAction, MartyrsEngine,
};
use scale::layer2::shielding::OrbitalShield;

#[test]
fn test_martyrs_engine_chronicle_integration() {
    let mut world = World::new();
    world.insert_resource(Events::<AddChronicleEvent>::default());

    let engine = world
        .spawn((
            MartyrsEngine { ticks_remaining: 0 },
            PowerSource {
                output: 0.0,
                ..Default::default()
            },
            OrbitalShield {
                capacity: 0.0,
                ..Default::default()
            },
        ))
        .id();

    let mut rng = rand::thread_rng();
    let pop = world.spawn(PopBundle::random(0, 0, &mut rng)).id();

    world.spawn(AttuneEngineAction {
        target_engine: engine,
        pop,
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(attune_engine_system);
    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut has_event = false;
    for event in reader.read(events) {
        if event.importance == EventImportance::Legendary && event.text.contains("Martyr's Engine")
        {
            has_event = true;
        }
    }
    assert!(has_event, "Should emit a Legendary AddChronicleEvent");
}

#[test]
fn test_martyrs_engine_vfx_integration() {
    let mut world = World::new();

    let engine = world
        .spawn((
            MartyrsEngine {
                ticks_remaining: 10,
            },
            PowerSource {
                output: 0.0,
                ..Default::default()
            },
            OrbitalShield {
                capacity: 0.0,
                ..Default::default()
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(process_martyrs_engine);

    // Run once: should add light source
    schedule.run(&mut world);

    assert!(
        world.get::<LightSource>(engine).is_some(),
        "Should add a LightSource when engine is active"
    );

    let light = world.get::<LightSource>(engine).unwrap();
    assert_eq!(light.color, (255, 50, 50), "Should be an eerie red glow");

    // Manually run it down
    world.get_mut::<MartyrsEngine>(engine).unwrap().ticks_remaining = 0;
    schedule.run(&mut world);

    assert!(
        world.get::<LightSource>(engine).is_none(),
        "Should remove LightSource when engine stops"
    );
}
