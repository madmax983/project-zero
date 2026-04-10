use crate::layer1::resources::ResourceType;
use crate::layer1::social::placebo::PlaceboProtocol;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Resource tracking active colony policies/edicts.
#[derive(Resource, Default, Debug, Clone)]
pub struct ColonyPolicies {
    /// Set of currently active policies.
    pub active_policies: HashSet<Policy>,
}

/// Available policies that can be enacted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Policy {
    /// Reduces hunger decay but lowers morale.
    Rationing,
    /// Increases work speed but lowers morale.
    DoubleShifts,
    /// Reduces vermin growth but lowers work speed.
    PestControl,
    /// Bans a specific resource, making it contraband.
    Prohibition(ResourceType),
    /// Blocks Remote Bonds and Intel gain from Subspace Pen Pals.
    FirewallComms,
    /// Issues a Placebo Protocol to temporarily reduce stress.
    Placebo(PlaceboProtocol),
    /// Censors delayed broadcasts, preventing large morale swings but increasing distrust.
    CensorBroadcasts,
}

impl ColonyPolicies {
    /// Toggles the state of a policy.
    pub fn toggle(&mut self, policy: Policy) {
        if self.active_policies.contains(&policy) {
            self.active_policies.remove(&policy);
        } else {
            self.active_policies.insert(policy);
        }
    }

    /// Checks if a policy is currently active.
    #[must_use]
    pub fn is_active(&self, policy: Policy) -> bool {
        self.active_policies.contains(&policy)
    }
}

/// Returns the modifier for hunger decay rate.
///
/// * `Rationing`: 0.5x decay.
#[must_use]
pub fn get_hunger_decay_modifier(policies: &ColonyPolicies) -> f32 {
    if policies.is_active(Policy::Rationing) {
        0.5
    } else {
        1.0
    }
}

/// Returns the modifier for work speed.
///
/// * `DoubleShifts`: 1.2x speed.
/// * `PestControl`: 0.95x speed.
#[must_use]
pub fn get_work_speed_modifier(policies: &ColonyPolicies) -> f32 {
    let mut modifier = 1.0;
    if policies.is_active(Policy::DoubleShifts) {
        modifier += 0.2;
    }
    if policies.is_active(Policy::PestControl) {
        modifier -= 0.05;
    }
    modifier
}

/// Returns the flat morale modifier.
///
/// * `Rationing`: -0.1 Morale.
/// * `DoubleShifts`: -0.15 Morale.
#[must_use]
pub fn get_morale_modifier(policies: &ColonyPolicies) -> f32 {
    let mut modifier = 0.0;
    if policies.is_active(Policy::Rationing) {
        modifier -= 0.1;
    }
    if policies.is_active(Policy::DoubleShifts) {
        modifier -= 0.15;
    }
    modifier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::GridPosition;

    #[test]
    fn test_policies_resource_defaults() {
        let policies = ColonyPolicies::default();
        assert!(policies.active_policies.is_empty());
    }

    #[test]
    fn test_toggle_policy() {
        let mut policies = ColonyPolicies::default();

        // Enable Rationing
        policies.toggle(Policy::Rationing);
        assert!(
            policies.is_active(Policy::Rationing),
            "Rationing should be active after toggle"
        );

        // Disable Rationing
        policies.toggle(Policy::Rationing);
        assert!(
            !policies.is_active(Policy::Rationing),
            "Rationing should be inactive after toggle"
        );
    }

    #[test]
    fn test_rationing_reduces_hunger_decay() {
        let mut world = World::new();
        let mut policies = ColonyPolicies::default();
        policies.toggle(Policy::Rationing);
        world.insert_resource(policies);

        // Spawn a pop with standard needs
        let _pop = world
            .spawn((
                Needs {
                    hunger: 1.0,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let modifier =
            crate::layer1::edicts::get_hunger_decay_modifier(world.resource::<ColonyPolicies>());
        assert!(
            modifier < 1.0,
            "Hunger decay modifier should be < 1.0 with Rationing"
        );
        assert!(
            (modifier - 0.5).abs() < f32::EPSILON,
            "Hunger decay should be 0.5"
        );
    }

    #[test]
    fn test_double_shifts_increases_speed_and_reduces_morale() {
        let mut world = World::new();
        let mut policies = ColonyPolicies::default();
        policies.toggle(Policy::DoubleShifts);
        world.insert_resource(policies);

        // Check modifiers
        let speed_mod =
            crate::layer1::edicts::get_work_speed_modifier(world.resource::<ColonyPolicies>());
        assert!(
            speed_mod > 1.0,
            "Work speed modifier should be > 1.0 with DoubleShifts"
        );

        let morale_mod =
            crate::layer1::edicts::get_morale_modifier(world.resource::<ColonyPolicies>());
        assert!(
            morale_mod < 0.0,
            "Morale modifier should be negative with DoubleShifts"
        );
    }

    #[test]
    fn test_pest_control_reduces_work_speed() {
        let mut policies = ColonyPolicies::default();
        policies.toggle(Policy::PestControl);
        // Should reduce speed by 0.05
        let modifier = get_work_speed_modifier(&policies);
        assert!((modifier - 0.95).abs() < f32::EPSILON);
    }
}
