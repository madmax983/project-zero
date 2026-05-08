use bevy::prelude::*;
use scale::layer1::pop::Pop;
use scale::layer1::map::GridPosition;
use scale::layer1::needs::Needs;
use scale::layer1::mind::utility_types::{ActionType, PopAction, UtilityWeights};
use scale::layer1::mind::utility_eval_types::{ScorableCandidate, UtilityAIBuffer, PopEvalData};
use scale::layer1::culture::nostalgia::Nostalgia;
use scale::layer1::architecture::Structure;
use scale::layer1::building::BuildingType;

#[test]
fn test_nostalgia_sabotage_integration() {
    // Basic test confirming that nostalgia overrides normal work logic
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Evaluate logic locally to test the seam
    let mut buffer = UtilityAIBuffer::default();

    let structure1 = Entity::from_raw(10);
    let mut cand1 = ScorableCandidate::new(structure1, GridPosition { x: 5, y: 0 });
    cand1.is_advanced_tech = false;

    let structure2 = Entity::from_raw(20);
    let mut cand2 = ScorableCandidate::new(structure2, GridPosition { x: 10, y: 0 });
    cand2.is_advanced_tech = true;

    buffer.all_structures.push(cand1);
    buffer.all_structures.push(cand2);

    let mut data = scale::layer1::mind::utility_eval_types::PopEvalData {
        entity: Entity::from_raw(1),
        pos: GridPosition { x: 0, y: 0 },
        needs: Needs::default(),
        weights: UtilityWeights::default(),
        action: PopAction::default(),
        equipment: None,
        carrying: None,
        carrying_item: None,
        carrying_item_type: None,
        mental_state: None,
        drafted: None,
        faction_member: None,
        penal_labor: None,
        breakdown: None,
        traits: None,
        stress: 0.0,
        hobby_type: None,
        chemical_state: None,
        is_memetic_carrier: false,
        health: None,
        job: None,
        insulation: 0.0,
        is_silent: false,
        is_nostalgic: true,
    };

    let result = scale::layer1::actions::sabotage::evaluate_sabotage(&data, &buffer);
    assert!(result.is_some());
    let (action, utility, target) = result.unwrap();
    assert_eq!(action, ActionType::Sabotage);
    assert_eq!(utility, 100.0);
    assert_eq!(target, Some(structure2));
}
