use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::architecture::structure::Structure;
use crate::layer1::entities::pop::Job;

#[derive(Component)]
pub struct NostalgiaCult;

#[allow(clippy::type_complexity)]
pub fn nostalgia_cult_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker), (With<Pop>, Without<NostalgiaCult>)>,
) {
    for (entity, stress) in query.iter() {
        if stress.accumulated_stress >= 85.0 {
            commands.entity(entity).insert(NostalgiaCult);
        }
    }
}

pub fn evaluate_job_score(world: &World, pop_id: Entity, job: &Job) -> f32 {
    if world.get::<NostalgiaCult>(pop_id).is_some() {
        if let Some(tech_level) = world.get::<crate::layer1::biology::grafting::TechLevel>(job.workplace) {
            if tech_level.level > 1 {
                return 0.0;
            }
        }
    }
    1.0
}

pub fn execute_sabotage_action(world: &mut World, pop_id: Entity, target_id: Entity) {
    if world.get::<NostalgiaCult>(pop_id).is_some() {
        if let Some(mut structure) = world.get_mut::<Structure>(target_id) {
            structure.current_hp -= 10.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nostalgia_cult_formation_from_high_stress() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_cult_formation_system);
        // Setup pop with extreme stress
        let pop_id = app.world_mut().spawn((
            Pop,
            StressTracker { accumulated_stress: 90.0 },
        )).id();

        // Run system simulating time/cult formation
        app.update();

        // Assert that the pop has joined the Nostalgia Cult
        assert!(app.world().get::<NostalgiaCult>(pop_id).is_some(), "Pop with extreme stress should join the Nostalgia Cult");
    }

    #[test]
    fn test_nostalgia_cult_job_refusal() {
        let mut app = App::new();

        // Setup pop in the Nostalgia Cult
        let pop_id = app.world_mut().spawn((
            Pop,
            NostalgiaCult,
        )).id();

        let building_id = app.world_mut().spawn(crate::layer1::biology::grafting::TechLevel { level: 2 }).id();

        // Setup a high-tech job
        let high_tech_job = Job { workplace: building_id, job_type: crate::layer1::utility_types::AssignmentType::Administrator };

        // Evaluate action score for the job
        let score = evaluate_job_score(&app.world(), pop_id, &high_tech_job);

        // Assert that the cult member refuses the high tech job (score 0.0)
        assert_eq!(score, 0.0, "Nostalgia Cult pops should refuse high-tech jobs");

        let primitive_building_id = app.world_mut().spawn(crate::layer1::biology::grafting::TechLevel { level: 1 }).id();
        let primitive_job = Job { workplace: primitive_building_id, job_type: crate::layer1::utility_types::AssignmentType::FarmWorker };
        let score2 = evaluate_job_score(&app.world(), pop_id, &primitive_job);
        assert_eq!(score2, 1.0, "Nostalgia Cult pops should accept primitive jobs");

        let non_cult_pop_id = app.world_mut().spawn(Pop).id();
        let score3 = evaluate_job_score(&app.world(), non_cult_pop_id, &high_tech_job);
        assert_eq!(score3, 1.0, "Non-cult pops should accept high-tech jobs");
    }

    #[test]
    fn test_nostalgia_cult_sabotage_action() {
        let mut app = App::new();

        // Setup pop in the Nostalgia Cult
        let pop_id = app.world_mut().spawn((
            Pop,
            NostalgiaCult,
        )).id();

        // Setup an advanced building target
        let building_id = app.world_mut().spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Force pop to execute sabotage action
        execute_sabotage_action(app.world_mut(), pop_id, building_id);

        // Assert that the building took damage
        let integrity = app.world().get::<Structure>(building_id).unwrap();
        assert!(integrity.current_hp < integrity.max_hp, "Nostalgia Cult pop should damage advanced buildings during sabotage");
    }
}
