use crate::layer1::architecture::building::BuildingType;
use crate::layer1::economy::refining::get_refining_recipe;
use crate::layer1::economy::resources::ColonyResources;

// 1. Resource Tests
#[test]
fn test_colony_resources_clothing_fields() {
    let res = ColonyResources::default();
    // Fields should exist and be initialized to 0.0
    assert_eq!(res.fiber, 0.0);
    assert_eq!(res.cloth, 0.0);
    assert_eq!(res.clothing, 0.0);

    // Defaults maxes should be > 0.0
    assert!(res.max_fiber > 0.0);
    assert!(res.max_cloth > 0.0);
    assert!(res.max_clothing > 0.0);
}

// 2. Refining Recipe Tests
#[test]
fn test_weaver_recipe() {
    let res = ColonyResources {
        fiber: 5.0,
        cloth: 0.0,
        max_cloth: 10.0,
        ..ColonyResources::zeroed()
    };

    let (can_afford, input, output, _) = get_refining_recipe(BuildingType::Weaver, &res);

    assert!(can_afford);
    assert_eq!(input.fiber, 1.0);
    assert_eq!(output.cloth, 1.0);
}

#[test]
fn test_tailor_recipe() {
    let res = ColonyResources {
        cloth: 5.0,
        clothing: 0.0,
        max_clothing: 10.0,
        ..ColonyResources::zeroed()
    };

    let (can_afford, input, output, _) = get_refining_recipe(BuildingType::Tailor, &res);

    assert!(can_afford);
    assert_eq!(input.cloth, 1.0);
    assert_eq!(output.clothing, 1.0);
}
