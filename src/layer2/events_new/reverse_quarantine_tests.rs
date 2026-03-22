#[cfg(test)]
mod tests {
    use crate::layer1::environment::events::DebrisFallEvent;
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer2::events_new::reverse_quarantine::{
        process_refugee_decisions_system, Decision, RefugeeFleetEvent,
    };
    use bevy_ecs::prelude::*;

    #[test]
    fn test_reject_refugees_causes_orbital_reprisal_and_guilt() {
        let mut world = World::new();
        world.insert_resource(Events::<RefugeeFleetEvent>::default());
        world.insert_resource(Events::<DebrisFallEvent>::default());

        // Setup a pop for morale checks
        let pop = world
            .spawn(Morale {
                value: 1.0,
                ..Default::default()
            })
            .id();

        // Trigger a refugee event, resolving as Rejected
        world.send_event(RefugeeFleetEvent {
            fleet_size: 5,
            decision: Some(Decision::Reject),
            target_location: GridPosition { x: 50, y: 50 },
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_refugee_decisions_system);
        schedule.run(&mut world);

        // Assert that debris fell as a result of repelling them
        let debris_events = world.resource::<Events<DebrisFallEvent>>();
        let reader = debris_events.get_cursor();
        assert_eq!(reader.len(debris_events), 5);

        // Assert that guilt spiked
        let pop_morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(
            pop_morale.modifiers.len(),
            1,
            "Rejecting desperate refugees should cause massive guilt/stress"
        );
        assert_eq!(pop_morale.modifiers[0].value, -0.25);
    }

    #[test]
    fn test_accept_refugees_introduces_plague() {
        let mut world = World::new();
        world.insert_resource(Events::<RefugeeFleetEvent>::default());
        world.insert_resource(Events::<DebrisFallEvent>::default());

        let pop = world
            .spawn(Morale {
                value: 1.0,
                ..Default::default()
            })
            .id();

        // Trigger a refugee event, resolving as Accepted
        world.send_event(RefugeeFleetEvent {
            fleet_size: 5,
            decision: Some(Decision::Accept),
            target_location: GridPosition { x: 50, y: 50 },
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_refugee_decisions_system);
        schedule.run(&mut world);

        let debris_events = world.resource::<Events<DebrisFallEvent>>();
        let reader = debris_events.get_cursor();
        assert_eq!(
            reader.len(debris_events),
            0,
            "Accepting refugees should not cause orbital debris"
        );

        let pop_morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(
            pop_morale.modifiers.len(),
            0,
            "Accepting refugees should not cause immediate massive guilt"
        );
    }
}
