#[cfg(test)]
mod tests {
    use super::super::ghost_code::{
        apply_ghost_traits_system, ghost_infection_system, perform_purge, purge_execution_system,
        residue_system, DataResidue, GhostCode, GhostEffectApplied, GhostTrait,
    };
    use crate::layer1::architecture::turret::Turret;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::combat::AttackProperties;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::core::events::{BuildingCompletedEvent, BuildingRemovedEvent};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ResourceType;
    use crate::layer1::utility_types::{ActionType, PopAction, StartPlan};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_data_residue_traits() {
        let mut residue = DataResidue {
            source_type: BuildingType::Tower,
        };
        assert_eq!(residue.get_ghost_trait(), GhostTrait::LegacyTargeting);

        residue.source_type = BuildingType::TrashCannon;
        assert_eq!(residue.get_ghost_trait(), GhostTrait::LegacyTargeting);

        residue.source_type = BuildingType::Hospital;
        assert_eq!(
            residue.get_ghost_trait(),
            GhostTrait::GhostProtocol("Triage".into())
        );

        residue.source_type = BuildingType::CryoPod;
        assert_eq!(
            residue.get_ghost_trait(),
            GhostTrait::GhostProtocol("Triage".into())
        );

        residue.source_type = BuildingType::Housing;
        assert_eq!(residue.get_ghost_trait(), GhostTrait::PhantomPower);
    }

    #[test]
    fn test_residue_system() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingRemovedEvent>>();

        world
            .resource_mut::<Events<BuildingRemovedEvent>>()
            .send(BuildingRemovedEvent {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Housing,
                position: GridPosition { x: 5, y: 5 },
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(residue_system);
        schedule.run(&mut world);

        let mut query = world.query::<(&DataResidue, &GridPosition)>();
        let residues: Vec<_> = query.iter(&world).collect();

        assert_eq!(residues.len(), 1);
        assert_eq!(residues[0].0.source_type, BuildingType::Housing);
        assert_eq!(*residues[0].1, GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_ghost_infection_system() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingCompletedEvent>>();

        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: building_entity,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_infection_system);
        schedule.run(&mut world);

        // Residue should be deleted
        assert!(world.get::<DataResidue>(residue_entity).is_none());

        // Building should be infected
        let ghost_code = world.get::<GhostCode>(building_entity).unwrap();
        assert_eq!(ghost_code.traits, vec![GhostTrait::PhantomPower]);
    }

    #[test]
    fn test_ghost_infection_system_no_residue() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingCompletedEvent>>();

        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: building_entity,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_infection_system);
        schedule.run(&mut world);

        // Building should not be infected
        assert!(world.get::<GhostCode>(building_entity).is_none());
    }

    #[test]
    fn test_ghost_infection_system_wrong_position() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingCompletedEvent>>();

        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Housing,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: building_entity,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_infection_system);
        schedule.run(&mut world);

        // Residue should NOT be deleted
        assert!(world.get::<DataResidue>(residue_entity).is_some());

        // Building should NOT be infected
        assert!(world.get::<GhostCode>(building_entity).is_none());
    }

    #[test]
    fn test_apply_ghost_traits_system_power_drain() {
        let mut world = World::new();

        let entity = world
            .spawn((
                GhostCode {
                    traits: vec![GhostTrait::PowerDrain],
                },
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ghost_traits_system);
        schedule.run(&mut world);

        let power = world.get::<PowerConsumer>(entity).unwrap();
        assert!((power.demand - 11.0).abs() < f32::EPSILON);
        assert!(world.get::<GhostEffectApplied>(entity).is_some());
    }

    #[test]
    fn test_apply_ghost_traits_system_legacy_targeting() {
        let mut world = World::new();

        let entity = world
            .spawn((
                GhostCode {
                    traits: vec![GhostTrait::LegacyTargeting],
                },
                Turret {
                    attack: AttackProperties {
                        range: 10.0,
                        damage: 5.0,
                        cooldown: 1,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ghost_traits_system);
        schedule.run(&mut world);

        let turret = world.get::<Turret>(entity).unwrap();
        assert!((turret.attack.range - 15.0).abs() < f32::EPSILON);
        assert!((turret.attack.damage - 6.0).abs() < f32::EPSILON);
        assert!(world.get::<GhostEffectApplied>(entity).is_some());
    }

    #[test]
    fn test_apply_ghost_traits_system_ignored_traits() {
        let mut world = World::new();

        let entity = world
            .spawn((
                GhostCode {
                    traits: vec![GhostTrait::GhostProtocol("Ignored".into())],
                },
                PowerConsumer {
                    demand: 10.0,
                    active: true,
                },
                Turret {
                    attack: AttackProperties {
                        range: 10.0,
                        damage: 5.0,
                        cooldown: 1,
                        accuracy: 1.0,
                    },
                    ammo_cost: 1.0,
                    ammo_type: ResourceType::Waste,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_ghost_traits_system);
        schedule.run(&mut world);

        // Assert nothing changed and it wasn't applied
        let power = world.get::<PowerConsumer>(entity).unwrap();
        assert!((power.demand - 10.0).abs() < f32::EPSILON);
        let turret = world.get::<Turret>(entity).unwrap();
        assert!((turret.attack.range - 10.0).abs() < f32::EPSILON);
        assert!(world.get::<GhostEffectApplied>(entity).is_none());
    }

    #[test]
    fn test_perform_purge() {
        let mut world = World::new();
        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        perform_purge(&mut world, residue_entity);
        assert!(world.get::<DataResidue>(residue_entity).is_none());
    }

    #[test]
    fn test_perform_purge_wrong_entity() {
        let mut world = World::new();
        let entity = world.spawn(()).id();

        perform_purge(&mut world, entity);
        // Should not panic or despawn since it's not a DataResidue
        assert!(world.get_entity(entity).is_ok());
    }

    #[test]
    fn test_purge_execution_system() {
        let mut world = World::new();

        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::PurgeResidue,
                    ticks_committed: 0,
                    current_utility: 0.0,
                },
                GridPosition { x: 5, y: 5 },
                StartPlan {
                    target: Some(residue_entity),
                    action: ActionType::PurgeResidue,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        // Residue should be despawned
        assert!(world.get_entity(residue_entity).is_err()); // or is_none()

        // Pop Action should be reset
        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
        assert!(world.get::<StartPlan>(pop_entity).is_none());
    }

    #[test]
    fn test_purge_execution_system_target_gone() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::PurgeResidue,
                    ticks_committed: 0,
                    current_utility: 0.0,
                },
                GridPosition { x: 5, y: 5 },
                StartPlan {
                    target: Some(Entity::from_raw(999)),
                    action: ActionType::PurgeResidue,
                }, // Non-existent target
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
        assert!(world.get::<StartPlan>(pop_entity).is_none());
    }

    #[test]
    fn test_purge_execution_system_no_target() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::PurgeResidue,
                    ticks_committed: 0,
                    current_utility: 0.0,
                },
                GridPosition { x: 5, y: 5 },
                StartPlan {
                    target: None,
                    action: ActionType::PurgeResidue,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
        assert!(world.get::<StartPlan>(pop_entity).is_none());
    }

    #[test]
    fn test_purge_execution_system_no_plan() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::PurgeResidue,
                    ticks_committed: 0,
                    current_utility: 0.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Idle
        );
    }

    #[test]
    fn test_purge_execution_system_different_action() {
        let mut world = World::new();

        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::Work,
                    ticks_committed: 0,
                    current_utility: 0.0,
                },
                GridPosition { x: 5, y: 5 },
                StartPlan {
                    target: Some(residue_entity),
                    action: ActionType::Work,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        // Residue should NOT be despawned
        assert!(world.get::<DataResidue>(residue_entity).is_some());

        // Pop Action should NOT be reset
        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::Work
        );
    }

    #[test]
    fn test_purge_execution_system_far_from_target() {
        let mut world = World::new();

        let residue_entity = world
            .spawn((
                DataResidue {
                    source_type: BuildingType::Housing,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let pop_entity = world
            .spawn((
                PopAction {
                    current: ActionType::PurgeResidue,
                    ticks_committed: 0,
                    current_utility: 0.0,
                },
                GridPosition { x: 5, y: 5 },
                StartPlan {
                    target: Some(residue_entity),
                    action: ActionType::PurgeResidue,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(purge_execution_system);
        schedule.run(&mut world);

        // Residue should NOT be despawned (too far)
        assert!(world.get::<DataResidue>(residue_entity).is_some());

        // Pop Action should NOT be reset (still moving)
        assert_eq!(
            world.get::<PopAction>(pop_entity).unwrap().current,
            ActionType::PurgeResidue
        );
        assert!(world.get::<StartPlan>(pop_entity).is_some());
    }
}
