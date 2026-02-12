#[cfg(test)]
mod tests {
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerSource;
    use crate::layer1::heirloom::{Heirloom, heirloom_decay_system};
    use crate::layer1::structure::{Structure, process_repair};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_heirloom_component_exists() {
        let _h = Heirloom; // Marker component
    }

    #[test]
    fn test_ancient_reactor_properties() {
        // Verify AncientReactor has high power output and structure
        let mut world = World::new();

        // Simulate spawning an AncientReactor (e.g., via a helper or directly)
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                Structure {
                    current_hp: 1000.0,
                    max_hp: 1000.0,
                },
                PowerSource { output: 50.0 }, // High output
                Heirloom,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let power = world.get::<PowerSource>(entity).unwrap();
        assert_eq!(power.output, 50.0);

        let heirloom = world.get::<Heirloom>(entity);
        assert!(heirloom.is_some());
    }

    #[test]
    fn test_heirloom_decay() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                Structure {
                    current_hp: 1000.0,
                    max_hp: 1000.0,
                },
                Heirloom,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run decay system
        let mut schedule = Schedule::default();
        schedule.add_systems(heirloom_decay_system);
        schedule.run(&mut world);

        let structure = world.get::<Structure>(entity).unwrap();
        assert!(structure.current_hp < 1000.0, "Heirloom should decay");
        assert!(structure.current_hp > 990.0, "Decay should be slow"); // Should be slow decay
    }

    #[test]
    fn test_repair_prevention_on_heirloom() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                Structure {
                    current_hp: 500.0,
                    max_hp: 1000.0,
                },
                Heirloom,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Designate for repair (mock designation entity)
        let designation = world
            .spawn((
                crate::layer1::designation::Designation {
                    designation_type: crate::layer1::designation::DesignationType::Repair,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Attempt repair
        process_repair(&mut world, designation, 10.0);

        let structure = world.get::<Structure>(entity).unwrap();
        assert_eq!(
            structure.current_hp, 500.0,
            "Heirloom should not be repaired"
        ); // No change
    }
}
