use bevy_ecs::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::events::BuildingRemovedEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::parasitic_architecture::{
    process_megastructure_consumption, ParasiticArchitecture,
};
use scale::layer1::structure::Structure;

#[test]
fn test_parasitic_architecture_emits_events() {
    let mut world = World::new();

    // Initialize event resources
    world.init_resource::<Events<BuildingRemovedEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    // Spawn a parasitic megastructure
    let _megastructure = world
        .spawn((
            GridPosition { x: 10, y: 10 },
            Building {
                building_type: BuildingType::Lander, // Or some megastructure type
            },
            ParasiticArchitecture {
                radius: 2.0,
                consumption_rate: 100.0, // High rate to ensure immediate destruction
            },
        ))
        .id();

    // Spawn a victim building within radius
    let victim = world
        .spawn((
            GridPosition { x: 11, y: 10 },
            Building {
                building_type: BuildingType::Housing,
            },
            Structure {
                current_hp: 50.0,
                max_hp: 100.0,
            },
        ))
        .id();

    // Run the system
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(process_megastructure_consumption);
    schedule.run(&mut world);

    // Verify building is destroyed
    assert!(world.get::<Structure>(victim).is_none());

    // Verify BuildingRemovedEvent is emitted
    let removed_events = world.resource::<Events<BuildingRemovedEvent>>();
    #[allow(deprecated)]
    let mut reader = removed_events.get_reader();
    let emitted_removed: Vec<_> = reader.read(removed_events).collect();
    assert_eq!(
        emitted_removed.len(),
        1,
        "Should emit exactly 1 BuildingRemovedEvent"
    );
    assert_eq!(emitted_removed[0].entity, victim);
    assert_eq!(emitted_removed[0].position, GridPosition { x: 11, y: 10 });
    assert_eq!(emitted_removed[0].building_type, BuildingType::Housing);

    // Verify AddChronicleEvent is emitted
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut chron_reader = chronicle_events.get_reader();
    let emitted_chron: Vec<_> = chron_reader.read(chronicle_events).collect();
    assert_eq!(
        emitted_chron.len(),
        1,
        "Should emit exactly 1 AddChronicleEvent"
    );
    assert_eq!(
        emitted_chron[0].importance,
        EventImportance::Standard
    );
    assert!(emitted_chron[0].text.contains("Housing"));
    assert!(emitted_chron[0].text.contains("consumed"));
}
