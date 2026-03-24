use bevy_app::App;
use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::integration::nanite_breach_chronicle_bridge;
use scale::layer1::items::ItemType;
use scale::layer1::map::GridPosition;
use scale::layer1::nanite_fabrication::{
    nanite_fabrication_system, ContainmentBreachEvent, Nanoforge,
};
use scale::layer1::resources::ColonyResources;

#[test]
fn test_nanoforge_containment_breach_emits_chronicle() {
    let mut app = App::new();

    // Register events
    app.add_event::<ContainmentBreachEvent>();
    app.add_event::<AddChronicleEvent>();

    // Initializing resources
    app.insert_resource(ColonyResources::default());

    // Add systems
    app.add_systems(
        bevy_app::Update,
        (nanite_fabrication_system, nanite_breach_chronicle_bridge).chain(),
    );

    // Setup guaranteed breach
    let forge_entity = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            Nanoforge {
                active_recipe: Some(ItemType::Tool),
                breach_risk: 1.0, // Guaranteed breach
            },
            GridPosition { x: 10, y: 20 },
        ))
        .id();

    // Act
    app.update();

    // Assert ContainmentBreachEvent was fired
    let breach_events = app.world().resource::<Events<ContainmentBreachEvent>>();
    let mut breach_cursor = breach_events.get_cursor();
    let breaches: Vec<_> = breach_cursor.read(breach_events).collect();
    assert_eq!(breaches.len(), 1);
    assert_eq!(breaches[0].source_entity, forge_entity);

    // Assert AddChronicleEvent was fired with Major importance
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut chronicle_cursor = chronicle_events.get_cursor();
    let chronologies: Vec<_> = chronicle_cursor.read(chronicle_events).collect();
    assert_eq!(
        chronologies.len(),
        1,
        "Should emit a chronicle event on containment breach"
    );
    assert_eq!(chronologies[0].importance, EventImportance::Major);
    assert!(chronologies[0].text.contains("Containment Breach"));
    assert!(chronologies[0].text.contains("(10, 20)"));
}
