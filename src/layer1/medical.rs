use bevy_ecs::prelude::*;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::health::Health;

/// Component indicating a building is a hospital that can heal patients.
#[derive(Component)]
pub struct Hospital {
    /// The amount of health restored per tick to patients.
    pub healing_rate: f32,
}

impl Default for Hospital {
    fn default() -> Self {
        Self { healing_rate: 0.5 } // 0.5 HP per tick
    }
}

/// System to heal pops assigned to a hospital.
pub fn healing_system(world: &mut World) {
    let mut updates: Vec<(Entity, f32)> = Vec::new();

    // Collect updates
    {
        // Query for pops assigned as Patient
        let mut query = world.query::<(Entity, &Health, &AssignedTo)>();

        let mut patients = Vec::new();
        for (entity, health, assigned) in query.iter(world) {
            if assigned.assignment_type == AssignmentType::Patient && health.current < health.max {
                patients.push((entity, assigned.entity));
            }
        }

        for (pop_entity, hospital_entity) in patients {
             if let Some(hospital) = world.get::<Hospital>(hospital_entity) {
                 updates.push((pop_entity, hospital.healing_rate));
             }
        }
    }

    // Apply updates
    for (entity, amount) in updates {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.current = (health.current + amount).min(health.max);
        }
    }
}

/// Evaluates the utility of seeking medical care.
///
/// If health is low (e.g. < 90%), and there is a hospital available, return a score.
/// Score increases as health decreases.
#[must_use]
pub fn evaluate_seek_medical_care<'a>(
    pop_pos: &crate::layer1::map::GridPosition,
    _needs: &crate::layer1::needs::Needs,
    health: &Health,
    weights: &crate::layer1::utility_ai::UtilityWeights,
    hospitals: impl Iterator<Item = (Entity, &'a crate::layer1::map::GridPosition, &'a Hospital)>,
) -> Option<(f32, Entity)> {
    if health.current >= health.max * 0.95 {
        return None;
    }

    let mut best: Option<(f32, Entity)> = None;

    // Urgency based on missing health
    // 50% health -> 0.5 missing -> urgency ?
    // Let's say max urgency 1.0 at 0 health.
    // urgency = (1.0 - health_pct) * scale
    let health_pct = health.current / health.max;
    let urgency = (1.0 - health_pct) * 2.0; // e.g. 50% health = 1.0 urgency. 10% health = 1.8 urgency.

    for (entity, pos, _hospital) in hospitals {
        // Simple context score
        let context = crate::layer1::utility_ai::math::calculate_context_score(
            *pop_pos,
            Some(*pos),
            10, // Assumed capacity for MVP
            0, // Occupied (not tracked for MVP yet)
            weights,
        );

        // Success modifier
        let success = crate::layer1::utility_ai::math::calculate_success_modifier(
            crate::layer1::utility_ai::ActionType::SeekMedicalCare,
            weights
        );

        let utility = urgency * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::health::Health;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_ai::UtilityWeights;

    #[test]
    fn test_hospital_component_defaults() {
        let hospital = Hospital::default();
        assert!(hospital.healing_rate > 0.0);
    }

    #[test]
    fn test_healing_system_restores_health() {
        let mut world = World::new();

        // Spawn Hospital
        let hospital_entity = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital { healing_rate: 1.0 },
        )).id();

        // Spawn Injured Pop assigned to Hospital (as patient)
        let pop_entity = world.spawn((
            Health { current: 50.0, max: 100.0 },
            AssignedTo {
                assignment_type: AssignmentType::Patient,
                entity: hospital_entity,
            }
        )).id();

        // Run system
        healing_system(&mut world);

        // Check health
        let health = world.get::<Health>(pop_entity).unwrap();
        assert!((health.current - 51.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_healing_stops_at_max() {
        let mut world = World::new();

        let hospital_entity = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital { healing_rate: 10.0 },
        )).id();

        let pop_entity = world.spawn((
            Health { current: 95.0, max: 100.0 },
            AssignedTo {
                assignment_type: AssignmentType::Patient,
                entity: hospital_entity,
            }
        )).id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop_entity).unwrap();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_assignment_type_patient_exists() {
        let assignment = AssignmentType::Patient;
        assert!(matches!(assignment, AssignmentType::Patient));
    }

    #[test]
    fn test_evaluate_seek_medical_care_healthy() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };

        // 96% health
        let health = Health { current: 96.0, max: 100.0 };

        // Spawn a hospital
        world.spawn((
            Hospital::default(),
            GridPosition { x: 5, y: 5 }
        ));

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world)
        );

        assert!(result.is_none(), "Should not seek care if health >= 95%");
    }

    #[test]
    fn test_evaluate_seek_medical_care_injured() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };

        // 90% health
        let health = Health { current: 90.0, max: 100.0 };

        // Spawn a hospital
        let hospital_entity = world.spawn((
            Hospital::default(),
            GridPosition { x: 5, y: 5 }
        )).id();

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world)
        );

        assert!(result.is_some(), "Should seek care if health < 95%");
        assert_eq!(result.unwrap().1, hospital_entity);
    }

    #[test]
    fn test_evaluate_seek_medical_care_urgency() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };

        // Spawn a hospital
        let _hospital = world.spawn((
            Hospital::default(),
            GridPosition { x: 5, y: 5 }
        )).id();

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();

        // Slightly injured (90%)
        let health_mild = Health { current: 90.0, max: 100.0 };
        let result_mild = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health_mild,
            &weights,
            hospitals_query.iter(&world)
        ).unwrap().0;

        // Severely injured (10%)
        let health_severe = Health { current: 10.0, max: 100.0 };
        let result_severe = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health_severe,
            &weights,
            hospitals_query.iter(&world)
        ).unwrap().0;

        assert!(result_severe > result_mild, "Severely injured pop should have higher utility");
    }

    #[test]
    fn test_evaluate_seek_medical_care_proximity() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let health = Health { current: 50.0, max: 100.0 };

        // Far hospital
        let far_hospital = world.spawn((
            Hospital::default(),
            GridPosition { x: 20, y: 20 }
        )).id();

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result_far = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world)
        ).unwrap();
        assert_eq!(result_far.1, far_hospital);

        // Close hospital
        let close_hospital = world.spawn((
            Hospital::default(),
            GridPosition { x: 1, y: 1 }
        )).id();

        let result_close = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world)
        ).unwrap();

        assert_eq!(result_close.1, close_hospital, "Should pick closer hospital");
        assert!(result_close.0 > result_far.0, "Closer hospital should have higher utility");
    }

    #[test]
    fn test_evaluate_seek_medical_care_no_hospitals() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let health = Health { current: 50.0, max: 100.0 };

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world)
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_healing_system_missing_hospital_component() {
        let mut world = World::new();

        // Spawn a building that is NOT a hospital (just an entity with ID)
        let fake_hospital = world.spawn(GridPosition { x: 0, y: 0 }).id();

        // Pop assigned to it
        let pop = world.spawn((
            Health { current: 50.0, max: 100.0 },
            AssignedTo {
                assignment_type: AssignmentType::Patient,
                entity: fake_hospital,
            }
        )).id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!((health.current - 50.0).abs() < f32::EPSILON, "Health should not change if assigned entity is not a hospital");
    }

    #[test]
    fn test_healing_system_wrong_assignment_type() {
        let mut world = World::new();

        let hospital = world.spawn((
            Hospital { healing_rate: 10.0 },
            GridPosition { x: 0, y: 0 }
        )).id();

        // Pop assigned as Worker (not Patient)
        let pop = world.spawn((
            Health { current: 50.0, max: 100.0 },
            AssignedTo {
                assignment_type: AssignmentType::FarmWorker, // Wrong type
                entity: hospital,
            }
        )).id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!((health.current - 50.0).abs() < f32::EPSILON, "Should not heal if not a Patient");
    }
}
