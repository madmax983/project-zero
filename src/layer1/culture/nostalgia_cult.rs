use bevy_ecs::prelude::*;
use crate::layer1::psychology::stress::StressTracker;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NostalgiaCult;

pub fn nostalgia_cult_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker), Without<NostalgiaCult>>,
) {
    for (entity, tracker) in query.iter() {
        if tracker.accumulated_stress >= 85.0 {
            commands.entity(entity).insert(NostalgiaCult);
        }
    }
}

pub fn evaluate_job_score(world: &World, pop_id: Entity, job: &crate::layer1::pop::Job) -> f32 {
    if world.get::<NostalgiaCult>(pop_id).is_some() {
        if let Some(tech_level) = world.get::<crate::layer1::tech::TechLevel>(job.workplace) {
            if *tech_level == crate::layer1::tech::TechLevel::Advanced {
                return 0.0;
            }
        }
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::pop::Job;
    use crate::layer1::utility_types::AssignmentType;
    use crate::layer1::tech::TechLevel;
    use crate::layer1::architecture::Structure;
    use crate::layer1::utility_types::{ActionType, PopAction};
    use crate::layer1::execution::components::MovementTarget;
    use crate::layer1::execution::sabotage::sabotage_action_system;

    #[test]
    fn test_nostalgia_cult_formation_from_high_stress() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_cult_formation_system);

        let pop_id = app.world_mut().spawn((
            Pop,
            StressTracker { accumulated_stress: 90.0 },
        )).id();

        app.update();

        assert!(app.world().get::<NostalgiaCult>(pop_id).is_some(), "Pop with extreme stress should join the Nostalgia Cult");
    }

    #[test]
    fn test_nostalgia_cult_job_refusal() {
        let mut world = World::new();

        let pop_id = world.spawn((Pop, NostalgiaCult)).id();

        let advanced_building = world.spawn(TechLevel::Advanced).id();
        let high_tech_job = Job { workplace: advanced_building, job_type: AssignmentType::LibraryWorker };

        let score = evaluate_job_score(&world, pop_id, &high_tech_job);
        assert_eq!(score, 0.0, "Nostalgia Cult pops should refuse high-tech jobs");
    }

    #[test]
    fn test_nostalgia_cult_sabotage_action() {
        let mut app = App::new();
        app.add_systems(Update, sabotage_action_system);

        let building_id = app.world_mut().spawn(Structure { current_hp: 100.0, max_hp: 100.0 }).id();

        app.world_mut().spawn((
            Pop,
            NostalgiaCult,
            PopAction { current: ActionType::Sabotage, ..Default::default() },
            MovementTarget { target_entity: building_id, target_position: crate::layer1::map::GridPosition { x: 0, y: 0 }, for_action: ActionType::Sabotage },
        ));

        app.update();

        let structure = app.world().get::<Structure>(building_id).unwrap();
        assert!(structure.current_hp < 100.0, "Nostalgia Cult pop should damage buildings during sabotage");
    }
}
