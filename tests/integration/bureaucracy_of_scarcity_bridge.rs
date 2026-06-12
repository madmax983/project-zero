use bevy::prelude::*;
use scale::layer1::bureaucracy_of_scarcity::Pop as ScarcityPop;
use scale::layer1::bureaucracy_of_scarcity::{
    apply_rationing_buff_system, evaluate_scarcity_system, Colony, GlobalRationingModifier,
    JobAssignment, JobBoard, ResourceStorage,
};
use scale::layer1::core::integration::{
    apply_scarcity_rationing_bridge_system, sync_scarcity_resources_bridge_system,
};
use scale::layer1::pop::Pop;
use scale::layer1::psychology::needs::Needs;
use scale::layer1::resources::ColonyResources;

#[test]
fn test_bureaucracy_of_scarcity_bridge() {
    let mut app = App::new();

    app.add_systems(
        Update,
        (
            sync_scarcity_resources_bridge_system,
            evaluate_scarcity_system,
            apply_rationing_buff_system,
            apply_scarcity_rationing_bridge_system,
        )
            .chain(),
    );

    let colony = app
        .world_mut()
        .spawn((
            Colony,
            ColonyResources {
                food: 10.0,
                ..Default::default()
            },
            ResourceStorage {
                food: 100,
                population_demand: 100,
            },
            JobBoard {
                available_bureaucrat_jobs: 0,
            },
            GlobalRationingModifier {
                reduction_percent: 0.0,
            },
        ))
        .id();

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            ScarcityPop,
            Needs {
                hunger: 0.5,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 1.0,
            },
            JobAssignment {
                is_active_bureaucrat: true,
            },
            scale::layer1::bureaucracy_of_scarcity::ConsumptionRate {
                base_food_per_tick: 1.0,
                food_per_tick: 1.0,
            },
        ))
        .id();

    app.update();

    let board = app.world().get::<JobBoard>(colony).unwrap();
    assert!(
        board.available_bureaucrat_jobs > 0,
        "Scarcity bridge should sync low food to trigger job spawn"
    );

    let modifier = app.world().get::<GlobalRationingModifier>(colony).unwrap();
    assert_eq!(modifier.reduction_percent, 0.05);

    let needs = app.world().get::<Needs>(pop).unwrap();
    assert!(
        needs.hunger > 0.5,
        "Bridge should restore some hunger decay"
    );
}
