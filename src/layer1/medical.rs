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
}
