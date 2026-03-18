#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::flora::{process_flora_clearing, Flora};
    use scale::layer1::map::GridPosition;
    use scale::layer1::nature::biosphere_empathy::{
        handle_flora_damage_empathy_system, FloraDamagedEvent,
    };
    use scale::layer1::pop::Pop;
    use scale::layer1::stress::StressTracker;
    use scale::layer1::systems::update_event_buffer;
    use scale::layer1::traits::{Trait, Traits};

    #[test]
    fn test_flora_clearing_triggers_empathy_damage() {
        let mut world = World::new();

        world.init_resource::<Events<FloraDamagedEvent>>();

        let pop = world
            .spawn((
                Pop,
                Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        let flora_entity = world
            .spawn((Flora::default(), GridPosition { x: 5, y: 5 }))
            .id();

        let designation = world.spawn((GridPosition { x: 5, y: 5 },)).id();

        process_flora_clearing(&mut world, designation, 150.0);

        assert!(
            world.get_entity(flora_entity).is_err(),
            "Flora should be despawned"
        );

        // Run systems
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                update_event_buffer::<FloraDamagedEvent>,
                handle_flora_damage_empathy_system,
            )
                .chain(),
        );
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap().accumulated_stress;

        assert!(stress > 10.0, "Pop stress should have increased due to flora clearing damage (was {}, expected > 10.0)", stress);
    }
}
