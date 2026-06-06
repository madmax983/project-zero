use bevy::prelude::*;
use scale::layer1::psychology::void_stare::{VoidExposure, void_manifestation_system};
use scale::layer1::utility_types::{ActionType, PopAction};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::pop::Pop;
use scale::layer1::core::integration::void_stare_chronicle_bridge;
use scale::shared::log::MessageLog;

#[test]
fn test_void_stare_chronicle() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.insert_resource(MessageLog::default());

    app.add_systems(Update, (
        void_manifestation_system,
        void_stare_chronicle_bridge
    ).chain());

    let _pop = app.world_mut().spawn((
        Pop,
        VoidExposure {
            current: 85.0,
            susceptibility: 1.0,
            check_timer: 0,
        },
        PopAction {
            current: ActionType::Work,
            ticks_committed: 10,
            current_utility: 0.0,
        }
    )).id();

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    assert_eq!(cursor.read(events).count(), 1, "Should emit one AddChronicleEvent");
}
