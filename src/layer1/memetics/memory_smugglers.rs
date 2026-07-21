use bevy_ecs::prelude::*;

use crate::layer1::entities::pop::Job;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::skills::Skills;

#[derive(Component)]
pub struct MemeticDisassociation {
    pub level: f32,
}

#[derive(Component)]
pub struct EngramPurchaseIntent;

pub fn process_memory_smuggling_system(
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
        for (_, xp) in skills.xp.iter_mut() {
            *xp = (*xp - 10.0).max(0.0);
        }

        // Remove intent after purchase
        commands.entity(entity).remove::<EngramPurchaseIntent>();
    }
}

pub fn process_job_execution_system(
    query: Query<(Entity, &MemeticDisassociation, &Job)>,
    mut commands: Commands,
) {
    for (entity, disassociation, _) in query.iter() {
        if disassociation.level >= 100.0 {
            // Unassign job from pop
            commands.entity(entity).remove::<Job>();
        }
    }
}

/// Marker component to prevent emitting duplicate events for MemeticDisassociation
#[derive(Component)]
pub struct ReportedMemeticDisassociation;

/// INT-1274: Bridges `MemeticDisassociation` to `AddChronicleEvent` (Chronicle).
pub fn memory_smugglers_chronicle_bridge(
    query: Query<
        (
            Entity,
            &crate::layer1::memetics::memory_smugglers::MemeticDisassociation,
        ),
        Without<ReportedMemeticDisassociation>,
    >,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
    mut commands: Commands,
) {
    for (entity, disassociation) in query.iter() {
        if disassociation.level >= 100.0 {
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                text: "Memory Smugglers have exacted their toll. Cases of extreme memetic disassociation are sweeping the colony, leaving pops hollowed out and unable to perform basic functions.".to_string(),
                importance: crate::layer1::core::chronicle::EventImportance::Major,
            });
            commands
                .entity(entity)
                .insert(ReportedMemeticDisassociation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::entities::pop::{Job, Pop};
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::skills::{SkillType, Skills};

    #[test]
    fn test_memory_smuggler_market_reduces_stress_but_causes_disassociation() {
        // Arrange
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(process_memory_smuggling_system);

        let mut skills = Skills::default();
        skills.xp.insert(SkillType::Construction, 50.0);

        let pop_entity = world
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
        schedule.run(&mut world);

        // Assert
        let stress = world.get::<StressTracker>(pop_entity).unwrap();
        let disassociation = world.get::<MemeticDisassociation>(pop_entity).unwrap();
        let skills = world.get::<Skills>(pop_entity).unwrap();

        assert!(
            stress.accumulated_stress < 90.0,
            "Stress should be reduced by the fake memory."
        );
        assert!(
            disassociation.level > 0.0,
            "Disassociation should increase from buying engrams."
        );
        assert!(
            skills.xp.get(&SkillType::Construction).unwrap() < &50.0,
            "Relying on fake memories should degrade real skills."
        );
    }

    #[test]
    fn test_high_disassociation_causes_job_failure() {
        // Arrange
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(process_job_execution_system);

        let workplace = world.spawn_empty().id();
        let pop_entity = world
            .spawn((
                Pop,
                MemeticDisassociation { level: 100.0 },
                Job {
                    workplace,
                    job_type: crate::layer1::utility_types::AssignmentType::FarmWorker,
                },
            ))
            .id();

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(
            world.get::<Job>(pop_entity).is_none(),
            "High disassociation should cause pops to fail or abandon their jobs."
        );
    }
}
