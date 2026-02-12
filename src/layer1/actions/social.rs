use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::social::Tavern;
use crate::layer1::utility_types::{
    ActionType, UtilityWeights, calculate_context_score, calculate_success_modifier,
    need_response_curve,
};
use bevy_ecs::prelude::*;

/// Evaluates the utility of socializing at available taverns.
pub fn evaluate_socialize<'a>(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    taverns: impl Iterator<Item = (Entity, &'a GridPosition, &'a Tavern)>,
) -> Option<(f32, Entity)> {
    let urgency = need_response_curve(needs.leisure);
    let mut best: Option<(f32, Entity)> = None;

    for (entity, pos, tavern) in taverns {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            tavern.capacity,
            tavern.visitors.len(),
            weights,
        );

        let success = calculate_success_modifier(ActionType::Socialize, weights);
        let utility = urgency * context * success;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, entity));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::social::Tavern;
    use crate::layer1::utility_types::{ActionType, UtilityWeights};

    #[test]
    fn test_evaluate_socialize_finds_tavern() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            leisure: 0.2,
            ..Default::default()
        }; // Low leisure
        let weights = UtilityWeights::default();

        let tavern_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Tavern,
                },
                GridPosition { x: 5, y: 0 },
                Tavern::default(),
            ))
            .id();

        let mut taverns = world.query::<(Entity, &GridPosition, &Tavern)>();

        let result = evaluate_socialize(&pop_pos, &needs, &weights, taverns.iter(&world));

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, tavern_entity);
        assert!(utility > 0.5, "Utility should be high for low leisure");
    }

    #[test]
    fn test_evaluate_socialize_ignored_when_leisure_high() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            leisure: 0.9,
            ..Default::default()
        };
        let weights = UtilityWeights::default();

        world.spawn((
            Building {
                building_type: BuildingType::Tavern,
            },
            GridPosition { x: 5, y: 0 },
            Tavern::default(),
        ));

        let mut taverns = world.query::<(Entity, &GridPosition, &Tavern)>();

        // Should produce very low utility or None depending on curve
        let result = evaluate_socialize(&pop_pos, &needs, &weights, taverns.iter(&world));

        if let Some((utility, _)) = result {
            assert!(utility < 0.2, "High leisure should result in low utility");
        }
    }

    #[test]
    fn test_action_type_socialize_variant() {
        let social = ActionType::Socialize;
        assert_eq!(social, ActionType::Socialize);
    }
}
