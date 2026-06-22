//! Access Control system for doors and airlocks.
//!
//! # Context
//! This module defines access restrictions for buildings like Doors, Airlocks, and Gates.
//! It allows locking down areas to specific pops or roles (e.g., only Soldiers can enter the Armory).
//!
//! # Usage
//! ```rust
//! use bevy_ecs::prelude::*;
//! use scale::layer1::access_control::{AccessControl, AccessMode, check_access};
//! use scale::layer1::pop::{Pop, Role};
//! use std::collections::HashSet;
//!
//! let mut world = World::new();
//! let soldier = world.spawn((Pop, Role::Soldier)).id();
//!
//! let mut allowed_roles = HashSet::new();
//! allowed_roles.insert(Role::Soldier);
//!
//! let door = world.spawn(AccessControl {
//!     mode: AccessMode::Restricted,
//!     allowed_roles,
//!     ..Default::default()
//! }).id();
//!
//! assert!(check_access(&world, door, soldier));
//! ```
//!
//! # Details
//! - [`AccessMode::Public`]: Open to everyone.
//! - [`AccessMode::Restricted`]: Checks `allowed_pops` and `allowed_roles`.
//! - [`AccessMode::Lockdown`]: Blocks everyone.
//!   Pathfinding automatically respects these rules.
//!
//! # Links
//! - [`AccessControl`]
//! - [`check_access`]
use crate::layer1::pop::Role;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Defines the access level of a building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessMode {
    /// Open to all pops.
    #[default]
    Public,
    /// Restricted to specific pops or roles.
    Restricted,
    /// Closed to all pops (except overrides).
    Lockdown,
}

/// Component that controls access to a building (Door, Airlock, Gate).
#[derive(Component, Default, Clone)]
pub struct AccessControl {
    /// The current access mode.
    pub mode: AccessMode,
    /// Set of specific pops allowed access.
    pub allowed_pops: HashSet<Entity>,
    /// Set of roles allowed access.
    pub allowed_roles: HashSet<Role>,
}

/// Checks if a pop is allowed to access a building.
///
/// Returns `true` if:
/// - The building has no [`AccessControl`] component.
/// - The mode is [`AccessMode::Public`].
/// - The mode is [`AccessMode::Restricted`] AND the pop is in `allowed_pops` OR has an allowed [`crate::layer1::pop::Role`].
///
/// Returns `false` if:
/// - The mode is [`AccessMode::Lockdown`].
/// - The mode is [`AccessMode::Restricted`] AND the pop is NOT allowed.
///
/// # Examples
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer1::access_control::{AccessControl, check_access};
/// use scale::layer1::pop::Pop;
///
/// let mut world = World::new();
/// let pop = world.spawn(Pop).id();
/// let door = world.spawn(AccessControl::default()).id();
///
/// assert!(check_access(&world, door, pop));
/// ```
pub fn check_access(world: &World, door_entity: Entity, pop_entity: Entity) -> bool {
    if crate::layer1::security::check_security_clearance(world, pop_entity, door_entity)
        == crate::layer1::security::AccessResult::DeniedDrift
    {
        return false;
    }

    let Some(access) = world.get::<AccessControl>(door_entity) else {
        return true; // No control = open
    };

    match access.mode {
        AccessMode::Public => true,
        AccessMode::Lockdown => false,
        AccessMode::Restricted => {
            // Check specific Pop allow list
            if access.allowed_pops.contains(&pop_entity) {
                // Verify entity is alive to prevent ID reuse exploits or logic bugs
                if world.get_entity(pop_entity).is_ok() {
                    return true;
                }
            }
            // Check Role allow list
            if world
                .get::<Role>(pop_entity)
                .is_some_and(|role| access.allowed_roles.contains(role))
            {
                return true;
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Pop, Role};
    use std::collections::HashSet;

    #[test]
    fn test_default_door_is_public() {
        let mut world = World::new();
        // Spawn Door with default AccessControl
        let door = world.spawn(AccessControl::default()).id();
        let pop = world.spawn(Pop).id();

        assert!(
            check_access(&world, door, pop),
            "Default door should be accessible"
        );
    }

    #[test]
    fn test_lockdown_blocks_everyone() {
        let mut world = World::new();
        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Lockdown,
                ..Default::default()
            })
            .id();
        let pop = world.spawn(Pop).id();

        assert!(
            !check_access(&world, door, pop),
            "Lockdown should block everyone"
        );
    }

    #[test]
    fn test_biometric_allow_list() {
        let mut world = World::new();
        let pop_allowed = world.spawn(Pop).id();
        let pop_denied = world.spawn(Pop).id();

        let mut allowed = HashSet::new();
        allowed.insert(pop_allowed);

        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Restricted,
                allowed_pops: allowed,
                ..Default::default()
            })
            .id();

        assert!(
            check_access(&world, door, pop_allowed),
            "Allowed pop should pass"
        );
        assert!(
            !check_access(&world, door, pop_denied),
            "Denied pop should fail"
        );
    }

    #[test]
    fn test_check_security_clearance_drift_denies_access() {
        use crate::layer1::security::{BiometricProfile, SecurityTerminal};

        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 1.0,
                    ..Default::default()
                },
            ))
            .id();
        let door = world
            .spawn((
                AccessControl::default(),
                SecurityTerminal {
                    strictness: 1.0,
                    required_clearance: 0,
                },
            ))
            .id();

        assert!(
            !check_access(&world, door, pop),
            "High drift should be denied access"
        );
    }

    #[test]
    fn test_check_security_clearance_delayed() {
        use crate::layer1::security::{BiometricProfile, SecurityTerminal};

        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                BiometricProfile {
                    drift: 0.6,
                    ..Default::default()
                },
            ))
            .id();
        let door = world
            .spawn((
                AccessControl::default(),
                SecurityTerminal {
                    strictness: 1.0,
                    required_clearance: 0,
                },
            ))
            .id();

        assert!(
            check_access(&world, door, pop),
            "Delayed access is not denied in check_access"
        );
    }

    #[test]
    fn test_check_security_clearance_no_access_control_allowed() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();
        let door = world.spawn_empty().id();

        assert!(
            check_access(&world, door, pop),
            "No AccessControl should mean open"
        );
    }

    #[test]
    fn test_check_security_clearance_restricted_allowed_pop_alive() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        let mut allowed = HashSet::new();
        allowed.insert(pop);

        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Restricted,
                allowed_pops: allowed,
                ..Default::default()
            })
            .id();

        assert!(
            check_access(&world, door, pop),
            "Allowed alive pop should pass"
        );
    }

    #[test]
    fn test_check_security_clearance_restricted_not_allowed() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        let allowed = HashSet::new();

        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Restricted,
                allowed_pops: allowed,
                ..Default::default()
            })
            .id();

        assert!(
            !check_access(&world, door, pop),
            "Pop not in allowed_pops should fail"
        );
    }

    #[test]
    fn test_check_security_clearance_restricted_allowed_pop_dead_entity() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        let mut allowed = HashSet::new();
        allowed.insert(pop);

        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Restricted,
                allowed_pops: allowed,
                ..Default::default()
            })
            .id();

        world.despawn(pop);

        assert!(
            !check_access(&world, door, pop),
            "Allowed dead pop should fail"
        );
    }

    #[test]
    fn test_role_based_access() {
        let mut world = World::new();
        let soldier = world.spawn((Pop, Role::Soldier)).id();
        let civilian = world.spawn((Pop, Role::Civilian)).id();

        let mut allowed_roles = HashSet::new();
        allowed_roles.insert(Role::Soldier);

        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Restricted,
                allowed_roles,
                ..Default::default()
            })
            .id();

        assert!(check_access(&world, door, soldier), "Soldier should pass");
        assert!(
            !check_access(&world, door, civilian),
            "Civilian should fail"
        );
    }

    #[test]
    fn test_check_security_clearance_restricted_allowed_role_not_found() {
        let mut world = World::new();
        let civilian = world.spawn((Pop, Role::Civilian)).id();

        let mut allowed = HashSet::new();
        allowed.insert(Role::Soldier);

        // The civilian is not in the allowed_roles set.
        let door = world
            .spawn(AccessControl {
                mode: AccessMode::Restricted,
                allowed_roles: allowed,
                ..Default::default()
            })
            .id();

        assert!(
            !check_access(&world, door, civilian),
            "Civilian should not be allowed"
        );
    }
}
