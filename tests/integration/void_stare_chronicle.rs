use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::{void_stare_chronicle_bridge, VoidStareChronicleLogged};
use scale::layer1::pop::Pop;
use scale::layer1::utility_types::{ActionType, PopAction};

#[test]
fn test_void_stare_chronicle_bridge() {
    let mut app = World::new();

    // Initialize events
    app.init_resource::<Events<AddChronicleEvent>>();

    // Create a pop with VoidStare action
    let pop = app
        .spawn((
            Pop,
            PopAction {
                current: ActionType::VoidStare,
                ..Default::default()
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(void_stare_chronicle_bridge);

    // Act: Run system
    schedule.run(&mut app);

    // Assert: Chronicle event sent and marker added
    {
        let events = app.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let events_list: Vec<_> = reader.read(events).collect();

        assert_eq!(events_list.len(), 1, "Should emit one Chronicle event");
        assert!(
            events_list[0].text.contains("Abyss"),
            "Chronicle event should mention Abyss"
        );
    }
    assert!(
        app.get::<VoidStareChronicleLogged>(pop).is_some(),
        "Pop should have marker"
    );

    // Clear events
    app.resource_mut::<Events<AddChronicleEvent>>().clear();

    // Act: Run again, should not emit another event
    schedule.run(&mut app);
    {
        let events = app.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let events_list: Vec<_> = reader.read(events).collect();
        assert_eq!(events_list.len(), 0, "Should not emit duplicate event");
    }

    // Act: Change action away from VoidStare
    app.get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;
    schedule.run(&mut app);

    // Assert: Marker removed
    assert!(
        app.get::<VoidStareChronicleLogged>(pop).is_none(),
        "Marker should be removed"
    );
}
