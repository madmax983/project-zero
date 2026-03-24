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
}
