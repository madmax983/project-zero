#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::customs::{vetting_work_system, ImmigrationStatus};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::zone::{ZoneGrid, ZoneType};

    #[test]
    fn test_vetting_requires_presence_at_customs() {
        // Setup
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(20, 20);
        zone_grid.set(10, 10, ZoneType::Customs);
        world.insert_resource(zone_grid);

        // Spawn visitor at (0, 0) - NOT at Customs
        let visitor = world
            .spawn((
                Pop::default(),
                ImmigrationStatus::Pending,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system twice to ensure any "initialization" frame is passed and progress would happen
        let mut schedule = Schedule::default();
        schedule.add_systems(vetting_work_system);
        schedule.run(&mut world);
        schedule.run(&mut world);

        // Assert: Status should still be Pending because they are not at customs
        let status = world.get::<ImmigrationStatus>(visitor).unwrap();

        // This assertion is expected to FAIL with the current implementation
        assert_eq!(
            *status,
            ImmigrationStatus::Pending,
            "Visitor should not start vetting if not at customs (Current: {:?})",
            status
        );

        // Move visitor to (10, 10) - At Customs
        let mut query = world.query::<&mut GridPosition>();
        let mut pos = query.get_mut(&mut world, visitor).unwrap();
        pos.x = 10;
        pos.y = 10;

        // Run system again
        schedule.run(&mut world);

        // Assert: NOW it should be Processing
        let status_at_customs = world.get::<ImmigrationStatus>(visitor).unwrap();
        assert!(
            matches!(*status_at_customs, ImmigrationStatus::Processing(_)),
            "Visitor at customs should start vetting (Current: {:?})",
            status_at_customs
        );
    }
}
