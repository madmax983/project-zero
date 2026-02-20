#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::justice::{Wanted, Inmate}; // Existing components from 072
    use crate::layer1::predictive_policing::{
        Suspect, PredictiveModel, PredictionConfig,
        check_prediction_system, evaluate_pre_crime_arrest
    };
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PredictionConfig {
            threshold: 0.8, // 80% probability required
            enabled: true,
        });
        world
    }

    // 1. Prediction Logic
    #[test]
    fn test_high_stress_volatile_pop_becomes_suspect() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prediction_system);

        // Volatile pop with high stress ticks
        // Spawn PredictiveModel (Algo-Hub) so the system runs
        world.spawn(PredictiveModel);

        let mut traits = std::collections::HashSet::new();
        traits.insert(Trait::Volatile); // Assumes Trait::Volatile exists or is added

        let pop = world.spawn((
            Pop::default(),
            StressTracker { accumulated_stress: 80.0 }, // Near breakdown (threshold is 100)
            Traits(traits),
            // No Suspect component yet
        )).id();

        schedule.run(&mut world);

        // Should be marked Suspect
        let suspect = world.get::<Suspect>(pop).unwrap();
        assert!(suspect.probability >= 0.8);
        assert_eq!(suspect.predicted_crime, "Breakdown: Violence");
    }

    #[test]
    fn test_low_stress_pop_is_safe() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prediction_system);

        let pop = world.spawn((
            Pop::default(),
            StressTracker { accumulated_stress: 10.0 },
            Traits(std::collections::HashSet::new()),
        )).id();

        schedule.run(&mut world);

        assert!(world.get::<Suspect>(pop).is_none());
    }

    // 2. Warden Evaluation
    #[test]
    fn test_warden_targets_suspect() {
        let mut world = setup_world();
        // Insert ZoneGrid for evaluation context (required by evaluate_warden_action logic if reused, but here we test direct function)
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));


        let suspect = world.spawn((
            Pop::default(),
            Suspect { probability: 0.9, predicted_crime: "Arson".to_string() },
            GridPosition { x: 5, y: 5 },
        )).id();

        let warden_pos = GridPosition { x: 0, y: 0 };

        // Evaluate action
        let result = evaluate_pre_crime_arrest(&mut world, &warden_pos);

        assert!(result.is_some());
        let (score, target) = result.unwrap();
        assert!(score > 0.0);
        assert_eq!(target, suspect);
    }

    // 3. Arrest Execution
    #[test]
    fn test_pre_crime_arrest_converts_to_inmate() {
        let mut world = setup_world();
        // Insert ZoneGrid for find_jail_spot
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));
        world.resource_mut::<crate::layer1::zone::ZoneGrid>().set(2, 2, crate::layer1::zone::ZoneType::Jail);


        // Register required resources/systems from 072 if needed,
        // or mock the arrest execution function for unit testing logic.
        // Here we test a specific pre-crime arrest handler.

        let suspect = world.spawn((
            Pop::default(),
            Suspect { probability: 0.95, predicted_crime: "Murder".to_string() },
            GridPosition { x: 1, y: 1 },
        )).id();

        let warden = world.spawn((
            Pop::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Execute arrest
        crate::layer1::predictive_policing::execute_pre_crime_arrest(&mut world, warden, suspect);

        // Should be Inmate (Protective Custody)
        let inmate = world.get::<Inmate>(suspect).unwrap();
        assert!(inmate.sentence_ticks > 0);
        // Should NOT be Wanted (they haven't done it yet)
        assert!(world.get::<Wanted>(suspect).is_none());
        // Suspect marker removed
        assert!(world.get::<Suspect>(suspect).is_none());
    }
}
