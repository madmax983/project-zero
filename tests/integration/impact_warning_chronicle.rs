use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, Chronicle, EventImportance};
use scale::layer1::core::integration::impact_warning_chronicle_bridge;
use scale::layer1::core::map::GridPosition;
use scale::layer1::environment::impact::ImpactWarningEvent;

#[test]
fn impact_warning_triggers_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<ImpactWarningEvent>();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(Chronicle::default());

    app.add_systems(Update, impact_warning_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<ImpactWarningEvent>>()
        .send(ImpactWarningEvent {
            target_pos: GridPosition { x: 10, y: 20 },
            ticks_remaining: 100,
        });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Impact Warning"));
}
