use bevy_ecs::prelude::*;
use crate::layer1::social::old_guard::Generation;
use crate::layer1::pop::Job;
use crate::layer1::social::morale::Morale;
use crate::layer1::social::morale::MoodModifier;
use std::collections::HashSet;

pub fn process_generational_dissonance_system(
    mut query: Query<(Entity, &Generation, &Job, &mut Morale)>,
) {
    let mut youth_workplaces: HashSet<Entity> = HashSet::new();

    for (_, gen, job, _) in query.iter() {
        if *gen == Generation::Immigrant {
            youth_workplaces.insert(job.workplace);
        }
    }

    let mut penalties: Vec<(Entity, f32)> = Vec::new();

    for (entity, gen, job, morale) in query.iter() {
        if *gen == Generation::Founder && youth_workplaces.contains(&job.workplace) {
            // Check if already penalized
            let has_penalty = morale.modifiers.iter().any(|m| m.label == "Working with ungrateful youth");
            if !has_penalty {
                penalties.push((entity, -0.05)); // 5% morale penalty
            }
        }
    }

    for (entity, penalty) in penalties {
        if let Ok((_, _, _, mut morale)) = query.get_mut(entity) {
            morale.add_modifier(MoodModifier {
                value: penalty,
                label: "Working with ungrateful youth".to_string(),
                duration: 10,
            });
        }
    }
}

#[derive(Component, Default, Clone, Debug, PartialEq)]
pub struct EdictCompliance {
    pub ignores_safety: bool,
}

pub fn evaluate_safety_edicts_system(
    mut query: Query<(&Generation, &mut EdictCompliance)>,
) {
    for (gen, mut compliance) in query.iter_mut() {
        if *gen == Generation::Immigrant && !compliance.ignores_safety {
            compliance.ignores_safety = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::Update;
    use crate::layer1::social::old_guard::{Generation};
    use crate::layer1::pop::{Pop, Job};
    use crate::layer1::utility_types::AssignmentType;
    use crate::layer1::social::morale::{Morale};

    #[test]
    fn test_mixed_generation_workgroup_reduces_founder_morale() {
        let mut world = World::new();
        let workplace = world.spawn(()).id();

        let founder = world.spawn((
            Pop,
            Generation::Founder,
            Morale { value: 0.8, modifiers: vec![] },
            Job { workplace, job_type: AssignmentType::FarmWorker },
        )).id();

        let youth = world.spawn((
            Pop,
            Generation::Immigrant,
            Morale { value: 0.8, modifiers: vec![] },
            Job { workplace, job_type: AssignmentType::FarmWorker },
        )).id();

        world.insert_resource(crate::shared::time::SimulationTime::default());

        let mut schedule = Schedule::new(Update);
        schedule.add_systems(process_generational_dissonance_system);
        schedule.run(&mut world);

        let founder_morale = world.get::<Morale>(founder).unwrap();
        let has_penalty = founder_morale.modifiers.iter().any(|m| m.label == "Working with ungrateful youth" && m.value < 0.0);
        assert!(has_penalty, "Founder morale should decrease when working with the youth");

        let youth_morale = world.get::<Morale>(youth).unwrap();
        let youth_has_penalty = youth_morale.modifiers.iter().any(|m| m.label == "Working with ungrateful youth");
        assert!(!youth_has_penalty, "Youth morale is not directly penalized in the same way by default");
    }

    #[test]
    fn test_youth_ignores_safety_edicts() {
        let mut world = World::new();

        let youth = world.spawn((
            Pop,
            Generation::Immigrant,
            EdictCompliance { ignores_safety: false },
        )).id();

        let mut schedule = Schedule::new(Update);
        schedule.add_systems(evaluate_safety_edicts_system);
        schedule.run(&mut world);

        let compliance = world.get::<EdictCompliance>(youth).unwrap();
        assert!(compliance.ignores_safety, "Generation 2+ pops should passively ignore safety edicts due to lack of trauma");
    }
}
