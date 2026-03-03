# 146: Command Center & System Visibility

**Layer:** Cross-layer (1 -> 2)
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** You are the Commander, but you are not omniscient. Your view of the empire depends on your sensors. If the power goes out, you go blind.

**Mechanic:**
- Access to the **System View** (Layer 2) requires a functioning, powered **Command Center** building on Layer 1.
- Introduce `SystemVisibility` resource (enum: `Full`, `None`).
- If `SystemVisibility` is `None`, the player cannot switch to `ViewMode::System`.
- If the player is *already* in `System` view and visibility is lost (e.g., power outage), they are forced back to `Colony` view with a "SIGNAL LOST" notification.

**Why:** Connects Layer 1 infrastructure to Layer 2 capabilities. Adds tension to power management during crises.

---

## 2. Dependencies

- `094` System View Architecture (Layer 2 base)
- `004` Building System (Layer 1 buildings)
- `140` Thermal Management (Power Consumer logic)

---

## 3. RED Phase: Tests First

These tests define the behavior. They should fail until the Green phase is implemented.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer; // Assuming existing Power component
    use crate::layer2::system::{ViewMode, SystemVisibility, update_visibility_system, enforce_view_mode_system};

    #[test]
    fn test_visibility_defaults_to_none() {
        let mut world = World::new();
        world.init_resource::<SystemVisibility>();
        assert_eq!(*world.resource::<SystemVisibility>(), SystemVisibility::None);
    }

    #[test]
    fn test_powered_command_center_grants_visibility() {
        let mut world = World::new();
        world.init_resource::<SystemVisibility>();

        // Spawn a powered Command Center
        world.spawn((
            Building { building_type: BuildingType::CommandCenter },
            PowerConsumer { active: true, demand: 10.0, ..default() },
        ));

        // Run visibility update system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_visibility_system);
        schedule.run(&mut world);

        assert_eq!(*world.resource::<SystemVisibility>(), SystemVisibility::Full);
    }

    #[test]
    fn test_unpowered_command_center_denies_visibility() {
        let mut world = World::new();
        world.init_resource::<SystemVisibility>();

        // Spawn an unpowered Command Center
        world.spawn((
            Building { building_type: BuildingType::CommandCenter },
            PowerConsumer { active: false, demand: 10.0, ..default() },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_visibility_system);
        schedule.run(&mut world);

        assert_eq!(*world.resource::<SystemVisibility>(), SystemVisibility::None);
    }

    #[test]
    fn test_loss_of_visibility_forces_colony_view() {
        let mut world = World::new();
        world.insert_resource(SystemVisibility::None);
        world.insert_resource(ViewMode::System); // User is currently looking at system

        // Run enforcement system
        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_view_mode_system);
        schedule.run(&mut world);

        // Should be forced back to Colony view
        assert_eq!(*world.resource::<ViewMode>(), ViewMode::Colony);
    }
}
```

---

## 4. GREEN Phase: Minimal Implementation

### 1. Define Resource and Components

```rust
// src/layer2/visibility.rs

#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemVisibility {
    #[default]
    None,
    Full,
}

// Ensure BuildingType::CommandCenter is added to `src/layer1/building.rs` enum.
```

### 2. Implement Systems

```rust
// src/layer2/visibility.rs

use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::PowerConsumer;
use crate::layer2::system::ViewMode;

pub fn update_visibility_system(
    mut visibility: ResMut<SystemVisibility>,
    query: Query<(&Building, &PowerConsumer)>,
) {
    let mut has_active_cc = false;

    for (building, power) in &query {
        if building.building_type == BuildingType::CommandCenter && power.active {
            has_active_cc = true;
            break;
        }
    }

    *visibility = if has_active_cc {
        SystemVisibility::Full
    } else {
        SystemVisibility::None
    };
}

pub fn enforce_view_mode_system(
    visibility: Res<SystemVisibility>,
    mut view_mode: ResMut<ViewMode>,
    // mut notifications: ResMut<Notifications>, // Optional for MVP
) {
    if *view_mode == ViewMode::System && *visibility == SystemVisibility::None {
        *view_mode = ViewMode::Colony;
        // notifications.add("SIGNAL LOST: Command Center Offline");
    }
}
```

### 3. Integrate with Input

In `src/shared/input.rs`, modify the `Tab` key handler:

```rust
// ... inside handle_normal_mode ...
GameKeyCode::Tab => {
    let visibility = world.resource::<SystemVisibility>();
    let mut view_mode = world.resource_mut::<ViewMode>();

    match *view_mode {
        ViewMode::Colony => {
            if *visibility == SystemVisibility::Full {
                *view_mode = ViewMode::System;
            } else {
                // Optional: Play "Access Denied" sound or log message
            }
        },
        ViewMode::System => *view_mode = ViewMode::Colony,
    }
}
```

---

## 5. REFACTOR Phase: Quality & Design

### Refactoring Opportunities

1.  **Event-Driven Updates:**
    -   Instead of checking every tick in `update_visibility_system`, only check when `PowerConsumer` changes or `Building` is spawned/despawned.
    -   Use `Changed<PowerConsumer>` filter.

2.  **Partial Visibility:**
    -   Future expansion: `SystemVisibility::Partial` (Fog of War on map, but view accessible) if Command Center is damaged or low power.

3.  **UI Feedback:**
    -   Add a "Signal Strength" indicator to the UI.
    -   When trying to switch to System View and failing, show a toast notification "Command Center Required".

### API Improvements

-   `SystemVisibility` could be a struct with `level: f32` and `status: Enum`.
-   Integration with `Research` (e.g., "Advanced Sensors" tech required for Full visibility).

---

## 6. Acceptance Criteria

- [ ] `SystemVisibility` resource exists.
- [ ] `BuildingType::CommandCenter` is defined.
- [ ] Building a powered Command Center enables System View.
- [ ] Destroying or unpowering the Command Center disables System View.
- [ ] If in System View when visibility is lost, view forces back to Colony.
- [ ] Tests pass.

---

## 7. Technical Guidance

-   **Building Definition:** Add `CommandCenter` to `BuildingType` enum in `src/layer1/building.rs`.
-   **Cost:** Make it expensive (e.g., 50 Metal, 20 Electronics).
-   **Power:** High power demand (e.g., 50.0).
-   **Schedule:** Run `update_visibility_system` in `Update`, before `enforce_view_mode_system`.

---

## 8. Questions

-   *Builder: Should multiple Command Centers provide redundancy?*
    *Architect: Yes, as long as at least one Command Center is powered and staffed, global visibility is maintained.*
*Architect: Yes, as long as one is powered and staffed, global UI elements remain active.*
    -   *Architect:* Yes. The logic `has_active_cc` implicitly supports this (OR logic).
-   *Builder: Does the Command Center need to be staffed?*
    *Architect: Yes, a Command Center requires at least one Pop working an Administrator/Officer job to function.*
*Architect: Yes, it requires Admin pops to function, adding a labor cost to the global view.*
    -   *Architect:* For MVP, no. Just Power. Future versions may require `JobType::Operator`.
