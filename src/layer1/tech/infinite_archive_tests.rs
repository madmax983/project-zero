#[cfg(test)]
mod tests {
    use crate::layer1::tech::infinite_archive::{purge_tech, update_efficiency_system, Archive};
    use crate::layer1::tech::{Tech, TechState, TechStatus};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_data_accumulation_reduces_efficiency() {
        let mut world = World::new();

        // Setup Archive
        world.insert_resource(Archive::default());

        // Setup a Tech Tree with some unlocked techs
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state
            .techs
            .insert(Tech::VoidWhispers, TechStatus::Active);
        tech_state.update_corruption(); // Update used_capacity
        world.insert_resource(tech_state);

        // Run system to update usage and efficiency
        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        // Expect used to be 50.0
        assert!(
            (archive.used - 50.0).abs() < f32::EPSILON,
            "Used capacity should be 50.0, got {}",
            archive.used
        );

        // Efficiency should be affected.
        // Formula: 1.0 - (usage / capacity * 0.5) = 1.0 - (0.5 * 0.5) = 0.75
        assert!(
            (archive.efficiency_multiplier - 0.75).abs() < f32::EPSILON,
            "Efficiency should be 0.75, got {}",
            archive.efficiency_multiplier
        );
    }

    #[test]
    fn test_overcapacity_halts_research() {
        let mut world = World::new();
        world.insert_resource(Archive::default());

        // Mock tech tree with MORE than capacity
        let tech_state = TechState {
            total_capacity: 100.0,
            used_capacity: 200.0,
            ..Default::default()
        };
        world.insert_resource(tech_state);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        // Over capacity -> Efficiency 0.0
        assert_eq!(
            archive.efficiency_multiplier, 0.0,
            "Efficiency should be 0.0 on massive overload"
        );
    }

    #[test]
    fn test_purge_tech_restores_efficiency() {
        let mut world = World::new();
        world.insert_resource(crate::shared::log::MessageLog::default());
        let mut tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        tech_state
            .techs
            .insert(Tech::VoidWhispers, TechStatus::Active); // 50.0 TB
        tech_state.update_corruption();
        world.insert_resource(tech_state);

        world.insert_resource(Archive::default());

        // Verify initial state (after update)
        let mut schedule = Schedule::default();
        schedule.add_systems(update_efficiency_system);
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert!((archive.used - 50.0).abs() < f32::EPSILON);
        assert!((archive.efficiency_multiplier - 0.75).abs() < f32::EPSILON);

        // Perform Purge
        purge_tech(&mut world, "Void Whispers");

        // Run system again
        schedule.run(&mut world);

        let archive = world.resource::<Archive>();
        assert_eq!(archive.used, 0.0, "Used should be 0.0 after purge");
        assert_eq!(
            archive.efficiency_multiplier, 1.0,
            "Efficiency should be 1.0 after purge"
        );

        let state = world.resource::<TechState>();
        // Tech should be gone
        assert!(
            !state.is_active(Tech::VoidWhispers),
            "Tech should not be active after purge"
        );
        assert!(
            !state.techs.contains_key(&Tech::VoidWhispers),
            "Tech should be removed from map"
        );
    }

    use crate::layer1::tech::infinite_archive::{
        delete_tech_system, research_tick_system, update_storage_system, DataStorage,
        DeleteTechDataEvent, ResearchProgress, ServerRack, TechId, TotalData, UnlockedTechs,
    };
    use bevy_app::App;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_systems(
            bevy_app::Update,
            (
                research_tick_system,
                update_storage_system,
                delete_tech_system,
            ),
        );
        app.add_event::<DeleteTechDataEvent>();
        app
    }

    #[test]
    fn test_data_generation_consumes_storage() {
        let mut app = setup_test_app();
        let server_rack = app.world_mut().spawn(ServerRack { capacity: 100 }).id();
        app.world_mut().insert_resource(ResearchProgress {
            current_tech: TechId::Laser,
            progress: 0.0,
        });

        app.update();
        // Running it twice because the first time it inserts the component
        app.update();

        let storage = app.world().get::<DataStorage>(server_rack).unwrap();
        assert!(storage.used > 0);
    }

    #[test]
    fn test_search_time_increases_with_data_volume() {
        let mut app_empty = setup_test_app();
        app_empty.world_mut().insert_resource(TotalData(0));
        app_empty.world_mut().insert_resource(ResearchProgress {
            current_tech: TechId::Laser,
            progress: 0.0,
        });

        let mut app_full = setup_test_app();
        app_full.world_mut().insert_resource(TotalData(1000));
        app_full.world_mut().insert_resource(ResearchProgress {
            current_tech: TechId::Laser,
            progress: 0.0,
        });

        app_empty.update();
        app_full.update();

        let empty_progress = app_empty.world().resource::<ResearchProgress>().progress;
        let full_progress = app_full.world().resource::<ResearchProgress>().progress;

        assert!(empty_progress > full_progress);
    }

    #[test]
    fn test_deleting_data_forgets_tech_and_restores_speed() {
        let mut app = setup_test_app();
        app.world_mut().insert_resource(TotalData(1000));
        app.world_mut()
            .insert_resource(UnlockedTechs(vec![TechId::SteamEngine]));

        app.world_mut()
            .send_event(DeleteTechDataEvent(TechId::SteamEngine));
        app.update();

        let unlocked = app.world().resource::<UnlockedTechs>();
        assert!(!unlocked.0.contains(&TechId::SteamEngine));
        assert!(app.world().resource::<TotalData>().0 < 1000);
    }
}
