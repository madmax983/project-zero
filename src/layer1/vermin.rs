use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

/// Tracks the severity of vermin infestation in the colony.
#[derive(Resource)]
pub struct VerminState {
    /// The current severity level (0.0 to 100.0).
    pub severity: f32,
    /// The maximum possible severity level.
    pub max_severity: f32,
}

impl Default for VerminState {
    fn default() -> Self {
        Self {
            severity: 0.0,
            max_severity: 100.0,
        }
    }
}

// Growth Constants
// 1000 food -> ~10.0 severity over a season (250 ticks).
// 10 / 250 = 0.04 per tick.
// 0.04 / 1000 = 0.00004
const GROWTH_FACTOR_FOOD: f32 = 0.00005;
const GROWTH_FACTOR_WASTE: f32 = 0.0001; // Waste attracts more vermin
const NATURAL_DECAY: f32 = 0.05; // Dies off slowly if no food

/// System that updates vermin severity based on available food and waste.
pub fn vermin_growth_system(
    mut vermin: ResMut<VerminState>,
    resources: Res<ColonyResources>,
    policies: Res<ColonyPolicies>,
) {
    let food_impact = resources.food * GROWTH_FACTOR_FOOD;
    let waste_impact = resources.waste * GROWTH_FACTOR_WASTE;

    let mut growth = food_impact + waste_impact;

    // Apply Pest Control Policy
    if policies.is_active(Policy::PestControl) {
        growth *= 0.5; // Halves growth rate
    }

    if growth > 0.0 {
        vermin.severity += growth;
    } else {
        vermin.severity -= NATURAL_DECAY;
    }

    // Clamp
    vermin.severity = vermin.severity.clamp(0.0, vermin.max_severity);
}

/// Calculates the food spoilage modifier based on vermin severity.
///
/// Returns a multiplier (1.0 to 10.0).
#[must_use]
pub fn calculate_spoilage_modifier(vermin: &VerminState) -> f32 {
    // 0 severity -> 1.0x
    // 100 severity -> 10.0x (Massive spoilage)
    (vermin.severity / 100.0).mul_add(9.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{VerminState, calculate_spoilage_modifier, vermin_growth_system};
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::resources::ColonyResources;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(VerminState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());
        world
    }

    #[test]
    fn test_initialization() {
        let world = setup_world();
        let vermin = world.resource::<VerminState>();
        assert_eq!(vermin.severity, 0.0);
        assert_eq!(vermin.max_severity, 100.0);
    }

    #[test]
    fn test_vermin_growth_from_food() {
        let mut world = setup_world();

        // Add Food
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(
            vermin.severity > 0.0,
            "Vermin should grow when food is present"
        );
    }

    #[test]
    fn test_vermin_growth_from_waste() {
        let mut world = setup_world();

        // Add Waste
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.waste = 500.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(
            vermin.severity > 0.0,
            "Vermin should grow when waste is present"
        );
    }

    #[test]
    fn test_pest_control_policy_reduces_growth() {
        let mut world_normal = setup_world();
        let mut world_policy = setup_world();

        // Setup identical conditions
        {
            let mut res = world_normal.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }
        {
            let mut res = world_policy.resource_mut::<ColonyResources>();
            res.food = 1000.0;
            world_policy
                .resource_mut::<ColonyPolicies>()
                .toggle(Policy::PestControl);
        }

        // Run systems
        world_normal.run_system_once(vermin_growth_system).unwrap();
        world_policy.run_system_once(vermin_growth_system).unwrap();

        let severity_normal = world_normal.resource::<VerminState>().severity;
        let severity_policy = world_policy.resource::<VerminState>().severity;

        assert!(
            severity_policy < severity_normal,
            "Pest Control should reduce growth"
        );
    }

    #[test]
    fn test_vermin_decay_when_clean() {
        let mut world = setup_world();

        // Set initial infestation and clear resources
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.severity = 50.0;
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 0.0;
            res.waste = 0.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(
            vermin.severity < 50.0,
            "Vermin should die off without food/waste"
        );
    }

    #[test]
    fn test_spoilage_modifier() {
        let mut vermin = VerminState::default();

        // No vermin
        vermin.severity = 0.0;
        let mod_zero = calculate_spoilage_modifier(&vermin);
        assert!((mod_zero - 1.0).abs() < f32::EPSILON);

        // Max vermin
        vermin.severity = 100.0;
        let mod_max = calculate_spoilage_modifier(&vermin);
        assert!(mod_max > 1.0, "Spoilage should increase with vermin");
        assert!(
            mod_max >= 5.0,
            "Spoilage should be significant at max severity"
        );
    }

    #[test]
    fn test_spoilage_system_integration() {
        let mut world = setup_world();

        // Add food
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }

        // Set max vermin
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.severity = 100.0;
        }

        // Run spoilage system
        world
            .run_system_once(crate::layer1::spoilage::spoilage_system)
            .unwrap();

        let res = world.resource::<ColonyResources>();
        // Normal decay is 0.05% (0.0005) per tick
        // With max vermin modifier (10x), decay is 0.5% (0.005) per tick
        // 1000 * 0.005 = 5.0 decay. Food should be 995.0.
        // If it was normal, 1000 * 0.0005 = 0.5 decay -> 999.5.

        assert!(res.food < 999.0, "Food should decay faster with vermin");
        assert!(
            (res.food - 995.0).abs() < 0.1,
            "Decay should match modifier"
        );
    }
}
