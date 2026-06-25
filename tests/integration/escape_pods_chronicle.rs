#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::actions::escape::{process_lifeboat_launches, DistressSignal, Lifeboat};
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::core::integration::escape_pods_chronicle_bridge;

    #[test]
    fn test_escape_pods_launch_chronicle_event() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();

        // Setup integration chain
        app.add_systems(
            Update,
            (process_lifeboat_launches, escape_pods_chronicle_bridge).chain(),
        );

        let pop1 = app.world_mut().spawn_empty().id();
        let pop2 = app.world_mut().spawn_empty().id();

        // Spawn Lifeboat
        app.world_mut().spawn(Lifeboat {
            capacity: 4,
            occupants: vec![pop1, pop2],
            launch_triggered: true,
        });

        app.update();

        // Assert DistressSignal was spawned
        let distress_signals = app
            .world_mut()
            .query::<&DistressSignal>()
            .iter(app.world())
            .count();
        assert_eq!(distress_signals, 1, "DistressSignal should be spawned");

        // Assert Chronicle event was emitted
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = events.get_cursor();
        assert_eq!(cursor.len(events), 1, "Should emit one AddChronicleEvent");
        let event = cursor.read(events).next().unwrap();
        assert!(event.text.contains("A lifeboat carrying 2 pop"));

        // Assert Pops were removed from the world
        assert!(
            app.world().get_entity(pop1).is_err(),
            "pop1 should be despawned from the simulation"
        );
        assert!(
            app.world().get_entity(pop2).is_err(),
            "pop2 should be despawned from the simulation"
        );
    }
}
