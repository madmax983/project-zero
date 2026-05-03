use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::diplomacy::red_tape_defense::{AdminResource, BureaucraticHold, HostileFleet, RedTapePlugin, invoke_red_tape};

#[test]
fn test_red_tape_chronicle_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(RedTapePlugin);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, scale::layer3::integration::red_tape_chronicle_bridge);

    let fleet = app.world_mut().spawn(HostileFleet { invasion_timer: 1.0 }).id();
    app.world_mut().insert_resource(AdminResource { amount: 100 });

    let mut system_state: bevy::ecs::system::SystemState<(Commands, Query<(Entity, Option<&mut BureaucraticHold>), With<HostileFleet>>, ResMut<AdminResource>)> = bevy::ecs::system::SystemState::new(app.world_mut());
    let (mut commands, mut q_fleets, mut admin) = system_state.get_mut(app.world_mut());

    let (entity, mut hold_opt) = q_fleets.single_mut();
    invoke_red_tape(entity, &mut commands, &mut admin, hold_opt.as_deref_mut());
    system_state.apply(app.world_mut());

    app.update(); // Trigger the bridge system

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.len(), 1);
    let mut cursor = events.get_cursor();
    let ev = cursor.read(events).next().unwrap();
    assert_eq!(ev.importance, EventImportance::Major);
    assert_eq!(ev.text, "Hostile fleet stalled by bureaucratic red tape.");
}
