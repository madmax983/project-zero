#[cfg(test)]
mod tests {
    use crate::layer1::resources::ColonyResources;

    // We will need to import calculate_work_efficiency once we stub it.
    // For now, let's assume it will be available in crate::layer1::execution
    use crate::layer1::execution::efficiency::calculate_work_efficiency;
    use crate::layer1::resources::ResourceType;

    // 1. Test efficiency with Tools (Baseline)
    #[test]
    fn test_efficiency_with_tools() {
        let resources = ColonyResources { tools: 10.0, ..Default::default() };

        let (efficiency, consumed) = calculate_work_efficiency(&resources);

        assert_eq!(efficiency, 1.0);
        assert_eq!(consumed, Some(ResourceType::Tools)); // Tools degrade slowly
    }

    // 2. Test efficiency with Stone (Improvised)
    #[test]
    fn test_efficiency_improvised_stone() {
        let resources = ColonyResources { tools: 0.0, stone: 10.0, ..Default::default() };

        let (efficiency, consumed) = calculate_work_efficiency(&resources);

        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Stone));
    }

    // 3. Test efficiency with Wood (Improvised)
    #[test]
    fn test_efficiency_improvised_wood() {
        let resources = ColonyResources { tools: 0.0, stone: 0.0, wood: 10.0, ..Default::default() };

        let (efficiency, consumed) = calculate_work_efficiency(&resources);

        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Wood));
    }

    // 4. Test efficiency with Scrap (Improvised)
    #[test]
    fn test_efficiency_improvised_scrap() {
        let resources = ColonyResources { tools: 0.0, stone: 0.0, wood: 0.0, scrap: 10.0, ..Default::default() };

        let (efficiency, consumed) = calculate_work_efficiency(&resources);

        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Scrap));
    }

    // 5. Test fallback to Bare Hands
    #[test]
    fn test_efficiency_bare_hands() {
        let resources = ColonyResources { tools: 0.0, stone: 0.0, wood: 0.0, scrap: 0.0, ..Default::default() };

        let (efficiency, consumed) = calculate_work_efficiency(&resources);

        assert_eq!(efficiency, 0.5);
        assert_eq!(consumed, None);
    }

    // 6. Test Consumption Logic
    #[test]
    fn test_consumption_logic() {
        // Mock RNG or force breakage
        let mut resources = ColonyResources { stone: 10.0, ..Default::default() };

        // Simulate "Using" the improvised tool
        // If the system calls consume(ResourceType::Stone, 1.0)
        resources.consume(ResourceType::Stone, 1.0);

        assert_eq!(resources.stone, 9.0);
    }
}
