use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::pop::Pop;
use scale::layer2::integration::refugee_arrival_bridge_system;
use scale::layer2::refugees::fleet_arrival::{RefugeeArrivalEvent, SubLightRefugeePlugin};

#[test]
fn test_refugee_arrival_bridge_spawns_pops_and_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add events manually because we only added minimal plugins,
    // though SubLightRefugeePlugin will add RefugeeArrivalEvent
    app.add_plugins(SubLightRefugeePlugin);
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, refugee_arrival_bridge_system);

    let fleet = app.world_mut().spawn_empty().id();

    // Act: Fire event
    app.world_mut().send_event(RefugeeArrivalEvent {
        fleet_entity: fleet,
    });

    app.update();

    // Assert: Pops should be spawned
    let mut pop_count = 0;
    for _ in app.world_mut().query::<&Pop>().iter(app.world()) {
        pop_count += 1;
    }
    assert!(
        pop_count > 0,
        "Refugee arrival should spawn at least one Pop."
    );

    // Assert: Chronicle event should be added
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    assert!(
        reader.read(events).next().is_some(),
        "Refugee arrival should record a Chronicle event."
    );
}
