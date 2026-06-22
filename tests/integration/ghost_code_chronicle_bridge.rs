use bevy::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::events::BuildingCompletedEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::tech::ghost_code::{DataResidue, GhostCode, GhostTrait};

// The integration function to write
use scale::layer1::core::integration::ghost_code_chronicle_bridge;

#[test]
fn test_ghost_code_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingCompletedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, ghost_code_chronicle_bridge);

    let pos = GridPosition { x: 5, y: 5 };

    let new_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Tower,
            },
            pos,
            GhostCode {
                traits: vec![GhostTrait::LegacyTargeting],
            },
        ))
        .id();

    // Trigger completion event
    app.world_mut().send_event(BuildingCompletedEvent {
        entity: new_building,
    });

    // Run schedule
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit exactly one AddChronicleEvent when a building acquires GhostCode"
    );
}
