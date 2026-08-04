use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::administration::admin::AdminStats;
use scale::layer1::administration::sentient_bureaucracy::{
    autonomous_work_reassignment_system, update_bureaucracy_sentience_system,
    SentientBureaucracyState, TaskAdministrativelyOptimizedEvent,
};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::sentient_bureaucracy_chronicle_bridge;
use scale::layer1::mind::utility_types::{ActionType, PopAction};

#[test]
fn sentient_bureaucracy_task_optimization_logged_to_chronicle() {
    let mut app = App::new();

    app.add_event::<TaskAdministrativelyOptimizedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (
            update_bureaucracy_sentience_system,
            autonomous_work_reassignment_system,
            sentient_bureaucracy_chronicle_bridge,
        )
            .chain(),
    );

    app.insert_resource(AdminStats {
        supply: 100.0,
        demand: 150.0,
        efficiency: 0.5,
    });

    app.insert_resource(SentientBureaucracyState {
        strain_duration: 100.0, // immediately trigger active state
        is_active: true,
    });

    let _pop_entity = app
        .world_mut()
        .spawn(PopAction {
            current: ActionType::Work,
            current_utility: 10.0,
            ticks_committed: 0,
        })
        .id();

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();

    let events: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(
        events[0].text,
        "The Sentient Bureaucracy has autonomously reassigned a task, overriding player input for the sake of optimization."
    );
}
