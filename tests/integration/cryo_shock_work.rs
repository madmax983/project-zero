use bevy::prelude::*;
use scale::layer1::cryo_shock::CryoShock;
use scale::layer1::execution::general_work::calculate_work_amount;
use scale::layer1::DesignationType;

#[test]
fn test_cryo_shock_reduces_work_speed() {
    let mut world = World::new();

    // Pop without CryoShock
    let pop_normal = world.spawn(()).id();

    // Pop with CryoShock
    let pop_shocked = world.spawn(CryoShock { duration_ticks: 100, severity: 0.5 }).id();

    let normal_work = calculate_work_amount(
        &world,
        pop_normal,
        DesignationType::Mine,
        None,
        1.0, // morale
        1.0, // work speed mod
        1.0, // improvised
    );

    let shocked_work = calculate_work_amount(
        &world,
        pop_shocked,
        DesignationType::Mine,
        None,
        1.0, // morale
        1.0, // work speed mod
        1.0, // improvised
    );

    assert!(shocked_work < normal_work, "CryoShock should reduce work amount");
    // Allowing for organic variation
    assert!(shocked_work <= normal_work * 0.6, "CryoShock severity 0.5 should halve work amount");
}
