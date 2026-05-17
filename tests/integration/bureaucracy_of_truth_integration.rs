use bevy::prelude::*;
use scale::layer1::resources::ColonyResources;
use scale::layer1::social::unrest::Unrest;
use scale::layer2::governance::{Governor, GovernorStats};
use scale::layer3::bureaucracy_of_truth::{ColonyReport, ColonyState, generate_colony_reports_system};
use scale::layer3::integration::bureaucracy_of_truth_integration_system;

#[test]
fn test_bureaucracy_of_truth_integration() {
    let mut app = App::new();

    // Layer 1 resources
    let mut resources = ColonyResources::default();
    resources.food = 10.0;
    app.insert_resource(resources);

    let mut unrest = Unrest::default();
    unrest.level = 0.8;
    app.insert_resource(unrest);

    // Layer 3 elements
    let pop_entity = app
        .world_mut()
        .spawn(GovernorStats {
            loyalty: 10.0,
            corruption: 90.0,
            ambition: 0.0,
        })
        .id();

    let colony_entity = app
        .world_mut()
        .spawn((
            ColonyState {
                food_reserves: 0,
                unrest: 0.0,
            },
            Governor {
                pop_entity,
                assigned_at: 0,
            },
        ))
        .id();

    app.add_systems(Update, (bureaucracy_of_truth_integration_system, generate_colony_reports_system).chain());
    app.update();

    let state = app.world().get::<ColonyState>(colony_entity).unwrap();
    assert_eq!(state.food_reserves, 10, "ColonyState should be updated from ColonyResources");
    assert_eq!(state.unrest, 80.0, "ColonyState should be updated from Unrest");

    let report = app.world().get::<ColonyReport>(colony_entity).unwrap();
    assert!(report.reported_food > 100, "Corrupt governor should falsify the report");
    assert!(report.reported_unrest < 10.0, "Corrupt governor should falsify the report");
}
