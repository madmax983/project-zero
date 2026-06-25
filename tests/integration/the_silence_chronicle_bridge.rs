use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer3::integration::the_silence_chronicle_bridge;
use scale::layer3::silence::HostileSpawnEvent;

#[test]
fn test_hostile_spawn_triggers_chronicle() {
    let mut app = App::new();
    app.init_resource::<Events<HostileSpawnEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, the_silence_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<HostileSpawnEvent>>()
        .send(HostileSpawnEvent { severity: 1 });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(
        chronicle_events.len(),
        1,
        "Should emit one AddChronicleEvent"
    );
}
