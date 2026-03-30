#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::environment::events::DebrisFallEvent;
    use scale::layer1::map::GridPosition;
    use scale::layer2::events_new::reverse_quarantine::{
        process_refugee_decisions_system, Decision, RefugeeFleetEvent,
    };
    use scale::layer2::integration::reverse_quarantine_chronicle_bridge;

    #[test]
    fn test_reverse_quarantine_chronicle_bridge_reject() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.init_resource::<Events<RefugeeFleetEvent>>();
        app.init_resource::<Events<DebrisFallEvent>>();
        app.init_resource::<Events<AddChronicleEvent>>();

        app.add_systems(
            Update,
            (
                process_refugee_decisions_system,
                reverse_quarantine_chronicle_bridge,
            )
                .chain(),
        );

        // Send a rejected refugee fleet event
        app.world_mut().send_event(RefugeeFleetEvent {
            fleet_size: 4,
            decision: Some(Decision::Reject),
            target_location: GridPosition { x: 50, y: 50 },
        });

        // Run the schedule
        app.update();

        // Verify the chronicle event was emitted
        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let emitted: Vec<_> = reader.read(chronicle_events).collect();

        assert_eq!(
            emitted.len(),
            1,
            "Should emit one chronicle event for rejected fleet"
        );
        assert!(emitted[0].text.contains("Desperate refugee fleet repelled"));
        assert_eq!(emitted[0].importance, EventImportance::Major);
    }
}
