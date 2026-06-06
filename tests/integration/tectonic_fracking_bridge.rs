use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::{
    tectonic_fracking_chronicle_bridge, trigger_tectonic_fracking_system,
};
use scale::layer1::geology::fracking::{tectonic_fracking_system, FrackEvent, TectonicFracker};
use scale::layer1::resources::ColonyResources;

#[test]
fn test_landfill_triggers_fracking() {
    let mut app = App::new();

    // Register events and resources
    app.add_event::<FrackEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(ColonyResources {
        waste: 100.0,
        ..Default::default()
    });
    app.insert_resource(bevy_time::Time::<bevy_time::Real>::default());
    app.insert_resource(bevy_time::Time::<bevy_time::Virtual>::default());

    // Add integration systems
    app.add_systems(
        bevy_app::Update,
        (
            trigger_tectonic_fracking_system,
            tectonic_fracking_system,
            tectonic_fracking_chronicle_bridge,
        )
            .chain(),
    );

    // Spawn a Landfill with the TectonicFracker component
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Landfill,
        },
        TectonicFracker,
    ));

    // Run the app multiple times to pass the timer threshold (e.g. 10.0 seconds)
    // Actually we don't have Time tick control easily, so we might just use a Local timer and advance time.
    let mut time: bevy_time::Time<()> = bevy_time::Time::default();
    time.advance_by(std::time::Duration::from_secs(15));
    app.insert_resource(time);
    app.init_resource::<scale::layer1::geology::tectonic::TectonicStress>();

    app.update();

    // Check that FrackEvent was triggered and recorded in Chronicle
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut found = false;
    for event in chronicle_events.get_cursor().read(chronicle_events) {
        if event.text.contains("fracking") {
            assert_eq!(event.importance, EventImportance::Major);
            found = true;
        }
    }
    assert!(found, "Chronicle event should be recorded for fracking");

    // Waste should be consumed (60.0 -> 50.0)
    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.waste, 90.0);
}
