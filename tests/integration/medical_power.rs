#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::building::{spawn_building, BuildingType, MaterialType};
    use scale::layer1::energy::{power_grid_system, BlackoutProtocol, PowerConsumer};
    use scale::layer1::health::Health;
    use scale::layer1::medical::{healing_system, Hospital};

    fn setup_world() -> World {
        let mut world = World::new();
        // Required resources
        world.insert_resource(scale::layer1::resources::ColonyResources::default());
        world.insert_resource(scale::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![scale::layer1::terrain::TerrainType::Grass; 100],
        });
        world.insert_resource(BlackoutProtocol::default());
        world.insert_resource(scale::layer1::medical::MedicalPolicy::default());
        world
    }

    #[test]
    fn test_hospital_spawns_with_power_consumer() {
        let mut world = setup_world();
        spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Hospital,
            MaterialType::default(),
        );

        let entity = world
            .query_filtered::<Entity, With<Hospital>>()
            .single(&world);
        let consumer = world.get::<PowerConsumer>(entity);

        assert!(
            consumer.is_some(),
            "Hospital must have PowerConsumer component"
        );
    }

    #[test]
    fn test_healing_requires_active_power() {
        let mut world = setup_world();

        // 1. Spawn Generator at (0,0)
        spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Generator,
            MaterialType::default(),
        );

        // 2. Spawn Hospital at (0,1) - Connected via adjacency (assuming adjacency works for grid)
        // Or ensure we add Conduit. Energy system uses adjacency.
        // Generator is PowerSource. Hospital is PowerConsumer (expected).
        spawn_building(
            &mut world,
            0,
            1,
            BuildingType::Hospital,
            MaterialType::default(),
        );

        let hospital_entity = world
            .query_filtered::<Entity, With<Hospital>>()
            .single(&world);

        // Manually insert PowerConsumer if it doesn't exist yet (so we can test healing logic separately from spawn logic if needed)
        // But for integration, we want it all to work.
        // If spawn fails, the previous test catches it.
        // Here, let's assume it exists or insert it to test the HEALING logic connection.
        // Actually, if we want to test the full chain, we should fail if it's missing.
        // However, to test step 4 logic, we need the component.

        // Let's rely on `test_hospital_spawns_with_power_consumer` to catch the missing component.
        // Here we can force insert it to test the *system interaction* even if spawn is broken,
        // OR we can let it fail. I'll let it fail naturally if missing.

        // 3. Spawn Patient
        let patient = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        // 4. Run Power System -> Should activate Hospital
        power_grid_system(&mut world);

        // Verify Active
        if let Some(cons) = world.get::<PowerConsumer>(hospital_entity) {
            assert!(cons.active, "Hospital should be powered");
        } else {
            // If component missing, we can't test power logic, fail.
            panic!("Hospital missing PowerConsumer");
        }

        // 5. Run Healing System -> Should Heal
        healing_system(&mut world);

        let health = world.get::<Health>(patient).unwrap();
        assert!(
            health.current > 50.0,
            "Patient should be healed when hospital is powered"
        );
    }

    #[test]
    fn test_healing_fails_without_power() {
        let mut world = setup_world();

        // 1. Spawn Hospital (Isolated, no generator)
        spawn_building(
            &mut world,
            5,
            5,
            BuildingType::Hospital,
            MaterialType::default(),
        );

        let hospital_entity = world
            .query_filtered::<Entity, With<Hospital>>()
            .single(&world);

        // 2. Spawn Patient
        let patient = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        // 3. Run Power System -> Should DEACTIVATE Hospital (default might be active=false, but check)
        power_grid_system(&mut world);

        if let Some(cons) = world.get::<PowerConsumer>(hospital_entity) {
            assert!(!cons.active, "Hospital should be unpowered");
        } else {
            // Forcing component for test if spawn not updated yet
            world.entity_mut(hospital_entity).insert(PowerConsumer {
                demand: 5.0,
                active: false,
            });
        }

        // 4. Run Healing System -> Should NOT Heal
        healing_system(&mut world);

        let health = world.get::<Health>(patient).unwrap();
        assert_eq!(
            health.current, 50.0,
            "Patient should NOT be healed when hospital is unpowered"
        );
    }

    #[test]
    fn test_blackout_stops_healing() {
        let mut world = setup_world();

        // 1. Activate Blackout
        world.resource_mut::<BlackoutProtocol>().active = true;

        // 2. Spawn Generator & Hospital (Connected)
        spawn_building(
            &mut world,
            0,
            0,
            BuildingType::Generator,
            MaterialType::default(),
        );
        spawn_building(
            &mut world,
            0,
            1,
            BuildingType::Hospital,
            MaterialType::default(),
        );

        let hospital_entity = world
            .query_filtered::<Entity, With<Hospital>>()
            .single(&world);
        let patient = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                AssignedTo {
                    assignment_type: AssignmentType::Patient,
                    entity: hospital_entity,
                },
            ))
            .id();

        // 3. Run Power System
        power_grid_system(&mut world);

        // 4. Verify Inactive
        if let Some(cons) = world.get::<PowerConsumer>(hospital_entity) {
            assert!(!cons.active, "Hospital should be inactive during blackout");
        } else {
            panic!("Hospital missing PowerConsumer");
        }

        // 5. Run Healing
        healing_system(&mut world);

        // 6. Verify No Healing
        let health = world.get::<Health>(patient).unwrap();
        assert_eq!(health.current, 50.0, "Healing should stop during blackout");
    }
}
