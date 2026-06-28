#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::anomalies::{process_scan_system, Anomaly, AnomalyType, ScanProgress};
    use scale::layer1::designation::{Designation, DesignationType};
    use scale::layer1::execution::{work_execution_system, AtTarget, MovementTarget};
    use scale::layer1::factions::{FactionId, FactionMember, FactionState, Factions};
    use scale::layer1::map::GridPosition;
    use scale::layer1::pop::Pop;
    use scale::layer1::resources::{ColonyResources, MiningProgress};
    use scale::layer1::skills::{SkillType, Skills};
    use scale::layer1::terrain::{TerrainGrid, TerrainType};
    use scale::layer1::utility_ai::ActionType; // PopAction unused in this file

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        let mut factions = Factions::default();
        // Initialize factions map
        factions.map.insert(
            FactionId::MinersGuild,
            scale::layer1::factions::FactionData {
                name: "Miners".to_string(),
                state: FactionState::Loyal, // Will change in test
                satisfaction: 1.0,
                ..Default::default()
            },
        );
        factions.map.insert(
            FactionId::Unaligned,
            scale::layer1::factions::FactionData {
                name: "Unaligned".to_string(),
                state: FactionState::Loyal,
                satisfaction: 1.0,
                ..Default::default()
            },
        );
        world.insert_resource(factions);
        world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
        world.insert_resource(scale::layer1::structural_integrity::RoofGrid::new(10, 10));

        // Terrain setup for mining
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        // Add occupied tiles resource (needed by is_walkable)
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        // Add taboo state (needed by work_execution_system/utility)
        world.insert_resource(scale::layer1::taboo::TabooState::default());

        world
    }

    #[test]
    fn striking_miner_produces_no_progress() {
        let mut world = setup_world();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

        // 1. Set Miners Guild to Striking
        world
            .resource_mut::<Factions>()
            .map
            .get_mut(&FactionId::MinersGuild)
            .unwrap()
            .state = FactionState::Striking;

        // 2. Spawn Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                // Pre-add progress to track changes (starts at 0.0 usually)
                MiningProgress {
                    current: 0.0,
                    max: 100.0,
                },
            ))
            .id();

        // 3. Spawn Striking Miner at target
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 100.0);

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
            FactionMember {
                faction_id: Some(FactionId::MinersGuild),
            },
        ));

        // 4. Run execution system
        let mut schedule = Schedule::default();
        schedule.add_systems(work_execution_system);
        schedule.run(&mut world);

        // 5. Assert NO progress
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert!(
            progress.current <= f32::EPSILON,
            "Striking miner should make NO progress, got {}",
            progress.current
        );
    }

    #[test]
    fn striking_scientist_makes_no_scan_progress() {
        let mut world = setup_world();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtInheritedEvent>>();
        world.init_resource::<bevy_ecs::event::Events<scale::layer1::economy::debt_of_the_dead::DebtSocializedEvent>>();

        // 1. Set Unaligned to Striking
        world
            .resource_mut::<Factions>()
            .map
            .get_mut(&FactionId::Unaligned)
            .unwrap()
            .state = FactionState::Striking;

        // 2. Spawn Anomaly
        let anomaly = world
            .spawn((
                Anomaly {
                    anomaly_type: AnomalyType::Ruins,
                    reward_amount: 10.0,
                },
                GridPosition { x: 2, y: 2 },
                ScanProgress {
                    current: 0.0,
                    required: 100.0,
                },
            ))
            .id();

        // 3. Spawn Striking Scientist at target
        world.spawn((
            Pop,
            GridPosition { x: 2, y: 2 },
            MovementTarget {
                target_entity: anomaly,
                target_position: GridPosition { x: 2, y: 2 },
                for_action: ActionType::Explore,
            },
            AtTarget,
            FactionMember {
                faction_id: Some(FactionId::Unaligned),
            },
        ));

        // 4. Run scan system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_scan_system);
        schedule.run(&mut world);

        // 5. Assert NO progress
        let progress = world.get::<ScanProgress>(anomaly).unwrap();
        assert!(
            progress.current <= f32::EPSILON,
            "Striking scientist should make NO progress, got {}",
            progress.current
        );
    }
}
