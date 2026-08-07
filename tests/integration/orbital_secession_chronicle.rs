use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::orbit::secession::{OrbitalHabitat, SecessionState, Unrest};

#[test]
fn test_orbital_secession_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<AddChronicleEvent>>();

    // Create an entity with SecessionState::Loyal
    let entity = world
        .spawn((
            OrbitalHabitat {
                population: 5000,
                wealth: 100000.0,
            },
            Unrest { level: 90.0 },
            SecessionState::Loyal,
        ))
        .id();

    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(scale::layer2::integration::orbital_secession_chronicle_bridge);

    // Run the bridge system - shouldn't trigger because it's Loyal
    world.clear_trackers();
    schedule.run(&mut world);

    {
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();
        assert_eq!(emitted.len(), 0);
    }

    // Mutate to Seceded
    world.entity_mut(entity).insert(SecessionState::Seceded);

    schedule.run(&mut world);

    // Verify the chronicle event was emitted
    {
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].importance, EventImportance::Major);
        assert!(emitted[0]
            .text
            .contains("An Orbital Habitat has declared independence"));
    }
}
