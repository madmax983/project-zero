use bevy::prelude::*;
use scale::layer1::flora::{
    ecological_succession_system, flora_growth_system, process_flora_clearing, EcologicalState, Flora,
    FloraClearingProgress, FloraType, GrowthStage,
};
use scale::layer1::map::GridPosition;

#[test]
fn test_flora_clearing_triggers_ecological_succession() {
    let mut app = App::new();
    app.add_systems(Update, (flora_growth_system, ecological_succession_system));

    let pos = GridPosition { x: 5, y: 5, z: 0 };

    let designation_entity = app.world_mut().spawn((
        FloraClearingProgress {
            current: 0.0,
            max: 100.0,
        },
        pos.clone(),
    )).id();

    // Spawn climax flora
    app.world_mut().spawn((
        Flora {
            flora_type: FloraType::Ironwood,
            ..Default::default()
        },
        pos.clone(),
    ));

    // Clear the flora completely
    process_flora_clearing(app.world_mut(), designation_entity, 100.0);

    // Verify EcologicalState is set to cleared_recently
    let mut found_state = false;
    for state in app.world().query::<&EcologicalState>().iter(app.world()) {
        if state.cleared_recently {
            found_state = true;
            break;
        }
    }
    assert!(found_state, "EcologicalState should be marked as cleared_recently");

    // Run succession system
    app.update();

    // Verify FireWeed (Pioneer species) spawned at pos
    let mut pioneer_found = false;
    for (flora, f_pos) in app.world().query::<(&Flora, &GridPosition)>().iter(app.world()) {
        if *f_pos == pos && flora.flora_type == FloraType::FireWeed {
            pioneer_found = true;
            break;
        }
    }
    assert!(pioneer_found, "Pioneer species (FireWeed) should spawn after clearing");
}
