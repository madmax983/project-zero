use crate::layer1::resources::{ColonyResources, ResourceType};

/// Calculates work efficiency based on available resources.
///
/// Returns a tuple of (`efficiency_multiplier`, `consumed_resource_type`).
/// - Tools: 1.0 efficiency (consumes Tools)
/// - Improvised (Scrap, Stone, Wood): 0.75 efficiency (consumes Material)
/// - Bare Hands: 0.5 efficiency (consumes nothing)
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
