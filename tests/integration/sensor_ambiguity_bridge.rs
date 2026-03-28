use bevy::prelude::*;
use scale::layer2::fleet::{Fleet, FleetFaction, InOrbit};
use scale::layer2::sensor_ambiguity::{SensorContact, Sensors, UnidentifiedContact, resolve_sensors_system};
use scale::layer2::integration::{assign_sensors_to_player_fleets_system, ensure_player_fleets_identified_system};

#[test]
fn player_fleets_have_sensors_and_are_identified() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add systems in order
    app.add_systems(
        Update,
        (
            assign_sensors_to_player_fleets_system,
            resolve_sensors_system,
            ensure_player_fleets_identified_system,
        ).chain()
    );

    // Spawn an empty planet
    let planet = app.world_mut().spawn_empty().id();
    let faraway_planet = app.world_mut().spawn_empty().id();

    // Spawn Player Fleet A without Sensors (should be added)
    let player_fleet = app.world_mut().spawn((
        Fleet,
        FleetFaction::Player,
        InOrbit { parent: planet },
    )).id();

    // Spawn Pirate Fleet B far away (should be unidentified)
    let pirate_fleet = app.world_mut().spawn((
        Fleet,
        FleetFaction::Pirate,
        InOrbit { parent: faraway_planet },
    )).id();

    app.update();

    // Verify Player Fleet has Sensors
    assert!(
        app.world().get::<Sensors>(player_fleet).is_some(),
        "Player Fleet should be assigned Sensors"
    );

    // Verify Player Fleet is NOT UnidentifiedContact
    assert!(
        app.world().get::<UnidentifiedContact>(player_fleet).is_none(),
        "Player Fleet should never be unidentified"
    );

    let contact_player = app.world().get::<SensorContact>(player_fleet).expect("Player Fleet should have SensorContact");
    assert_eq!(
        contact_player.resolved_entity,
        Some(player_fleet),
        "Player Fleet's SensorContact should be resolved to itself"
    );

    // Verify Pirate Fleet IS UnidentifiedContact
    assert!(
        app.world().get::<UnidentifiedContact>(pirate_fleet).is_some(),
        "Faraway Pirate Fleet should be unidentified"
    );

    let contact_pirate = app.world().get::<SensorContact>(pirate_fleet).expect("Pirate Fleet should have SensorContact");
    assert!(
        contact_pirate.resolved_entity.is_none(),
        "Faraway Pirate Fleet's SensorContact should not be resolved"
    );
}
