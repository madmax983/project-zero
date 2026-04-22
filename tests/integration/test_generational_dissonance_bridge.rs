use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::social::generational_dissonance::EdictCompliance;
use scale::layer1::social::old_guard::Generation;
use scale::layer1::utility_types::{ActionType, UtilityWeights};
#[cfg(test)]
use scale::layer1::mind::utility_eval_types::{PopEvalData, ScorableCandidate, CandidateEvaluator};
use scale::layer1::map::GridPosition;
use scale::layer1::needs::Needs;
use scale::layer1::mind::utility_types::PopAction;
#[cfg(test)]
use scale::layer1::mind::utility_eval_types::WorldContext;
#[cfg(test)]
use scale::layer1::mind::utility_ai::evaluate_single_pop;
#[cfg(test)]
use scale::layer1::mind::utility_eval_types::UtilityAIBuffer;

#[test]
fn test_generational_dissonance_utility_ai() {
    let mut world = World::new();

    // Create a pop that ignores safety
    let pop_entity = world.spawn((
        Generation::Immigrant,
        EdictCompliance {
            ignores_safety: true,
        },
    )).id();

    // We can just unit test evaluate_and_consider directly
    let resources = scale::layer1::resources::ColonyResources::default();
    let cycle = scale::layer1::day_night::DayNightCycle::default();
    let taboo = scale::layer1::taboo::TabooState::default();
    let zone_grid = scale::layer1::zone::ZoneGrid::new(1, 1);

    let context = WorldContext {
        resources: &resources,
        cycle: &cycle,
        taboo: &taboo,
        factions: None,
        zone_grid: &zone_grid,
        temperature_grid: None,
    };

    let mut evaluator = CandidateEvaluator::new(0.0, false);

    let danger_action = ActionType::Work; // Base danger 0.001
    assert!(danger_action.danger_level() > 0.0);

    // Test without ignores_safety
    evaluator.evaluate_and_consider(Some((100.0, pop_entity)), danger_action, &context, 0.0, false);
    let (_, utility_with_safety, _) = evaluator.result();

    // Test with ignores_safety
    let mut evaluator2 = CandidateEvaluator::new(0.0, false);
    evaluator2.evaluate_and_consider(Some((100.0, pop_entity)), danger_action, &context, 0.0, true);
    let (_, utility_ignoring_safety, _) = evaluator2.result();

    // The utility should be higher when ignoring safety because it doesn't get penalized
    assert!(utility_ignoring_safety > utility_with_safety, "{} > {}", utility_ignoring_safety, utility_with_safety);
}
