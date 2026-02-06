#[cfg(test)]
mod tests {
    use crate::layer1::building::BuildingType;
    use crate::layer1::refining::get_refining_recipe;
    use crate::layer1::resources::ColonyResources;
    // use crate::layer1::execution::work_execution_system; // We will test logic via helpers or separate tests if needed

    // 1. ColonyResources should have tools
    #[test]
    fn test_colony_resources_tools_fields() {
        let resources = ColonyResources::default();
        assert_eq!(resources.tools, 2.0);
        assert_eq!(resources.max_tools, 50.0); // Default cap
    }

    // 2. BuildingType::Smithy should exist
    #[test]
    fn test_building_type_smithy() {
        let bt = BuildingType::Smithy;
        assert_eq!(bt.label(), "Smithy");
        assert_eq!(bt.char(), 'T'); // T for Tool

        // Check cost (example: 30 Wood, 10 Stone - WAIT, spec said "30 Wood, 10 Stone" in the test example but "Metal + Wood" for recipe. I will follow the test example for cost)
        // Wait, the spec text says "Smithy building produces Tools from Metal and Wood."
        // The spec TEST says:
        // let cost = bt.cost();
        // assert_eq!(cost.wood, 30.0);
        // assert_eq!(cost.stone, 10.0);
        // I will follow the spec test for cost.

        let cost = bt.cost();
        assert_eq!(cost.wood, 30.0);
        assert_eq!(cost.stone, 10.0);
    }

    // 3. Refining Recipe for Smithy
    #[test]
    fn test_smithy_recipe() {
        let mut resources = ColonyResources::default();
        resources.metal = 1.0;
        resources.wood = 1.0;
        resources.tools = 0.0;

        let (can_refine, input, output) = get_refining_recipe(BuildingType::Smithy, &resources);

        assert!(can_refine);
        assert_eq!(input.metal, 1.0);
        assert_eq!(input.wood, 1.0);
        assert_eq!(output.tools, 1.0);
    }
}
