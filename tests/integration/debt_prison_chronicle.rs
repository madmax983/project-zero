use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::events::debt_prison::{AcceptBailoutEvent, process_bailout_acceptance_system};
use scale::layer1::integration::debt_bailout_chronicle_bridge;
use scale::layer2::trade::blockade::ColonyDebt;

#[test]
fn test_debt_bailout_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<AcceptBailoutEvent>();
    app.add_event::<AddChronicleEvent>();
    app.world_mut().insert_resource(ColonyDebt { amount: 1000000.0, threshold: 50000.0 });

    app.add_systems(Update, (
        process_bailout_acceptance_system,
        debt_bailout_chronicle_bridge.after(process_bailout_acceptance_system),
    ));

    app.world_mut().send_event(AcceptBailoutEvent);
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let reader = events.get_cursor();
    let emitted = reader.len(events);

    assert_eq!(emitted, 1, "Expected one AddChronicleEvent to be emitted");

    // Verify importance is Legendary
    let mut reader2 = events.get_cursor();
    let first_event = reader2.read(events).next().unwrap();
    assert_eq!(first_event.importance, EventImportance::Legendary);
    assert!(first_event.event_type.contains("Bailout"));
}
