use bevy::prelude::*;
use scale::layer3::diplomacy::red_tape_defense::{process_bureaucracy_delays, HostileFleet, BureaucraticHold};

#[test]
fn test_process_bureaucracy_delays_integration() {
    let mut app = App::new();
    // Use an empty app to control time fully
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, process_bureaucracy_delays);

    let fleet = app.world_mut().spawn((HostileFleet { invasion_timer: 10.0 }, BureaucraticHold { timer: 0.05, cost_multiplier: 2 })).id();

    // Simulate some time
    let mut time = app.world_mut().resource_mut::<Time<()> >();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));

    app.update();

    // The hold should be removed
    assert!(app.world().get::<BureaucraticHold>(fleet).is_none());
}
