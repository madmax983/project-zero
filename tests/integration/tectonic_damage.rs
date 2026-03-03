#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::geology::tectonic::MegaQuakeEvent;
    use scale::layer1::health::Health;
    use scale::layer1::integration::mega_quake_integration_system;
    use scale::layer1::notifications::{NotificationQueue, NotificationSeverity};
    use scale::shared::time::SimulationTime;

    #[test]
    fn test_mega_quake_damages_buildings_and_notifies() {
        let mut world = World::new();

        // Register resources
        world.insert_resource(NotificationQueue::default());
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        world.init_resource::<Events<MegaQuakeEvent>>();

        // Register system
        let mut schedule = Schedule::default();
        schedule.add_systems(mega_quake_integration_system);

        // Setup: spawn a couple of buildings with health
        let b1 = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let b2 = world
            .spawn((
                Building {
                    building_type: BuildingType::Generator,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Send MegaQuakeEvent
        world.send_event(MegaQuakeEvent);

        // Run the schedule
        schedule.run(&mut world);

        // Assert: health reduced by 50.0
        assert_eq!(world.get::<Health>(b1).unwrap().current, 50.0);
        assert_eq!(world.get::<Health>(b2).unwrap().current, 50.0);

        // Assert: Notification added
        let queue = world.resource::<NotificationQueue>();
        assert_eq!(queue.active.len(), 1);
        assert_eq!(queue.active[0].severity, NotificationSeverity::Error);
        assert!(queue.active[0].text.contains("Mega-Quake"));
    }
}
