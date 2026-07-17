use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::fleet::{Fleet, InTransit};
use scale::layer2::navigation::chronological_stutter::{
    apply_chronological_stutter_system, HyperlaneTransitEvent,
};

#[test]
fn test_chronological_stutter_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();
    app.add_event::<HyperlaneTransitEvent>();

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

    app.add_systems(Update, apply_chronological_stutter_system);

    let mut found_event = false;
    for _ in 0..100 {
        app.update();
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<&AddChronicleEvent> = reader.read(events).collect();
        if !emitted.is_empty() {
            assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
            assert_eq!(emitted[0].importance, EventImportance::Major);
            assert!(emitted[0].text.contains("Chronological Stutter"));
            found_event = true;
            break;
        }

        app.world_mut().send_event(HyperlaneTransitEvent {
            fleet,
            is_unstable: true,
        });
    }

    assert!(
        found_event,
        "Chronological Stutter failed to emit an AddChronicleEvent over 100 trials"
    );
}
