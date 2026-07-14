use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::skills::{SkillType, Skills};
use bevy::prelude::*;

#[derive(Component)]
pub struct MemeticDisassociation {
    pub level: f32,
}

#[derive(Component)]
pub struct EngramPurchaseIntent;

pub fn process_memory_smuggling(
    mut query: Query<
        (
            Entity,
            &mut StressTracker,
            &mut MemeticDisassociation,
            &mut Skills,
        ),
        With<EngramPurchaseIntent>,
    >,
    mut commands: Commands,
) {
    for (entity, mut stress, mut disassociation, mut skills) in query.iter_mut() {
        // Reduce stress significantly
        stress.accumulated_stress = (stress.accumulated_stress - 40.0).max(0.0);

        // Increase disassociation
        disassociation.level += 20.0;

        // Degrade real skills due to memory overwrite
        // For simplicity we degrade engineering as in the spec
        let _current_eng = skills.get_xp(SkillType::Engineering);
        // We can't directly set XP easily without clearing and inserting, so let's mutate the underlying map
        if let Some(xp) = skills.xp.get_mut(&SkillType::Engineering) {
            *xp = (*xp - 1000.0).max(0.0);
        }

        // Remove intent after purchase
        commands.entity(entity).remove::<EngramPurchaseIntent>();
    }
}

pub fn process_job_execution(
    mut commands: Commands,
    query: Query<(Entity, &MemeticDisassociation), With<crate::layer1::entities::pop::Job>>,
) {
    for (entity, disassociation) in query.iter() {
        if disassociation.level >= 100.0 {
            // Abandon jobs completely
            commands
                .entity(entity)
                .remove::<crate::layer1::entities::pop::Job>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_memory_smuggler_market_reduces_stress_but_causes_disassociation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_memory_smuggling);

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Engineering, 5000.0);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 90.0,
                },
                skills,
                MemeticDisassociation { level: 0.0 },
                EngramPurchaseIntent,
            ))
            .id();

        // Act
        app.update();

        // Assert
        let stress = app.world().get::<StressTracker>(pop_entity).unwrap();
        let disassociation = app
            .world()
            .get::<MemeticDisassociation>(pop_entity)
            .unwrap();
        let skills = app.world().get::<Skills>(pop_entity).unwrap();

        assert!(
            stress.accumulated_stress < 90.0,
            "Stress should be reduced by the fake memory."
        );
        assert!(
            disassociation.level > 0.0,
            "Disassociation should increase from buying engrams."
        );
        assert!(
            skills.get_xp(SkillType::Engineering) < 5000.0,
            "Relying on fake memories should degrade real skills."
        );
    }

    #[test]
    fn test_high_disassociation_causes_job_failure() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_job_execution);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                MemeticDisassociation { level: 100.0 },
                crate::layer1::entities::pop::Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: crate::layer1::mind::utility_types::AssignmentType::FarmWorker,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let job = app
            .world()
            .get::<crate::layer1::entities::pop::Job>(pop_entity);
        assert!(
            job.is_none(),
            "High disassociation should cause pops to fail or abandon their jobs."
        );
    }
}
