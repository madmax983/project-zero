#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::stress::StressTracker;
    use crate::layer1::social::placebo::{PlaceboProtocol, ActivePlacebo, placebo_tick_system, reveal_betrayal_system};

    #[test]
    fn test_placebo_reduces_stress_temporarily() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            StressTracker { accumulated_stress: 80.0 },
        )).id();

        // Issue "Fake Reinforcements"
        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::FakeReinforcements,
            duration: 10.0,
            stress_relief: 20.0,
            revealed: false,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(placebo_tick_system);
        schedule.run(&mut world);

        let stress_tracker = world.get::<StressTracker>(pop).unwrap();
        assert!(stress_tracker.accumulated_stress <= 60.0 + f32::EPSILON);
    }

    #[test]
    fn test_betrayal_spikes_stress_and_adds_trait() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            StressTracker { accumulated_stress: 60.0 }, // Reduced level
            Traits(std::collections::HashSet::new()),
        )).id();

        // Expire/Fail a placebo
        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::FakeReinforcements,
            duration: 0.0, // Expired
            stress_relief: 20.0,
            revealed: true, // Failed
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(reveal_betrayal_system);
        schedule.run(&mut world);

        let stress_tracker = world.get::<StressTracker>(pop).unwrap();
        // Stress should return to original (80) PLUS penalty (e.g. +20) = 100
        assert!(stress_tracker.accumulated_stress >= 90.0);

        // Check for Distrustful trait (if Trait component supports dynamic addition)
        let traits = world.get::<Traits>(pop).unwrap();
        assert!(traits.0.contains(&Trait::Distrustful));
    }
}
