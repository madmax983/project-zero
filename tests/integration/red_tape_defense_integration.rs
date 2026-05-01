use bevy::prelude::*;
use scale::layer3::diplomacy::red_tape_defense::{
    invoke_red_tape, process_bureaucracy_delays, AdminResource, BureaucraticHold, HostileFleet,
};

#[test]
#[allow(clippy::type_complexity)]
fn test_red_tape_defense_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, process_bureaucracy_delays);

    // Initial time and resource setup
    app.insert_resource(Time::<()>::default());
    app.insert_resource(AdminResource { amount: 200 });

    let fleet = app
        .world_mut()
        .spawn(HostileFleet {
            invasion_timer: 10.0,
        })
        .id();

    // Init Time resource
    app.update();

    let mut system_state: bevy::ecs::system::SystemState<(
        Commands,
        Query<(Entity, Option<&mut BureaucraticHold>), With<HostileFleet>>,
        ResMut<AdminResource>,
    )> = bevy::ecs::system::SystemState::new(app.world_mut());

    let (mut commands, mut q_fleets, mut admin) = system_state.get_mut(app.world_mut());
    let (entity, hold_opt) = q_fleets.single_mut();

    let mut binding = hold_opt;
    invoke_red_tape(entity, &mut commands, &mut admin, binding.as_deref_mut());
    system_state.apply(app.world_mut());

    let hold_data = app.world().get::<BureaucraticHold>(fleet).unwrap();
    assert_eq!(hold_data.timer, 20.0);

    let initial_fleet_timer = app.world().get::<HostileFleet>(fleet).unwrap().invasion_timer;

    // Simulate manually since we can't fast forward easily here with Time Virtual
    // We will just directly test the logic:
    // Update should tick timer down by ~time delta
    // And when the timer is gone, invasion_timer drops.
    let mut hold = app.world_mut().get_mut::<BureaucraticHold>(fleet).unwrap();
    hold.timer = 0.001; // nearly expired

    app.update(); // Tick - hold removed, invasion timer holds

    assert!(app.world().get::<BureaucraticHold>(fleet).is_none());

    // Now invasion timer should resume ticking down
    let current_fleet_timer = app.world().get::<HostileFleet>(fleet).unwrap().invasion_timer;
    assert_eq!(current_fleet_timer, initial_fleet_timer); // It shouldn't have changed yet

    app.update(); // tick without hold

    let fleet_data = app.world().get::<HostileFleet>(fleet).unwrap();
    assert!(fleet_data.invasion_timer < current_fleet_timer, "fleet timer: {}", fleet_data.invasion_timer);
}
