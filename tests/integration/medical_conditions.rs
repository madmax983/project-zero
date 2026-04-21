#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::cryo_dreams::CryoTrauma;
    use scale::layer1::health::Health;
    use scale::layer1::map::GridPosition;
    use scale::layer1::medical::{healing_system, Hospital};
    use scale::layer1::radioactive::RadiationSickness;

    #[test]
    fn test_healing_system_cures_cryo_trauma() {
        let mut world = World::new();
        // Setup Hospital
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Patient with CryoTrauma (and full HP)
        let patient = world
            .spawn((
                Health { current: 100.0, max: 100.0, conditions: Vec::new(), },
                AssignedTo {
                    entity: hospital,
                    assignment_type: AssignmentType::Patient,
                },
                CryoTrauma { severity: 0.8 },
            ))
            .id();

        // Run healing system
        let mut schedule = Schedule::default();
        schedule.add_systems(healing_system);
        schedule.run(&mut world);

        // Assert Trauma is reduced or removed
        let trauma = world.get::<CryoTrauma>(patient);
        if let Some(t) = trauma {
            assert!(t.severity < 0.8, "Trauma severity should decrease");
        } else {
            // Removed is also fine
        }
    }

    #[test]
    fn test_healing_system_cures_radiation_sickness() {
        let mut world = World::new();
        // Setup Hospital
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                Hospital::default(),
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Spawn Patient with RadiationSickness (and full HP)
        let patient = world
            .spawn((
                Health { current: 100.0, max: 100.0, conditions: Vec::new(), },
                AssignedTo {
                    entity: hospital,
                    assignment_type: AssignmentType::Patient,
                },
                RadiationSickness { severity: 50.0 },
            ))
            .id();

        // Run healing system
        let mut schedule = Schedule::default();
        schedule.add_systems(healing_system);
        schedule.run(&mut world);

        // Assert Sickness is reduced
        let sickness = world.get::<RadiationSickness>(patient).unwrap();
        assert!(
            sickness.severity < 50.0,
            "Radiation Sickness should decrease"
        );
        // Ensure effective treatment (more than 0.1 natural decay)
        assert!(
            sickness.severity <= 49.0,
            "Hospital should reduce severity by at least 1.0"
        );
    }
}
