use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::alien_broadcast_bridge_system;
use scale::layer1::memetics::MemeticInfection;
use scale::layer1::notifications::{NotificationQueue, NotificationSeverity};
use scale::layer1::pop::Pop;
use scale::layer3::events::alien_broadcast::AlienBroadcastEvent;

#[test]
fn test_alien_broadcast_infects_pops() {
    let mut world = World::new();

    // Register resources and events
    world.init_resource::<Events<AlienBroadcastEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();
    world.init_resource::<NotificationQueue>();

    // Spawn 10 Pops
    for _ in 0..10 {
        world.spawn(Pop);
    }

    // Trigger AlienBroadcastEvent
    world
        .resource_mut::<Events<AlienBroadcastEvent>>()
        .send(AlienBroadcastEvent);

    // Run system
    let mut schedule = Schedule::default();
    schedule.add_systems(alien_broadcast_bridge_system);
    schedule.run(&mut world);

    // Assert that at least some pops are infected
    let mut infected_count = 0;
    for (_, infection) in world
        .query::<(&Pop, Option<&MemeticInfection>)>()
        .iter(&world)
    {
        if let Some(MemeticInfection::ParasiticBroadcast) = infection {
            infected_count += 1;
        }
    }

    assert!(
        infected_count > 0,
        "At least one Pop should be infected with ParasiticBroadcast"
    );

    // Assert Notification was added
    let notifications = world.resource::<NotificationQueue>();
    assert!(
        notifications.active.iter().any(|n| n.severity == NotificationSeverity::Error
            && n.text.contains("infectious alien broadcast")),
        "Error notification should be generated"
    );

    // Assert Chronicle event was emitted
    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();
    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("comms array picked up a strange"));
}
