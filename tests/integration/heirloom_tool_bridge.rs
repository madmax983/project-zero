use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::heirloom_tool::HeirloomTool;
use scale::layer1::core::integration::heirloom_tool_chronicle_bridge;
use scale::layer1::psychology::traits::Traits;
use scale::shared::colony::ColonyName;
use scale::shared::narrative::NarrativeGenerator;

#[test]
fn test_heirloom_tool_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<AddChronicleEvent>();
    app.insert_resource(NarrativeGenerator::from_embedded());
    app.insert_resource(ColonyName {
        name: "Test Colony".to_string(),
    });

    app.add_systems(Update, heirloom_tool_chronicle_bridge);

    app.world_mut().spawn(HeirloomTool {
        original_traits: Traits::default(),
        efficiency_boost: 50.0,
        original_owner_name: "Master Crafter Bob".to_string(),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one Chronicle event");
    assert!(
        emitted[0].text.contains("Master Crafter Bob"),
        "Event text should contain the original owner's name"
    );
    assert!(
        matches!(emitted[0].importance, EventImportance::Major),
        "Heirloom creation should be a major event"
    );
}
