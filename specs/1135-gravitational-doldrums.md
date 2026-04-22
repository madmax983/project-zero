# 1135: Gravitational Doldrums

## 1. Overview

In the System Layer (Layer 2), "Gravitational Doldrums" represent zones of space where standard reaction drives are highly ineffective due to gravitational interference or null-gravity pockets. Ships moving through these zones operate at 10% efficiency. Specialized "Tug" ships or careful gravity assists are needed to navigate them efficiently. This creates strategic terrain in space combat and trade.

## 2. Dependencies

- Layer 2 Fleet movement system (basic movement components).
- Layer 2 Node/System mapping (ability to define regions in a system).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    // 1. Test that ships in Doldrums move slowly
    #[test]
    fn test_doldrums_reduce_movement_speed() {
        // Arrange: Setup World with Doldrums region and a basic Ship
        let mut app = App::new();
        // Setup ...

        // Act: Run movement system

        // Assert: Ship speed multiplier is 0.1
    }

    // 2. Test that Tug ships are immune or less affected
    #[test]
    fn test_tug_ships_ignore_doldrums() {
        // Arrange: Ship with Tug component in Doldrums

        // Act: Run movement system

        // Assert: Ship speed multiplier is normal
    }

    // 3. Test that fleets being towed move at Tug speed
    #[test]
    fn test_towed_fleets_move_at_tug_speed() {
        // Arrange: Normal ship towed by Tug ship in Doldrums

        // Act: Run movement system

        // Assert: Normal ship moves at normal speed
    }

    // 4. Test pathfinding cost avoids doldrums unless necessary
    #[test]
    fn test_pathfinding_avoids_doldrums() {
        // Arrange: Pathfinding graph with clear path and shorter Doldrums path

        // Act: Calculate path

        // Assert: The clear (but physically longer) path is chosen
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation

#[derive(Component)]
pub struct GravitationalDoldrums {
    pub penalty_multiplier: f32, // typically 0.1
}

#[derive(Component)]
pub struct TugShip {
    pub tow_capacity: u32,
}

#[derive(Component)]
pub struct TowedBy {
    pub tug_entity: Entity,
}

// Intercept movement speed calculation
pub fn calculate_fleet_speed(
    base_speed: f32,
    in_doldrums: bool,
    is_tug: bool,
    is_towed: bool,
) -> f32 {
    if in_doldrums && !is_tug && !is_towed {
        return base_speed * 0.1;
    }
    base_speed
}
```

## 5. REFACTOR Phase: Quality & Design

- **Pathfinding AI**: Ensure AI opponents know to avoid Doldrums unless they are using Tugs or are desperate.
- **Combat**: If two fleets meet in Doldrums, combat rounds might last longer or evasion might be severely penalized due to lack of maneuverability.
- **UI**: Doldrums need a distinct visual representation on the system map so players don't accidentally send ships into a trap.
- **Towing Logic**: Ensure the "TowedBy" link breaks if the Tug is destroyed.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Fleets without Tugs move at 10% speed through marked Doldrums zones.

## 7. Technical Guidance

- Doldrums should be implemented as an environmental aura or a spatial grid depending on how Layer 2 maps are structured.
- Make sure "TowedBy" uses `bevy_ecs` Entity relationships or hierarchical transforms correctly if fleet positions are linked.
- The pathfinding weights must dynamically reflect the 10x movement time penalty.

## 8. Questions

*Builder: add questions here if spec is unclear.*
