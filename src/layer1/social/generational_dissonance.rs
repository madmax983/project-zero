use crate::layer1::pop::Job;
use crate::layer1::social::morale::MoodModifier;
use crate::layer1::social::morale::Morale;
use crate::layer1::social::old_guard::Generation;
use bevy_ecs::prelude::*;

/// ⚡ Bolt Optimization: Eliminated intermediate Vec allocation and replaced default SipHash with AHash.
pub fn process_generational_dissonance_system(
    mut query: Query<(Entity, &Generation, &Job, &mut Morale)>,
) {
    let mut youth_workplaces: bevy_utils::HashSet<Entity> = bevy_utils::HashSet::default();

    for (_, gen, job, _) in query.iter() {
        if *gen == Generation::Immigrant {
            youth_workplaces.insert(job.workplace);
        }
    }

    for (_, gen, job, mut morale) in query.iter_mut() {
        if *gen == Generation::Founder && youth_workplaces.contains(&job.workplace) {
            // Check if already penalized
            let has_penalty = morale
                .modifiers
                .iter()
                .any(|m| m.label == "Working with ungrateful youth");
            if !has_penalty {
                morale.add_modifier(MoodModifier {
                    value: -0.05,
                    label: "Working with ungrateful youth".to_string(),
                    duration: 10,
                });
            }
        }
    }
}

#[derive(Component, Default, Clone, Debug, PartialEq)]
pub struct EdictCompliance {
    pub ignores_safety: bool,
}

pub fn evaluate_safety_edicts_system(mut query: Query<(&Generation, &mut EdictCompliance)>) {
    for (gen, mut compliance) in query.iter_mut() {
        if *gen == Generation::Immigrant && !compliance.ignores_safety {
            compliance.ignores_safety = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Job, Pop};
    use crate::layer1::social::morale::Morale;
    use crate::layer1::social::old_guard::Generation;
    use crate::layer1::utility_types::AssignmentType;
    use bevy_app::Update;

    #[test]
    fn test_mixed_generation_workgroup_reduces_founder_morale() {
        let mut world = World::new();
        let workplace = world.spawn(()).id();

        let founder = world
            .spawn((
                Pop,
                Generation::Founder,
                Morale {
                    value: 0.8,
                    modifiers: vec![],
                },
                Job {
                    workplace,
                    job_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        let youth = world
            .spawn((
                Pop,
                Generation::Immigrant,
                Morale {
                    value: 0.8,
                    modifiers: vec![],
                },
                Job {
                    workplace,
                    job_type: AssignmentType::FarmWorker,
                },
            ))
            .id();

        world.insert_resource(crate::shared::time::SimulationTime::default());

        let mut schedule = Schedule::new(Update);
        schedule.add_systems(process_generational_dissonance_system);
        schedule.run(&mut world);

        let founder_morale = world.get::<Morale>(founder).unwrap();
        let has_penalty = founder_morale
            .modifiers
            .iter()
            .any(|m| m.label == "Working with ungrateful youth" && m.value < 0.0);
        assert!(
            has_penalty,
            "Founder morale should decrease when working with the youth"
        );

        let youth_morale = world.get::<Morale>(youth).unwrap();
        let youth_has_penalty = youth_morale
            .modifiers
            .iter()
            .any(|m| m.label == "Working with ungrateful youth");
        assert!(
            !youth_has_penalty,
            "Youth morale is not directly penalized in the same way by default"
        );
    }

    #[test]
    fn test_youth_ignores_safety_edicts() {
        let mut world = World::new();

        let youth = world
            .spawn((
                Pop,
                Generation::Immigrant,
                EdictCompliance {
                    ignores_safety: false,
                },
            ))
            .id();

        let mut schedule = Schedule::new(Update);
        schedule.add_systems(evaluate_safety_edicts_system);
        schedule.run(&mut world);

        let compliance = world.get::<EdictCompliance>(youth).unwrap();
        assert!(
            compliance.ignores_safety,
            "Generation 2+ pops should passively ignore safety edicts due to lack of trauma"
        );
    }
}
