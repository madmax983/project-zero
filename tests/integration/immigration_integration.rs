#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::customs::ImmigrationStatus;
    use scale::layer1::integration::immigration_integration_system;
    use scale::layer1::notifications::NotificationQueue;
    use scale::layer1::pop::PopName;
    use scale::layer1::visitor::{Visitor, VisitorState};
    use scale::shared::time::SimulationTime;

    #[test]
    fn test_vetted_visitor_enters() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NotificationQueue::default());

        let visitor = world
            .spawn((
                Visitor {
                    state: VisitorState::Arriving,
                    ..Default::default()
                },
                ImmigrationStatus::Vetted,
                PopName("Test Visitor".to_string()),
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(immigration_integration_system);
        schedule.run(&mut world);

        // Assert
        let visitor_state = world.get::<Visitor>(visitor).unwrap();
        assert_eq!(
            visitor_state.state,
            VisitorState::Loitering,
            "Vetted visitor should start loitering"
        );

        // Check Notification
        let notifications = world.resource::<NotificationQueue>();
        assert!(
            notifications.active.iter().any(|n| n.text.contains("allowed entry")),
            "Should notify entry"
        );
    }

    #[test]
    fn test_rejected_visitor_leaves() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NotificationQueue::default());

        let visitor = world
            .spawn((
                Visitor {
                    state: VisitorState::Arriving,
                    ..Default::default()
                },
                ImmigrationStatus::Rejected("Spy".to_string()),
                PopName("Suspicious Guy".to_string()),
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(immigration_integration_system);
        schedule.run(&mut world);

        // Assert
        let visitor_state = world.get::<Visitor>(visitor).unwrap();
        assert_eq!(
            visitor_state.state,
            VisitorState::Departing,
            "Rejected visitor should start departing"
        );

        // Check Notification
        let notifications = world.resource::<NotificationQueue>();
        assert!(
            notifications.active.iter().any(|n| n.text.contains("denied entry")),
            "Should notify rejection"
        );
    }

    #[test]
    fn test_pending_visitor_unchanged() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(NotificationQueue::default());

        let visitor = world
            .spawn((
                Visitor {
                    state: VisitorState::Arriving,
                    ..Default::default()
                },
                ImmigrationStatus::Pending,
                PopName("Waiting Guy".to_string()),
            ))
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(immigration_integration_system);
        schedule.run(&mut world);

        // Assert
        let visitor_state = world.get::<Visitor>(visitor).unwrap();
        assert_eq!(
            visitor_state.state,
            VisitorState::Arriving,
            "Pending visitor should stay arriving"
        );
    }
}
