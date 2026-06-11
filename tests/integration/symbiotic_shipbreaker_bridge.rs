use bevy_app::{App, Update};
use bevy_ecs::event::Events;
use bevy_ecs::schedule::IntoSystemConfigs;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::symbiotic_salvage_chronicle_bridge;
use scale::layer1::shipbreaking_symbiotic::{
    simulate_shipbreaker_salvage, DerelictEcosystem, ShipbreakerMission, SymbioticSalvageEvent,
};
use scale::layer1::economy::resources::ColonyResources;
use scale::shared::colony::ColonyName;
use scale::shared::narrative::NarrativeGenerator;
use scale::shared::time::SimulationTime;

#[test]
fn test_symbiotic_shipbreaker_chronicle_bridge() {
    let mut app = App::new();

    app.insert_resource(ColonyResources::default());
    app.insert_resource(ColonyName {
        name: "Test Colony".to_string(),
    });
    app.insert_resource(SimulationTime {
        tick: 0,
        ..Default::default()
    });
    app.insert_resource(NarrativeGenerator::from_embedded());

    app.add_event::<SymbioticSalvageEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (
            simulate_shipbreaker_salvage,
            symbiotic_salvage_chronicle_bridge,
        )
            .chain(),
    );

    let derelict = app
        .world_mut()
        .spawn(DerelictEcosystem {
            threat_level: 5.0, // Low threat so one tick despawns it
            salvage_yield: 25.0,
        })
        .id();

    app.world_mut().spawn(ShipbreakerMission {
        target_derelict: derelict,
        progress: 0.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        emitted[0].text.contains("Stellar Alloy") || emitted[0].text.contains("salvaged"),
        "Text should contain narrative output: {}",
        emitted[0].text
    );
}
