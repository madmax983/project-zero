# 133 Ventilation Networks

## 1. Overview

Control the flow of atmosphere and pollution through your colony using Ventilation Networks. While Walls block both movement and airflow, Vents allow gases to pass freely while maintaining physical security against larger threats.

**Why:**
- Adds strategic depth to base layout (managing airflow vs security).
- Fixes the issue where pollution diffuses through solid walls.
- Enables new gameplay for small entities (Vermin, Drones) infiltrating through vents.

## 2. Dependencies

- `063` Atmospheric Simulation (Implemented)
- `119` Airlock & Pressure (Implemented)
- `073` Vermin Infestation (Implemented)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::atmosphere::AtmosphereGrid;
    use crate::layer1::pressure::PressureGrid;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::{Pop, PopState};
    use crate::layer1::vermin::Vermin;
    use bevy_ecs::prelude::*;

    // --- Pressure Tests ---

    #[test]
    fn test_pressure_blocked_by_wall() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(0, 0, 1.0); // Source
        world.insert_resource(grid);

        // Wall at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
        ));

        // Run pressure update
        crate::layer1::pressure::update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(2, 0) < 0.01, "Pressure should not pass through Wall");
    }

    #[test]
    fn test_pressure_passes_through_vent() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(0, 0, 1.0); // Source
        world.insert_resource(grid);

        // Vent at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Vent },
            GridPosition { x: 1, y: 0 },
        ));

        // Run pressure update multiple times to allow diffusion
        for _ in 0..5 {
            crate::layer1::pressure::update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(2, 0) > 0.1, "Pressure SHOULD pass through Vent");
    }

    // --- Pollution Tests (AtmosphereGrid) ---

    #[test]
    fn test_pollution_blocked_by_wall() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(5, 1);
        grid.set(0, 0, 1.0); // Source
        world.insert_resource(grid);

        // Wall at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
        ));

        // Run atmosphere update
        crate::layer1::atmosphere::update_atmosphere_system(&mut world);

        let grid = world.resource::<AtmosphereGrid>();
        assert!(grid.get(2, 0) < 0.01, "Pollution should NOT pass through Wall");
    }

    #[test]
    fn test_pollution_passes_through_vent() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(5, 1);
        grid.set(0, 0, 1.0); // Source
        world.insert_resource(grid);

        // Vent at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Vent },
            GridPosition { x: 1, y: 0 },
        ));

        // Run atmosphere update
        for _ in 0..5 {
            crate::layer1::atmosphere::update_atmosphere_system(&mut world);
        }

        let grid = world.resource::<AtmosphereGrid>();
        assert!(grid.get(2, 0) > 0.1, "Pollution SHOULD pass through Vent");
    }

    // --- Movement Tests ---

    #[test]
    fn test_vent_blocks_pop_movement() {
        let mut world = World::new();
        // Setup simple map
        // ... (Grid setup) ...

        // Vent at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Vent },
            GridPosition { x: 1, y: 0 },
        ));

        // Pop at (0, 0) trying to move to (2, 0)
        // Check pathfinding
        let path = crate::layer1::pathfinding::find_path(&world, (0,0), (2,0));
        assert!(path.is_none(), "Pop should not path through Vent");
    }

    #[test]
    fn test_vermin_passes_through_vent() {
        let mut world = World::new();
        // Setup simple map
        // ...

        // Vent at (1, 0)
        world.spawn((
            Building { building_type: BuildingType::Vent },
            GridPosition { x: 1, y: 0 },
        ));

        // Vermin at (0, 0)
        // Check pathfinding with Vermin capability
        // This requires pathfinding to accept an entity or capability flag
        let path = crate::layer1::pathfinding::find_path_for_entity(&world, (0,0), (2,0), &Vermin);
        assert!(path.is_some(), "Vermin SHOULD path through Vent");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Add `Vent` Building
Update `src/layer1/building.rs`:
```rust
pub enum BuildingType {
    // ...
    Vent,
}

// In impl BuildingType:
// label() -> "Vent"
// char() -> '≡' (Identical to 3 lines) or '='
// cost() -> Metal: 5.0
// is_obstacle() -> true (Blocks standard movement)
```

### 2. Update `PressureGrid` Logic
Update `src/layer1/pressure.rs`:
- In `update_pressure_system`, add `BuildingType::Vent` to the blockers map with transmissivity `1.0` (or `0.9` for slight resistance).

### 3. Update `AtmosphereGrid` Logic (Pollution)
Update `src/layer1/atmosphere.rs`:
- Modify `AtmosphereGrid::diffuse` to accept a `blockers` HashMap, similar to `PressureGrid`.
- In `update_atmosphere_system`, query buildings to build the `blockers` map (checking for Wall, Airlock, Gate, Vent).
- Pass blockers to `diffuse`.
- Ensure `Vent` allows pollution, `Wall` blocks it.

### 4. Update Pathfinding for Vermin
Update `src/layer1/pathfinding.rs` (or `navigation.rs`):
- Add a way to check if an entity can traverse a tile.
- Standard Pops: Blocked by `is_obstacle()`.
- Vermin: Blocked by `is_obstacle()` UNLESS it is `BuildingType::Vent`.
- This might require a trait `CanUseVents` or simply checking the entity type in the A* cost function.

## 5. REFACTOR Phase

- **Shared Blocker Logic**: Both `PressureGrid` and `AtmosphereGrid` build a "blockers" map every tick. Refactor this into a `FlowBlockerMap` resource that is updated only when buildings change (via `Added<Building>` or `Removed<Building>` events/queries), or just unify the query logic into a shared function.
- **Combined Atmosphere**: Eventually, `Pressure` and `Pollution` could be layers of a single `Atmosphere` struct to avoid duplicating diffusion code.

## 6. Acceptance Criteria

- [ ] `PressureGrid` correctly blocked by Walls, passed by Vents.
- [ ] `AtmosphereGrid` (Pollution) correctly blocked by Walls, passed by Vents.
- [ ] `Vent` building is constructible.
- [ ] Pops cannot path through Vents.
- [ ] Vermin CAN path through Vents.
- [ ] Existing tests pass.

## 7. Technical Guidance

- **Pathfinding**: The current pathfinding likely uses a simple boolean `is_walkable`. You may need to extend `find_path` to `find_path_with_capabilities` or pass a closure `F: Fn(Pos) -> bool` for walkability.
- **Performance**: Building the blocker map every tick is O(N) where N is buildings. For 1000 buildings, this is fast. Don't over-optimize yet.
- **Visuals**: Vents should look distinct. Maybe use a specific character like `#` but lighter color, or `≡`.

## Questions
- *Builder: Should Vents conduct temperature?*
  *Architect:* Future scope: Yes, but for now focus on Pressure/Pollution.
