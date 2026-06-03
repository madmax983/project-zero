use bevy::prelude::*;
use scale::layer1::core::integration::contraband_possession_crime_bridge_system;
use scale::layer1::law::contraband::ContrabandPossession;
use scale::layer1::law::justice::{
    process_crimes_system, CrimeCommittedEvent, CrimeRecord,
};

#[test]
fn test_contraband_possession_triggers_smuggling_crime() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CrimeCommittedEvent>();

    app.add_systems(
        Update,
        (
            contraband_possession_crime_bridge_system,
            process_crimes_system,
        )
            .chain(),
    );

    let pop = app
        .world_mut()
        .spawn((ContrabandPossession, CrimeRecord::default()))
        .id();

    app.update();

    let record = app.world().get::<CrimeRecord>(pop).unwrap();
    assert!(
        record.wanted,
        "Pop should be wanted for possessing contraband"
    );
    assert_eq!(
        record.severity, 40,
        "Crime severity should be 40 for Smuggling"
    );
}
