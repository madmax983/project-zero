#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::ghost_code::{DataResidue, GhostCode, GhostTrait, residue_system, ghost_infection_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::events::{BuildingCompletedEvent, BuildingRemovedEvent};

    #[test]
    fn test_deconstruction_leaves_residue() {
        let mut world = World::new();
        world.insert_resource(Events::<BuildingRemovedEvent>::default());
        let mut events = world.resource_mut::<Events<BuildingRemovedEvent>>();

        let pos = GridPosition { x: 10, y: 10 };
        // Simulate deconstruction event
        events.send(BuildingRemovedEvent {
            entity: Entity::PLACEHOLDER, // Mock entity
            position: pos,
            building_type: BuildingType::Tower,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(residue_system);
        schedule.run(&mut world);

        // Check for Residue entity at pos
        let mut found = false;
        for (residue, p) in world.query::<(&DataResidue, &GridPosition)>().iter(&world) {
            if *p == pos {
                assert_eq!(residue.source_type, BuildingType::Tower);
                found = true;
            }
        }
        assert!(found, "DataResidue should be spawned at deconstruction site");
    }

    #[test]
    fn test_construction_inherits_ghost_code() {
        let mut world = World::new();
        world.insert_resource(Events::<BuildingCompletedEvent>::default());
        let pos = GridPosition { x: 5, y: 5 };

        // 1. Spawn Residue (from old MedicalBay aka Hospital)
        let residue = world.spawn((
            DataResidue { source_type: BuildingType::Hospital },
            pos
        )).id();

        // 2. Build New Building (Tower)
        let new_building = world.spawn((
            Building { building_type: BuildingType::Tower, ..Default::default() },
            pos
        )).id();

        // 3. Trigger Completion Event
        let mut events = world.resource_mut::<Events<BuildingCompletedEvent>>();
        events.send(BuildingCompletedEvent { entity: new_building });

        // 4. Run Infection System
        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_infection_system);
        schedule.run(&mut world);

        // 5. Assert GhostCode presence
        let ghost = world.get::<GhostCode>(new_building).expect("Building should acquire GhostCode");
        assert_eq!(ghost.traits.len(), 1);

        let trait_val = &ghost.traits[0];
        // Hospital residue should cause GhostProtocol("Triage")
        match trait_val {
            GhostTrait::GhostProtocol(val) => assert_eq!(val, "Triage"),
            _ => panic!("Expected GhostProtocol trait"),
        }

        // 6. Assert Residue Consumed
        assert!(!world.entities().contains(residue), "Residue should be consumed");
    }

    #[test]
    fn test_purge_action_removes_residue() {
        let mut world = World::new();
        let pos = GridPosition { x: 0, y: 0 };
        let residue_entity = world.spawn((
            DataResidue { source_type: BuildingType::Wall },
            pos
        )).id();

        // Perform Purge Action
        crate::layer1::tech::ghost_code::perform_purge(&mut world, residue_entity);

        assert!(!world.entities().contains(residue_entity), "Residue should be despawned after purge");
    }
}
