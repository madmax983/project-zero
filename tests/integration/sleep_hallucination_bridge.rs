use bevy::prelude::*;
use scale::layer1::agriculture::gastronomy::Hallucinating;
use scale::layer1::execution::general_work::calculate_work_amount;
use scale::layer1::pop::Pop;
use scale::layer1::skills::Skills;
use scale::layer1::DesignationType;

#[test]
fn test_hallucination_work_penalty() {
    let mut world = World::new();

    // Spawn a normal worker
    let normal_worker = world.spawn((Pop, Skills::default())).id();

    // Spawn a hallucinating worker
    let hallucinating_worker = world
        .spawn((Pop, Skills::default(), Hallucinating { duration: 10 }))
        .id();

    let normal_amount = calculate_work_amount(
        &world,
        normal_worker,
        DesignationType::Mine,
        None,
        0.5, // neutral morale
        1.0, // work speed mod
        1.0, // improvised efficiency
    );

    let hallucinating_amount = calculate_work_amount(
        &world,
        hallucinating_worker,
        DesignationType::Mine,
        None,
        0.5, // neutral morale
        1.0, // work speed mod
        1.0, // improvised efficiency
    );

    assert!(normal_amount > 0.0, "Normal worker should produce work");
    assert_eq!(
        hallucinating_amount, 0.0,
        "Hallucinating worker should produce 0 work"
    );
}
