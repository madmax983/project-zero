use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::navigation::chronological_stutter::{
    apply_chronological_stutter_system, HyperlaneTransitEvent,
};
use scale::layer2::fleet::{Fleet, InTransit};

#[test]
fn test_chronological_stutter_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<HyperlaneTransitEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, apply_chronological_stutter_system);

    // The random stutter is between -50 and 500, so negative stutter happens ~9% of the time.
    let mut found_event = false;

    for _ in 0..1000 {
        // Spawning a new entity in each loop to ensure no side effects break the loop
        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InTransit {
                    origin: Entity::PLACEHOLDER,
                    destination: Entity::PLACEHOLDER,
                    progress: 0.0,
                    duration: 100.0,
                },
            ))
            .id();

        app.world_mut().send_event(HyperlaneTransitEvent {
            fleet,
            is_unstable: true,
        });
        app.update();

        // Read events inside the loop because Bevy clears events after two frames.
        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

        if let Some(event) = events.first() {
            found_event = true;
            // Verify the content of the event
            assert_eq!(event.importance, EventImportance::Major);
            assert!(
                event.text.contains("paradox occurred"),
                "Chronicle event should mention paradox"
            );
            break;
        }
    }

    assert!(
        found_event,
        "Should emit at least one AddChronicleEvent for negative stutters"
    );
}
