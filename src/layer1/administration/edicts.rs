use crate::layer1::resources::ResourceType;
use crate::layer1::social::placebo::PlaceboProtocol;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Resource tracking active colony policies/edicts.
#[derive(Resource, Default, Debug, Clone)]
pub struct ColonyPolicies {
    /// Set of currently active policies.
    pub active_policies: HashSet<Policy>,
    /// Set of policies that have become orphaned and cannot be normally removed.
    pub orphaned_policies: HashSet<Policy>,
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
    /// Automated systems target and destroy any infected individuals.
    ShootInfected,
}

#[derive(Event, Debug)]
pub struct TogglePolicyEvent(pub Policy);

#[derive(Event, Debug)]
pub struct AccessDeniedEvent {
    pub reason: String,
}

#[derive(Event, Debug)]
pub struct HackCentralHubEvent {
    pub target_policy: Policy,
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

pub fn handle_policy_toggle_system(
    mut events: EventReader<TogglePolicyEvent>,
    mut policies: ResMut<ColonyPolicies>,
    mut access_denied: EventWriter<AccessDeniedEvent>,
) {
    for ev in events.read() {
        let policy = ev.0;
        if policies.orphaned_policies.contains(&policy) {
            access_denied.send(AccessDeniedEvent {
                reason: format!("Edict {:?} is orphaned and cannot be toggled", policy),
            });
        } else {
            policies.toggle(policy);
        }
    }
}

pub fn handle_hack_hub_system(
    mut events: EventReader<HackCentralHubEvent>,
    mut policies: ResMut<ColonyPolicies>,
) {
    for ev in events.read() {
        let policy = ev.target_policy;
        policies.orphaned_policies.remove(&policy);
        policies.active_policies.remove(&policy);
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

    #[test]
    fn test_player_cannot_rescind_orphaned_edict() {
        // Arrange: Player attempts to toggle the edict off
        let mut app = bevy_app::App::new();
        app.add_event::<TogglePolicyEvent>();
        app.add_event::<AccessDeniedEvent>();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::ShootInfected);
        policies.orphaned_policies.insert(Policy::ShootInfected);
        app.insert_resource(policies);

        app.add_systems(bevy_app::Update, handle_policy_toggle_system);

        // Act: Send UI/Input event to disable the edict
        app.world_mut()
            .send_event(TogglePolicyEvent(Policy::ShootInfected));
        app.update();

        // Assert: The edict remains active, and an 'AccessDenied' event is logged.
        let policies = app.world().resource::<ColonyPolicies>();
        assert!(policies.is_active(Policy::ShootInfected));

        let events = app.world().resource::<Events<AccessDeniedEvent>>();
        let mut cursor = events.get_cursor();
        assert!(cursor.read(events).next().is_some());
    }

    #[test]
    fn test_resolving_orphaned_edict_via_bureaucratic_hack() {
        // Arrange: App with orphaned edict
        let mut app = bevy_app::App::new();
        app.add_event::<HackCentralHubEvent>();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::ShootInfected);
        policies.orphaned_policies.insert(Policy::ShootInfected);
        app.insert_resource(policies);

        app.add_systems(bevy_app::Update, handle_hack_hub_system);

        // Act: Perform a 'HackCentralHub' action
        app.world_mut().send_event(HackCentralHubEvent {
            target_policy: Policy::ShootInfected,
        });
        app.update();

        // Assert: The edict is finally removed from the active edicts list.
        let policies = app.world().resource::<ColonyPolicies>();
        assert!(!policies.is_active(Policy::ShootInfected));
        assert!(!policies.orphaned_policies.contains(&Policy::ShootInfected));
    }
}
