use crate::layer1::actions::AssignedTo;
use crate::layer1::jobs::AssignmentType;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component marker for Observatory buildings.
#[derive(Component, Default)]
pub struct Observatory;

/// Processes logic for Pops assigned to Observatories.
///
/// 1. Generates 0.02 Knowledge per pop per tick.
/// 2. Has a 1% chance per tick to trigger "The Overview Effect" (Mood Modifier).
///    - 50% "Cosmic Inspiration" (+0.15)
///    - 50% "Existential Dread" (-0.10)
pub fn process_observe_system(
    mut pops: Query<(Entity, &AssignedTo, &mut Morale)>,
    observatories: Query<&Observatory>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for (_entity, assignment, mut morale) in &mut pops {
        if assignment.assignment_type == AssignmentType::ObservatoryWorker
            && observatories.get(assignment.entity).is_ok()
        {
            // 1. Generate Knowledge
            resources.knowledge += 0.02;
            resources.knowledge = resources.knowledge.clamp(0.0, resources.max_knowledge);

            // 2. Chance for Overview Effect (1% per tick)
            if rng.gen_bool(0.01) {
                // 50/50 split for now
                if rng.gen_bool(0.5) {
                    morale.add_modifier(MoodModifier {
                        label: "Cosmic Inspiration".to_string(),
                        value: 0.15,
                        duration: 500,
                    });
                } else {
                    morale.add_modifier(MoodModifier {
                        label: "Existential Dread".to_string(),
                        value: -0.10,
                        duration: 500,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Observatory, process_observe_system};
    use crate::layer1::actions::AssignedTo;
    use crate::layer1::building::BuildingType;
    use crate::layer1::jobs::AssignmentType;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::Tech;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_astronomy_tech_exists() {
        // Just verifying variant exists
        let _ = Tech::Astronomy;
        assert_eq!(Tech::Astronomy.cost(), 50.0); // Expensive late-game tech
    }

    #[test]
    fn test_observatory_requires_astronomy() {
        assert_eq!(
            BuildingType::Observatory.required_tech(),
            Some(Tech::Astronomy)
        );
    }

    #[test]
    fn test_observe_action_generates_knowledge() {
        let mut world = World::new();

        // Setup Resources
        let mut res = ColonyResources::default();
        res.knowledge = 0.0;
        res.max_knowledge = 100.0;
        world.insert_resource(res);

        // Setup Observatory
        let observatory = world.spawn(Observatory).id();

        // Setup Pop working there
        world.spawn((
            Pop,
            AssignedTo {
                entity: observatory,
                assignment_type: AssignmentType::ObservatoryWorker,
            },
            Morale::default(), // Needed for query
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);
        schedule.run(&mut world);

        // Check Knowledge Gain
        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0);
    }

    #[test]
    fn test_observe_action_applies_mood_modifier() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let observatory = world.spawn(Observatory).id();
        let pop = world
            .spawn((
                Pop,
                Morale::default(),
                AssignedTo {
                    entity: observatory,
                    assignment_type: AssignmentType::ObservatoryWorker,
                },
            ))
            .id();

        // Run system multiple times to ensure probability triggers
        let mut schedule = Schedule::default();
        schedule.add_systems(process_observe_system);

        // Mock RNG or run enough times to trigger effect
        // 1% chance. 500 attempts => 99.3% chance of at least one hit.
        for _ in 0..500 {
            schedule.run(&mut world);
        }

        let morale = world.get::<Morale>(pop).unwrap();
        let has_inspired = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Cosmic Inspiration");
        let has_dread = morale
            .modifiers
            .iter()
            .any(|m| m.label == "Existential Dread");

        assert!(
            has_inspired || has_dread,
            "Should have triggered a mood modifier"
        );
    }
}
