use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::subconscious_grid_lockdown_chronicle_bridge;
use scale::layer1::infrastructure::subconscious_grid::{GridState, SmartGrid};

#[test]
fn test_subconscious_grid_lockdown_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, subconscious_grid_lockdown_chronicle_bridge);

    let colony = app
        .world_mut()
        .spawn(SmartGrid {
            state: GridState::Normal,
        })
        .id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    assert_eq!(cursor.read(events).count(), 0);

    app.world_mut()
        .get_mut::<SmartGrid>(colony)
        .unwrap()
        .state = GridState::Lockdown;

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("lockdown protocol"));
}
