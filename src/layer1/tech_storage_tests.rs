#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::schedule::Schedule;
    use crate::layer1::tech::{Tech, TechState, TechStatus, DataStorage};
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_tech_has_storage_cost() {
        assert_eq!(Tech::Masonry.storage_cost(), 5.0);
        assert_eq!(Tech::Astronomy.storage_cost(), 20.0);
    }

    #[test]
    fn test_tech_state_tracks_capacity() {
        let mut state = TechState::default();
        assert_eq!(state.total_capacity, 0.0);
        assert_eq!(state.used_capacity, 0.0);

        // Manual capacity adjustment for test
        state.total_capacity = 100.0;
        assert_eq!(state.total_capacity, 100.0);
    }

    #[test]
    fn test_unlocking_tech_increases_usage() {
        let mut state = TechState::default();
        state.total_capacity = 100.0;

        let success = state.try_unlock(Tech::Masonry);

        assert!(success);
        assert!(state.is_unlocked(Tech::Masonry));
        assert_eq!(state.used_capacity, Tech::Masonry.storage_cost());
    }

    #[test]
    fn test_cannot_unlock_if_full() {
        let mut state = TechState::default();
        state.total_capacity = 4.0; // Not enough for Masonry (5.0)

        // Should fail
        let success = state.try_unlock(Tech::Masonry);

        assert!(!success);
        assert!(!state.is_unlocked(Tech::Masonry));
    }

    #[test]
    fn test_capacity_drop_triggers_corruption() {
        let mut state = TechState::default();
        state.total_capacity = 10.0;
        state.unlock(Tech::Masonry); // Cost 5.0, Unlock forces active

        // Disaster! Capacity drops
        state.total_capacity = 0.0;

        // System update should mark tech as Corrupted
        state.update_corruption();

        // Expectation: TechStatus::Corrupted
        match state.techs.get(&Tech::Masonry) {
             Some(TechStatus::Corrupted) => {},
             _ => panic!("Expected Corrupted, got {:?}", state.techs.get(&Tech::Masonry)),
        }
    }

    #[test]
    fn test_corrupted_tech_blocks_access() {
        let mut state = TechState::default();
        state.force_unlock(Tech::Masonry, TechStatus::Corrupted);

        assert!(!state.is_active(Tech::Masonry));
    }

    #[test]
    fn test_server_bank_increases_capacity() {
        let mut world = World::new();
        world.insert_resource(TechState::default());

        // Spawn ServerBank
        let _id = world.spawn((
            Building { building_type: BuildingType::ServerBank },
            DataStorage { capacity: 50.0 },
            crate::layer1::energy::PowerConsumer { active: true, ..Default::default() },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::tech::update_tech_capacity_system);
        schedule.run(&mut world);

        let state = world.resource::<TechState>();
        assert_eq!(state.total_capacity, 50.0);
    }

    #[test]
    fn test_passive_storage_works() {
        // Test that storage without PowerConsumer works (Lander case)
        let mut world = World::new();
        world.insert_resource(TechState::default());

        // Spawn Passive Storage (e.g. Lander)
        world.spawn((
            Building { building_type: BuildingType::Lander },
            DataStorage { capacity: 10.0 },
            // No PowerConsumer
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::tech::update_tech_capacity_system);
        schedule.run(&mut world);

        let state = world.resource::<TechState>();
        assert_eq!(state.total_capacity, 10.0);
    }

    #[test]
    fn test_unlocking_existing_corrupted_tech_costs_nothing() {
        let mut world = World::new();
        let mut state = TechState::default();
        // Force unlock as corrupted
        state.force_unlock(Tech::Masonry, TechStatus::Corrupted);
        world.insert_resource(state);

        world.insert_resource(crate::layer1::resources::ColonyResources {
            knowledge: 100.0,
            ..Default::default()
        });
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Unlock again
        let success = crate::layer1::tech::unlock_tech(&mut world, Tech::Masonry);

        assert!(success);

        // Should NOT deduct cost
        let res = world.resource::<crate::layer1::resources::ColonyResources>();
        assert_eq!(res.knowledge, 100.0);
    }
}
