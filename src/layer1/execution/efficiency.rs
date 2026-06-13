use crate::layer1::resources::{ColonyResources, ResourceType};

/// Calculates work efficiency based on available resources.
///
/// Returns a tuple of (`efficiency_multiplier`, `consumed_resource_type`).
/// - Tools: 1.0 efficiency (consumes Tools)
/// - Improvised (Scrap, Stone, Wood): 0.75 efficiency (consumes Material)
/// - Bare Hands: 0.5 efficiency (consumes nothing)
///
/// # Examples
///
/// ```
/// use scale::layer1::resources::{ColonyResources, ResourceType};
/// use scale::layer1::execution::efficiency::calculate_work_efficiency;
///
/// let mut resources = ColonyResources::default();
/// resources.tools = 10.0;
///
/// let (efficiency, consumed) = calculate_work_efficiency(&resources);
/// assert_eq!(efficiency, 1.0);
/// assert_eq!(consumed, Some(ResourceType::Tools));
///
/// // Bare hands fallback
/// let mut empty_resources = ColonyResources::default();
/// empty_resources.tools = 0.0;
/// empty_resources.scrap = 0.0;
/// empty_resources.stone = 0.0;
/// empty_resources.wood = 0.0;
/// let (efficiency, consumed) = calculate_work_efficiency(&empty_resources);
/// assert_eq!(efficiency, 0.5);
/// assert_eq!(consumed, None);
/// ```
pub fn calculate_work_efficiency(res: &ColonyResources) -> (f32, Option<ResourceType>) {
    // 1. Proper Tools (Best)
    // Checks if there are any tools in the global stockpile
    if res.tools >= 1.0 {
        return (1.0, Some(ResourceType::Tools));
    }

    // 2. Improvised Tools (Okay)
    // Priority: Scrap > Stone > Wood
    if res.scrap >= 1.0 {
        return (0.75, Some(ResourceType::Scrap));
    }
    if res.stone >= 1.0 {
        return (0.75, Some(ResourceType::Stone));
    }
    if res.wood >= 1.0 {
        return (0.75, Some(ResourceType::Wood));
    }

    // 3. Bare Hands (Slow)
    (0.5, None)
}
#[cfg(test)]
mod efficiency_extra_tests {
    use super::*;
    use crate::layer1::resources::{ColonyResources, ResourceType};

    #[test]
    fn test_calculate_work_efficiency_with_tools() {
        let mut resources = ColonyResources::zeroed();
        resources.tools = 1.0;
        let (efficiency, consumed) = calculate_work_efficiency(&resources);
        assert_eq!(efficiency, 1.0);
        assert_eq!(consumed, Some(ResourceType::Tools));
    }

    #[test]
    fn test_calculate_work_efficiency_with_scrap() {
        let mut resources = ColonyResources::zeroed();
        resources.scrap = 1.0;
        let (efficiency, consumed) = calculate_work_efficiency(&resources);
        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Scrap));
    }

    #[test]
    fn test_calculate_work_efficiency_with_stone() {
        let mut resources = ColonyResources::zeroed();
        resources.stone = 1.0;
        let (efficiency, consumed) = calculate_work_efficiency(&resources);
        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Stone));
    }

    #[test]
    fn test_calculate_work_efficiency_with_wood() {
        let mut resources = ColonyResources::zeroed();
        resources.wood = 1.0;
        let (efficiency, consumed) = calculate_work_efficiency(&resources);
        assert_eq!(efficiency, 0.75);
        assert_eq!(consumed, Some(ResourceType::Wood));
    }

    #[test]
    fn test_calculate_work_efficiency_bare_hands() {
        let resources = ColonyResources::zeroed();
        let (efficiency, consumed) = calculate_work_efficiency(&resources);
        assert_eq!(efficiency, 0.5);
        assert_eq!(consumed, None);
    }
}
