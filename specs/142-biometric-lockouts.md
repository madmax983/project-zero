# 142: Biometric Lockouts

## Overview

Introduces granular access control for buildings (Doors, Airlocks, Gates). Players can restrict access to specific Pops, Roles (e.g., Militia, Engineer), or Factions. This adds security layers, creates "Staff Only" zones, and enables emerging narratives like "The Armory Riot" where unauthorized pops cannot access weapons.

## Dependencies

- `007` — Housing (Doors)
- `119` — Airlock & Pressure
- `004` — Pop Entity
- `067` — Militia System (Roles)

## RED Phase: Tests First

```rust
// src/layer1/access_control_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::access_control::{AccessControl, AccessMode, check_access};
    use crate::layer1::pop::{Pop, Role};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pathfinding::find_path_for_pop;
    use std::collections::HashSet;

    #[test]
    fn test_default_door_is_public() {
        let mut world = World::new();
        // Spawn Door with default AccessControl
        let door = world.spawn(AccessControl::default()).id();
        let pop = world.spawn(Pop::default()).id();

        assert!(check_access(&world, door, pop), "Default door should be accessible");
    }

    #[test]
    fn test_lockdown_blocks_everyone() {
        let mut world = World::new();
        let door = world.spawn(AccessControl {
            mode: AccessMode::Lockdown,
            ..Default::default()
        }).id();
        let pop = world.spawn(Pop::default()).id();

        assert!(!check_access(&world, door, pop), "Lockdown should block everyone");
    }

    #[test]
    fn test_biometric_allow_list() {
        let mut world = World::new();
        let pop_allowed = world.spawn(Pop::default()).id();
        let pop_denied = world.spawn(Pop::default()).id();

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
        let soldier = world.spawn((Pop::default(), Role::Soldier)).id();
        let civilian = world.spawn((Pop::default(), Role::Civilian)).id();

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
        // Setup simple map: Pop -> Door -> Target
        // If locked, path should fail or go around.
        // Assuming linear map for simplicity: 0 (Pop) - 1 (Door) - 2 (Target)

        let mut world = World::new();
        // ... setup map/terrain ...

        let pop = world.spawn(Pop::default()).id();

        // Locked Door at 1
        world.spawn((
            GridPosition { x: 1, y: 0 },
            AccessControl { mode: AccessMode::Lockdown, ..Default::default() }
        ));

        let path = find_path_for_pop(&world, (0, 0), (2, 0), pop);
        assert!(path.is_none(), "Path should be blocked by lockdown");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. AccessControl Component

```rust
// src/layer1/access_control.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessMode {
    #[default]
    Public,     // Open to all
    Restricted, // Checks allow lists
    Lockdown,   // Closed to all (except maybe Override hackers)
}

#[derive(Component, Default, Clone)]
pub struct AccessControl {
    pub mode: AccessMode,
    pub allowed_pops: HashSet<Entity>,
    pub allowed_roles: HashSet<Role>,
    // Future: allowed_factions
}

pub fn check_access(world: &World, door_entity: Entity, pop_entity: Entity) -> bool {
    let access = match world.get::<AccessControl>(door_entity) {
        Some(a) => a,
        None => return true, // No control = open
    };

    match access.mode {
        AccessMode::Public => true,
        AccessMode::Lockdown => false,
        AccessMode::Restricted => {
            // Check Pop ID
            if access.allowed_pops.contains(&pop_entity) {
                return true;
            }
            // Check Role
            if let Some(role) = world.get::<Role>(pop_entity) {
                if access.allowed_roles.contains(role) {
                    return true;
                }
            }
            false
        }
    }
}
```

### 2. Pathfinding Update

Update `find_path` to accept a context.

```rust
// src/layer1/pathfinding.rs

pub fn find_path_for_pop(world: &World, start: (i32, i32), end: (i32, i32), pop: Entity) -> Option<Vec<(i32, i32)>> {
    // ... setup ...
    // Pass `Some(pop)` to internal logic
}

fn is_walkable(..., accessor: Option<Entity>) -> bool {
    // ...
    // In step 3 (Check Building Type):
    if let Some(access_control) = building_access_map.get(&(x,y)) {
         if let Some(pop) = accessor {
             // We need to resolve the check here.
             // Optimization: Pre-calculate map of "Passable for this Pop" before A*?
             // Or look up component on pop entity every step? (Slower)
             // Better: Pass `&PopData` (roles, id) to `is_walkable`.
         } else {
             // No accessor = assume blocked if Restricted/Lockdown?
             // Or assume open? Usually UI pathfinding assumes open unless Lockdown.
         }
    }
    // ...
}
```

**Optimization Note**: `is_walkable` is called per node. Querying `World` inside it is slow.
*Approach*: `find_path` should gather the Pop's credentials (`Role`, `EntityId`) *once* at the start, and pass a lightweight `Credentials` struct to `is_walkable`.

```rust
struct AccessCredentials {
    entity: Entity,
    role: Option<Role>,
}
```

## REFACTOR Phase: Quality & Design

- **UI Integration**: The `Inspector` (Spec 091) needs a widget to configure `AccessControl` (Toggle Public/Restricted, Add/Remove allowed).
- **Visual Feedback**: Locked doors should have a red LED overlay; authorized doors green.
- **Hack Actions**: Spies or desperate pops could `Hack` a door to temporarily add themselves to the allow list.

## Acceptance Criteria

- [ ] `AccessControl` component exists.
- [ ] `check_access` logic correctly handles Public, Restricted, Lockdown.
- [ ] `find_path` respects `AccessControl` for specific pops.
- [ ] Pathfinding fails if the only route is through a locked door the pop cannot open.
- [ ] Existing `Gate` logic (if any) refactored to use `AccessControl`.

## Technical Guidance

- Replace `Gate.is_locked` with `AccessControl { mode: Lockdown }`.
- When spawning `Door` or `Airlock` buildings, attach `AccessControl::default()` (Public).
- Ensure `is_walkable` optimization: Don't do ECS lookups inside the hot loop. Extract `AccessControl` data into a `HashMap<(x,y), AccessControl>` (clone expensive?) or better, `HashMap<(x,y), &AccessControl>` references before the loop if possible, OR just the relevant data (Mode + Sets).
