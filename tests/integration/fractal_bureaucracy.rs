use bevy::prelude::*;
use scale::layer1::administration::fractal_bureaucracy::LogicCascadeEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::fractal_bureaucracy_chronicle_bridge;

fn spawn_test_world() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Events<LogicCascadeEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, fractal_bureaucracy_chronicle_bridge);

    app
}

#[test]
fn test_logic_cascade_triggers_chronicle_event() {
    let mut app = spawn_test_world();

    // Trigger Logic Cascade Event
    app.world_mut().send_event(LogicCascadeEvent);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        events[0].text.contains("Logic Cascade"),
        "Event text should mention Logic Cascade"
    );
    assert!(
        matches!(events[0].importance, EventImportance::Major),
        "Event should be major"
    );
}
