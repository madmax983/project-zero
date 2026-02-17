use bevy_ecs::prelude::*;
use crate::layer1::pop::Role;
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
/// - The building has no `AccessControl` component.
/// - The mode is `Public`.
/// - The mode is `Restricted` AND the pop is in `allowed_pops` OR has an allowed `Role`.
///
/// Returns `false` if:
/// - The mode is `Lockdown`.
/// - The mode is `Restricted` AND the pop is NOT allowed.
pub fn check_access(world: &World, door_entity: Entity, pop_entity: Entity) -> bool {
    let Some(access) = world.get::<AccessControl>(door_entity) else {
        return true; // No control = open
    };

    match access.mode {
        AccessMode::Public => true,
        AccessMode::Lockdown => false,
        AccessMode::Restricted => {
            // Check specific Pop allow list
            if access.allowed_pops.contains(&pop_entity) {
                return true;
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

        assert!(check_access(&world, door, pop), "Default door should be accessible");
    }

    #[test]
    fn test_lockdown_blocks_everyone() {
        let mut world = World::new();
        let door = world.spawn(AccessControl {
            mode: AccessMode::Lockdown,
            ..Default::default()
        }).id();
        let pop = world.spawn(Pop).id();

        assert!(!check_access(&world, door, pop), "Lockdown should block everyone");
    }

    #[test]
    fn test_biometric_allow_list() {
        let mut world = World::new();
        let pop_allowed = world.spawn(Pop).id();
        let pop_denied = world.spawn(Pop).id();

        let mut allowed = HashSet::new();
        allowed.insert(pop_allowed);

        let door = world.spawn(AccessControl {
            mode: AccessMode::Restricted,
            allowed_pops: allowed,
            ..Default::default()
        }).id();

        assert!(check_access(&world, door, pop_allowed), "Allowed pop should pass");
        assert!(!check_access(&world, door, pop_denied), "Denied pop should fail");
    }

    #[test]
    fn test_role_based_access() {
        let mut world = World::new();
        let soldier = world.spawn((Pop, Role::Soldier)).id();
        let civilian = world.spawn((Pop, Role::Civilian)).id();

        let mut allowed_roles = HashSet::new();
        allowed_roles.insert(Role::Soldier);

        let door = world.spawn(AccessControl {
            mode: AccessMode::Restricted,
            allowed_roles,
            ..Default::default()
        }).id();

        assert!(check_access(&world, door, soldier), "Soldier should pass");
        assert!(!check_access(&world, door, civilian), "Civilian should fail");
    }

    #[test]
    fn test_pathfinding_integration() {
        use crate::layer1::map::GridPosition;
        use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
        use crate::layer1::pathfinding::find_path_for_pop;
        use crate::layer1::terrain::{TerrainGrid, TerrainType};

        let mut world = World::new();
        // Setup Map
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(OccupiedTiles::default());

        let pop = world.spawn(Pop).id();

        // Create a choke point at (1, 0)
        // Block (1, 1) with a Wall
        world.spawn((
            GridPosition { x: 1, y: 1 },
            Building { building_type: BuildingType::Wall },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 1));

        // Since it's a 10x10 grid, we also need to block going around the other way or assume start/end are constrained.
        // Actually, let's just surround the start point (0,0).
        // Block (0, 1) and (1, 1).
        // And (1, 0) is the Gate.

        // Wall at (0, 1)
        world.spawn((
            GridPosition { x: 0, y: 1 },
            Building { building_type: BuildingType::Wall },
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((0, 1));

        // Locked Door at (1, 0)
        world.spawn((
            GridPosition { x: 1, y: 0 },
            AccessControl { mode: AccessMode::Lockdown, ..Default::default() },
            Building { building_type: BuildingType::Gate }, // Gate is an obstacle
        ));
        world.resource_mut::<OccupiedTiles>().0.insert((1, 0));

        let path = find_path_for_pop(&world, (0, 0), (2, 0), pop);
        assert!(path.is_none(), "Path should be blocked by lockdown");
    }
}
