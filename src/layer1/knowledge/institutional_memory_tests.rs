#[cfg(test)]
mod tests {
    use crate::layer1::execution::{AtTarget, MovementTarget};
    use crate::layer1::institutional_memory::{
        manual_aura_system, produce_manual_system, InstitutionalMemoryConfig, Manual,
    };
    use crate::layer1::items::Item;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::utility_ai::ActionType;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_produce_manual_high_skill() {
        let mut world = World::new();

        // Guarantee production for test
        world.insert_resource(InstitutionalMemoryConfig {
            production_chance: 1.0,
        });

        // Spawn high-skill pop working
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 2500.0); // Level 5
        let _pop = world
            .spawn((
                Pop,
                skills,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(0),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget, // Must be actively working
            ))
            .id();

        // Run system multiple times to trigger chance (or mock RNG)
        let mut schedule = Schedule::default();
        schedule.add_systems(produce_manual_system);

        schedule.run(&mut world);

        // Check if Manual item spawned at position
        let mut manual_query = world.query::<(&Manual, &GridPosition)>();
        let mut found = false;
        for (manual, pos) in manual_query.iter(&world) {
            if pos.x == 5 && pos.y == 5 && manual.skill_type == SkillType::Mining {
                found = true;
                break;
            }
        }
        assert!(found, "Should spawn a manual for high skill worker");
    }

    #[test]
    fn test_produce_manual_low_skill_fails() {
        let mut world = World::new();
        // Spawn low-skill pop working
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 0.0); // Level 0
        world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: Entity::from_raw(0),
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(produce_manual_system);

        // Run many times to be sure
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let count = world.query::<&Manual>().iter(&world).count();
        assert_eq!(count, 0, "Low skill pop should not produce manual");
    }

    #[test]
    fn test_manual_aura_boosts_xp() {
        let mut world = World::new();

        // Spawn Manual on ground
        world.spawn((
            Item::default(),
            Manual {
                skill_type: SkillType::Mining,
                xp_multiplier: 1.5,
                durability: 100.0,
                max_durability: 100.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker nearby (Level 0)
        let worker = world
            .spawn((
                Pop,
                Skills::default(),
                GridPosition { x: 6, y: 5 }, // Adjacent
                MovementTarget {
                    target_entity: Entity::from_raw(0),
                    target_position: GridPosition { x: 6, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        // Run aura system
        let mut schedule = Schedule::default();
        schedule.add_systems(manual_aura_system);
        schedule.run(&mut world);

        // Verify XP gain
        let skills = world.get::<Skills>(worker).unwrap();
        assert!(
            skills.get_xp(SkillType::Mining) > 0.0,
            "Should gain passive XP from Manual aura"
        );
    }

    #[test]
    fn test_manual_degradation() {
        let mut world = World::new();
        let manual = world
            .spawn((
                Manual {
                    skill_type: SkillType::Mining,
                    xp_multiplier: 1.5,
                    durability: 0.05, // Very low durability
                    max_durability: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Spawn worker to trigger usage
        world.spawn((
            Pop,
            Skills::default(),
            GridPosition { x: 6, y: 5 },
            MovementTarget {
                target_entity: Entity::from_raw(0),
                target_position: GridPosition { x: 6, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(manual_aura_system);
        schedule.run(&mut world); // Tick 1: Durability decreases

        // Check if manual is destroyed
        // Assuming decay is > 0.05 per use
        assert!(
            world.get_entity(manual).is_err(),
            "Manual should be destroyed when durability hits 0"
        );
    }

    #[test]
    fn test_produce_manual_with_designation() {
        use crate::layer1::designation::{Designation, DesignationType};

        let mut world = World::new();
        world.insert_resource(InstitutionalMemoryConfig {
            production_chance: 1.0,
        });

        // Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Spawn Pop with Mining skill working on designation
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 2500.0);
        world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(produce_manual_system);
        schedule.run(&mut world);

        let count = world.query::<&Manual>().iter(&world).count();
        assert_eq!(
            count, 1,
            "Should produce manual when working on designation"
        );

        let manual = world.query::<&Manual>().single(&world);
        assert_eq!(manual.skill_type, SkillType::Mining);
    }

    #[test]
    fn test_manual_aura_strict_match() {
        use crate::layer1::designation::{Designation, DesignationType};
        let mut world = World::new();

        // Manual for Mining
        world.spawn((
            Item::default(),
            Manual {
                skill_type: SkillType::Mining,
                xp_multiplier: 1.5,
                durability: 10.0,
                max_durability: 10.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Designation for Forestry (Mismatch)
        let forestry_designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        // Worker doing Forestry
        let worker = world
            .spawn((
                Pop,
                Skills::default(),
                GridPosition { x: 6, y: 5 },
                MovementTarget {
                    target_entity: forestry_designation,
                    target_position: GridPosition { x: 6, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(manual_aura_system);
        schedule.run(&mut world);

        // Should NOT gain XP in Mining because task is Forestry
        let skills = world.get::<Skills>(worker).unwrap();
        assert_eq!(
            skills.get_xp(SkillType::Mining),
            0.0,
            "Should not gain XP for mismatched task"
        );
    }

    #[test]
    fn test_manual_aura_distance_limit() {
        let mut world = World::new();
        // Manual at 0,0
        world.spawn((
            Item::default(),
            Manual {
                skill_type: SkillType::Mining,
                xp_multiplier: 1.5,
                durability: 10.0,
                max_durability: 10.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Worker at 6,0 (Distance 6)
        let worker = world
            .spawn((
                Pop,
                Skills::default(),
                GridPosition { x: 6, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(0),
                    target_position: GridPosition { x: 6, y: 0 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(manual_aura_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(worker).unwrap();
        assert_eq!(
            skills.get_xp(SkillType::Mining),
            0.0,
            "Should not gain XP if too far"
        );
    }
}
