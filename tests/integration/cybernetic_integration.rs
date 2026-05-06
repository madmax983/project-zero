use bevy::prelude::*;
use scale::layer1::biology::cybernetic_ascendancy::CyberneticIntegration;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::cybernetic_integration_chronicle_bridge;
use scale::layer1::pop::{Pop, PopName};

#[test]
fn test_cybernetic_integration_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, cybernetic_integration_chronicle_bridge);

    let pop_entity = app
        .world_mut()
        .spawn((
            Pop,
            PopName("Test Pop".to_string()),
            CyberneticIntegration {
                integration_level: 0.5,
            },
        ))
        .id();

    app.update();

    let events = app
        .world()
        .resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    assert_eq!(cursor.read(events).count(), 0);

    // Update integration level to 1.0
    app.world_mut()
        .get_mut::<CyberneticIntegration>(pop_entity)
        .unwrap()
        .integration_level = 1.0;

    app.update();

    let events = app
        .world()
        .resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();

    let mut count = 0;
    for event in cursor.read(events) {
        assert_eq!(event.text, "Test Pop has achieved complete Cybernetic Integration.");
        count += 1;
    }
    assert_eq!(count, 1);
}
