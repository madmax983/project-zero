use scale::layer1::resources::{ColonyResources, ResourceType};

#[test]
fn test_add_nan_is_ignored() {
    let mut resources = ColonyResources {
        food: 10.0,
        ..Default::default()
    };

    // Attempt to add NaN
    resources.add_food(f32::NAN);

    // Should remain 10.0
    // Note: NaN != NaN, so we check if it is still 10.0.
    // If it became NaN, this assertion will fail.
    assert!(
        (resources.food - 10.0).abs() < f32::EPSILON,
        "Food should be 10.0, but was {}",
        resources.food
    );
}

#[test]
fn test_consume_nan_is_ignored() {
    let mut resources = ColonyResources {
        wood: 10.0,
        ..Default::default()
    };

    resources.consume(ResourceType::Wood, f32::NAN);

    assert!(
        (resources.wood - 10.0).abs() < f32::EPSILON,
        "Wood should be 10.0, but was {}",
        resources.wood
    );
}

#[test]
fn test_try_deduct_nan_is_rejected() {
    let mut resources = ColonyResources {
        stone: 10.0,
        ..Default::default()
    };

    let mut cost = ColonyResources::zeroed();
    cost.stone = f32::NAN;

    let success = resources.try_deduct(&cost);

    assert!(!success, "Should not be able to deduct NaN cost");
    assert!(
        (resources.stone - 10.0).abs() < f32::EPSILON,
        "Stone should remain 10.0"
    );
}
