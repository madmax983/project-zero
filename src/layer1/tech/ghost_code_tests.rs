#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::events::{BuildingCompletedEvent, BuildingRemovedEvent};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::tech::ghost_code::{
        apply_ghost_traits_system, ghost_infection_system, purge_execution_system, residue_system,
        DataResidue, GhostCode, GhostEffectApplied, GhostTrait,
    };
    use crate::layer1::turret::Turret;
    use crate::layer1::utility_types::StartPlan;
    use crate::layer1::utility_types::{ActionType, PopAction};
    use bevy_ecs::prelude::*; // Needed to add Turret component for LegacyTargeting test

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
        assert!(
            found,
            "DataResidue should be spawned at deconstruction site"
        );
    }

    #[test]
    fn test_construction_inherits_ghost_code() {
        let mut world = World::new();
        world.insert_resource(Events::<BuildingCompletedEvent>::default());
        let pos = GridPosition { x: 5, y: 5 };

        // 1. Spawn Residue (from old MedicalBay aka Hospital)
        let residue = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Hospital,
                },
                pos,
            ))
            .id();

        // 2. Build New Building (Tower)
        let new_building = world
            .spawn((
                Building {
                    building_type: BuildingType::Tower,
                    ..Default::default()
                },
                pos,
            ))
            .id();

        // 3. Trigger Completion Event
        let mut events = world.resource_mut::<Events<BuildingCompletedEvent>>();
        events.send(BuildingCompletedEvent {
            entity: new_building,
        });

        // 4. Run Infection System
        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_infection_system);
        schedule.run(&mut world);

        // 5. Assert GhostCode presence
        let ghost = world
            .get::<GhostCode>(new_building)
            .expect("Building should acquire GhostCode");
        assert_eq!(ghost.traits.len(), 1);

        let trait_val = &ghost.traits[0];
        // Hospital residue should cause GhostProtocol("Triage")
        match trait_val {
            GhostTrait::GhostProtocol(val) => assert_eq!(val, "Triage"),
            _ => panic!("Expected GhostProtocol trait"),
        }

        // 6. Assert Residue Consumed
        assert!(
            !world.entities().contains(residue),
            "Residue should be consumed"
        );
    }

    #[test]
    fn test_purge_action_removes_residue() {
        let mut world = World::new();
        let pos = GridPosition { x: 0, y: 0 };
        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Wall,
                },
                pos,
            ))
            .id();

        // Perform Purge Action
        crate::layer1::tech::ghost_code::perform_purge(&mut world, residue_entity);

        assert!(
            !world.entities().contains(residue_entity),
            "Residue should be despawned after purge"
        );
    }

    #[test]
    fn test_ghost_trait_power_drain_once() {
        let mut world = World::new();

        // Spawn a building with PowerDrain ghost trait
        let building = world
            .spawn((
                GhostCode {
                    traits: vec![GhostTrait::PowerDrain],
                },
                PowerConsumer {
                    active: true,
                    demand: 10.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run apply_ghost_traits_system MULTIPLE TIMES
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ghost_traits_system);

        // First Run
        schedule.run(&mut world);
        let power = world.get::<PowerConsumer>(building).unwrap();
        assert!(
            (power.demand - 11.0).abs() < f32::EPSILON,
            "First run should apply 10% drain (10.0 -> 11.0)"
        );
        assert!(
            world.get::<GhostEffectApplied>(building).is_some(),
            "Should have Applied marker"
        );

        // Second Run
        schedule.run(&mut world);
        let power_after = world.get::<PowerConsumer>(building).unwrap();
        assert!(
            (power_after.demand - 11.0).abs() < f32::EPSILON,
            "Second run should NOT apply drain again"
        );
    }

    #[test]
    fn test_ghost_trait_legacy_targeting_tag() {
        let mut world = World::new();

        let building = world
            .spawn((
                GhostCode {
                    traits: vec![GhostTrait::LegacyTargeting],
                },
                // Need Turret component for this to be applied, based on logic
                Turret {
                    attack: crate::layer1::combat::AttackProperties {
                        damage: 10.0,
                        range: 10.0,
                        cooldown: 10,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: crate::layer1::resources::ResourceType::Waste,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ghost_traits_system);
        schedule.run(&mut world);

        assert!(
            world.get::<GhostEffectApplied>(building).is_some(),
            "LegacyTargeting should also mark as Applied"
        );
    }

    #[test]
    fn test_purge_execution_system() {
        let mut world = World::new();
        let pos = GridPosition { x: 2, y: 2 };

        // Spawn Residue
        let residue = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Wall,
                },
                pos,
            ))
            .id();

        // Spawn Pop assigned to Purge
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 2 }, // At location
                PopAction {
                    current: ActionType::PurgeResidue,
                    current_utility: 0.8,
                    ticks_committed: 10,
                },
                crate::layer1::utility_types::StartPlan {
                    action: ActionType::PurgeResidue,
                    target: Some(residue),
                },
            ))
            .id();

        // Run purge execution system
        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        // Assert Residue is gone
        assert!(
            !world.entities().contains(residue),
            "Residue should be purged by pop"
        );

        // Assert Pop is back to Idle (or at least action completed)
        let action = world.get::<PopAction>(pop).unwrap();
        // The system might reset action to Idle or just finish the task.
        // Usually systems reset to Idle when done.
        assert_eq!(
            action.current,
            ActionType::Idle,
            "Pop should be idle after purge"
        );
    }
}
