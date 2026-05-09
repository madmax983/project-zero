use bevy::prelude::*;
use scale::layer1::void_weed::PirateRaidEvent;
use scale::layer3::pirates::{resource_curse_raid_bridge, PirateThreatLevel};

#[test]
fn test_resource_curse_triggers_raid() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<PirateThreatLevel>();
    app.add_event::<PirateRaidEvent>();

    let mut schedule = Schedule::default();
    schedule.add_systems(resource_curse_raid_bridge);

    app.world_mut().resource_mut::<PirateThreatLevel>().level = 15.0;

    schedule.run(app.world_mut());

    let threat = app.world().resource::<PirateThreatLevel>().level;
    assert_eq!(threat, 5.0, "Threat level should decrease by 10.0");

    let events = app.world().resource::<Events<PirateRaidEvent>>();
    let mut cursor = events.get_cursor();
    let emitted_events: Vec<_> = cursor.read(events).collect();

    assert_eq!(
        emitted_events.len(),
        1,
        "A single PirateRaidEvent should have been emitted"
    );
}
