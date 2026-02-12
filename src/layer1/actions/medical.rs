use crate::layer1::health::Health;
use crate::layer1::medical::Hospital;
use crate::layer1::utility_types::{
    ActionType, UtilityWeights, calculate_context_score, calculate_success_modifier,
};
use bevy_ecs::prelude::*;

/// Evaluates the utility of seeking medical care.
///
/// If health is low (e.g. < 90%), and there is a hospital available, return a score.
/// Score increases as health decreases.
#[must_use]
pub fn evaluate_seek_medical_care<'a>(
    pop_pos: &crate::layer1::map::GridPosition,
    _needs: &crate::layer1::needs::Needs,
    health: &Health,
    weights: &UtilityWeights,
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
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            10, // Assumed capacity for MVP
            0,  // Occupied (not tracked for MVP yet)
            weights,
        );

        // Success modifier
        let success = calculate_success_modifier(ActionType::SeekMedicalCare, weights);

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
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_seek_medical_care_healthy() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };

        // 96% health
        let health = Health {
            current: 96.0,
            max: 100.0,
        };

        // Spawn a hospital
        world.spawn((Hospital::default(), GridPosition { x: 5, y: 5 }));

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world),
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
        let health = Health {
            current: 90.0,
            max: 100.0,
        };

        // Spawn a hospital
        let hospital_entity = world
            .spawn((Hospital::default(), GridPosition { x: 5, y: 5 }))
            .id();

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world),
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
        let _hospital = world
            .spawn((Hospital::default(), GridPosition { x: 5, y: 5 }))
            .id();

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();

        // Slightly injured (90%)
        let health_mild = Health {
            current: 90.0,
            max: 100.0,
        };
        let result_mild = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health_mild,
            &weights,
            hospitals_query.iter(&world),
        )
        .unwrap()
        .0;

        // Severely injured (10%)
        let health_severe = Health {
            current: 10.0,
            max: 100.0,
        };
        let result_severe = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health_severe,
            &weights,
            hospitals_query.iter(&world),
        )
        .unwrap()
        .0;

        assert!(
            result_severe > result_mild,
            "Severely injured pop should have higher utility"
        );
    }

    #[test]
    fn test_evaluate_seek_medical_care_proximity() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let health = Health {
            current: 50.0,
            max: 100.0,
        };

        // Far hospital
        let far_hospital = world
            .spawn((Hospital::default(), GridPosition { x: 20, y: 20 }))
            .id();

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result_far = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world),
        )
        .unwrap();
        assert_eq!(result_far.1, far_hospital);

        // Close hospital
        let close_hospital = world
            .spawn((Hospital::default(), GridPosition { x: 1, y: 1 }))
            .id();

        let result_close = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world),
        )
        .unwrap();

        assert_eq!(
            result_close.1, close_hospital,
            "Should pick closer hospital"
        );
        assert!(
            result_close.0 > result_far.0,
            "Closer hospital should have higher utility"
        );
    }

    #[test]
    fn test_evaluate_seek_medical_care_no_hospitals() {
        let mut world = World::new();
        let weights = UtilityWeights::default();
        let needs = Needs::default();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let health = Health {
            current: 50.0,
            max: 100.0,
        };

        let mut hospitals_query = world.query::<(Entity, &GridPosition, &Hospital)>();
        let result = evaluate_seek_medical_care(
            &pop_pos,
            &needs,
            &health,
            &weights,
            hospitals_query.iter(&world),
        );

        assert!(result.is_none());
    }
}
