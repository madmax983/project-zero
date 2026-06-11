use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::symbiotic_salvage_chronicle_bridge;
use scale::layer1::shipbreaking_symbiotic::SymbioticSalvageCompleteEvent;
use scale::shared::colony::ColonyName;
use scale::shared::narrative::NarrativeGenerator;
use scale::shared::time::SimulationTime;

#[test]
fn test_symbiotic_salvage_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<SymbioticSalvageCompleteEvent>();
    app.add_event::<AddChronicleEvent>();

    app.insert_resource(SimulationTime::default());
    app.insert_resource(ColonyName("Test Colony".to_string()));
    app.insert_resource(NarrativeGenerator::from_embedded());

    app.add_systems(Update, symbiotic_salvage_chronicle_bridge);

    app.world_mut().send_event(SymbioticSalvageCompleteEvent {
        derelict_name: "Derelict Hull".to_string(),
        salvage_yield: 50.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert!(emitted[0].text.contains("Derelict Hull"), "Narrative should include derelict name");
    assert!(emitted[0].text.contains("50"), "Narrative should include salvage yield");
}
