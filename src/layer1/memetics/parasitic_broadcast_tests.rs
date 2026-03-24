#[cfg(test)]
mod tests {
    use crate::layer1::execution::general_work::calculate_work_amount;
    use crate::layer1::memetics::{
        process_parasitic_work_reduction, MemeticInfection,
    };
    use crate::layer1::morale::Morale;
    use crate::layer1::skills::Skills;
    use crate::layer1::DesignationType;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_parasitic_broadcast_increases_morale_and_reduces_work() {
        let mut world = World::new();
        // Setup a pop with the broadcast infection
        let pop = world
            .spawn((
                Morale {
                    value: 50.0,
                    ..Default::default()
                },
                MemeticInfection::ParasiticBroadcast, // Humming the catchy tune
                Skills::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_parasitic_work_reduction);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();

        // Massive morale boost
        assert!(morale.value > 50.0, "The song is extremely catchy");
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Entertained (Parasitic Broadcast)"));

        let infected_work_amount =
            calculate_work_amount(&world, pop, DesignationType::Mine, None, 50.0, 1.0, 1.0);

        world.entity_mut(pop).remove::<MemeticInfection>();

        let uninfected_work_amount =
            calculate_work_amount(&world, pop, DesignationType::Mine, None, 50.0, 1.0, 1.0);

        // Due to random variance, checking bounds instead of strict equality
        assert!(
            infected_work_amount < uninfected_work_amount * 0.7,
            "Work output should be roughly halved"
        );
    }
}
