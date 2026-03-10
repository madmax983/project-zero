// src/layer1/tech/neural_leech_tests.rs

#[cfg(test)]
mod tests {
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::Skills;
    use crate::layer1::tech::neural_leech::*;
    use crate::layer1::GridPosition;
    use crate::layer1::StressTracker;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_neural_hub_buffs_nearby_pops() {
        let mut world = setup_world();

        let _hub = world
            .spawn((Pop, NeuralHub, GridPosition { x: 0, y: 0 }))
            .id();

        let worker = world
            .spawn((
                Pop,
                Skills::default(),
                GridPosition { x: 5, y: 0 }, // Within radius
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_neural_link_buffs_system);
        schedule.run(&mut world);

        // Worker should now have the Linked buff
        assert!(world.entity(worker).contains::<NeuralLinked>());
    }

    #[test]
    fn test_neural_hub_stress_maxes_out() {
        let mut world = setup_world();

        let hub = world.spawn((Pop, NeuralHub, StressTracker::default())).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_neural_hub_decay_system);
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(hub).unwrap();
        assert!(stress.accumulated_stress >= 99.0); // Should be pegged to max
    }

    #[test]
    fn test_hub_death_causes_cascading_breakdown() {
        let mut world = setup_world();
        world.init_resource::<Events<NeuralHubDeathEvent>>();

        let worker = world
            .spawn((
                Pop,
                NeuralLinked {
                    hub_entity: Entity::PLACEHOLDER,
                }, // Will be updated manually for test
            ))
            .id();

        // Simulate hub death event
        world.send_event(NeuralHubDeathEvent {
            hub_entity: Entity::PLACEHOLDER,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_hub_death_system);
        schedule.run(&mut world);

        // Worker should now have a severe breakdown component
        assert!(world.entity(worker).contains::<NeuralShock>());
    }
}
