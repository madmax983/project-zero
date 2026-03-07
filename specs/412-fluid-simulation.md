# 412-Fluid Simulation

## 1. Overview
**Layer:** 1
**Fantasy:** Harnessing the lifeblood of the planet. Digging canals, building dams, and causing accidental floods.
**Mechanic:** Water (and potentially magma) exists as a dynamic volume, not just static tiles. Fluids flow into lower or adjacent empty tiles. Pumps and floodgates allow control.
**Emergence:** You dig into a "dry" cavern only to puncture an aquifer, flooding your mines. You divert a river to create a defensive moat, but it dries up the forest downstream.
**Tension:** Access to water (farming/industry) vs. Risk of drowning (safety).

## 2. Dependencies
- Terrain system (must support fluid volumes per tile and flow mechanics).
- Physics/Update loop (fluid dynamics require iterative simulation).
- Building mechanics (dams, pumps, floodgates).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fluid_volume_exists_on_tile() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // ... setup terrain ...

        // Act
        // Spawn a tile with 10.0 units of water
        let tile = app.world_mut().spawn(Fluid { volume: 10.0, fluid_type: FluidType::Water }).id();
        app.update();

        // Assert
        // Check if fluid exists
        let fluid = app.world().get::<Fluid>(tile).unwrap();
        assert_eq!(fluid.volume, 10.0, "Fluid component should hold volume");
    }

    #[test]
    fn test_fluid_flows_to_adjacent_empty_tiles() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Let's assume a simplified grid or adjacencies.
        // We simulate a basic 1D or 2D cellular automata step
        let tile_a = app.world_mut().spawn((Fluid { volume: 10.0, fluid_type: FluidType::Water }, TilePos { x: 0, y: 0 })).id();
        let tile_b = app.world_mut().spawn((Fluid { volume: 0.0, fluid_type: FluidType::Water }, TilePos { x: 1, y: 0 })).id();

        // Act
        // Run flow system (1 step)
        app.add_systems(Update, fluid_flow_system);
        app.update();

        // Assert
        // Fluid should balance out
        let volume_a = app.world().get::<Fluid>(tile_a).unwrap().volume;
        let volume_b = app.world().get::<Fluid>(tile_b).unwrap().volume;
        assert!(volume_a < 10.0, "Fluid should leave origin tile");
        assert!(volume_b > 0.0, "Fluid should flow into adjacent empty tile");
    }

    #[test]
    fn test_dams_block_fluid_flow() {
         // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let tile_a = app.world_mut().spawn((Fluid { volume: 10.0, fluid_type: FluidType::Water }, TilePos { x: 0, y: 0 })).id();
        let tile_b = app.world_mut().spawn((Fluid { volume: 0.0, fluid_type: FluidType::Water }, TilePos { x: 1, y: 0 })).id();

        // Spawn a wall/dam between them or on one of them that blocks flow
        app.world_mut().entity_mut(tile_b).insert(Dam);

        // Act
        // Run flow system
        app.add_systems(Update, fluid_flow_system);
        app.update();

        // Assert
        let volume_a = app.world().get::<Fluid>(tile_a).unwrap().volume;
        let volume_b = app.world().get::<Fluid>(tile_b).unwrap().volume;
        assert_eq!(volume_a, 10.0, "Fluid should NOT flow if blocked by a Dam");
        assert_eq!(volume_b, 0.0, "Dam tile should not receive fluid");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct Fluid {
    pub volume: f32,
    pub fluid_type: FluidType,
}

#[derive(PartialEq, Clone, Copy)]
pub enum FluidType {
    Water,
    Magma,
}

#[derive(Component)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Dam;

pub fn fluid_flow_system(
    mut query: Query<(Entity, &mut Fluid, &TilePos, Option<&Dam>)>,
) {
    // Simple naive implementation just for the 1D test to pass.
    // In reality, this requires a cellular automata double-buffer or careful iteration.
    let mut transfers = Vec::new();

    // We assume the query order works for the test for now
    let mut tiles: Vec<_> = query.iter().collect();

    if tiles.len() >= 2 {
        let (e1, f1, p1, d1) = &tiles[0];
        let (e2, f2, p2, d2) = &tiles[1];

        if d2.is_none() && f1.volume > f2.volume {
            let diff = (f1.volume - f2.volume) / 2.0;
            transfers.push((*e1, *e2, diff));
        }
    }

    // Apply (Simulated here simply by running over query again)
    for (src, dst, amount) in transfers {
        if let Ok((_, mut f, _, _)) = query.get_mut(src) {
            f.volume -= amount;
        }
        if let Ok((_, mut f, _, _)) = query.get_mut(dst) {
            f.volume += amount;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - The `fluid_flow_system` must be rebuilt using a proper grid resource (e.g., `Grid<Fluid>`) rather than iterating over components via Query to handle adjacency performantly.
  - Implement a double-buffer system (Read volume -> Calculate Flow -> Write new volume) to prevent update-order artifacts.
  - Define "Pressure" vs "Gravity" if Z-levels exist.
- **Code Smells:**
  - The $O(N^2)$ or unsafe mutation logic in the naive implementation.
- **Performance Considerations:**
  - Cellular automata for fluids on a large grid can be very expensive. Consider chunking or only updating tiles that are 'active' (have fluid > 0 or are adjacent to fluid).
- **API Improvements:**
  - Separate `Fluid` into a global Grid resource rather than thousands of ECS entities to save overhead, while visual entities just read from the Grid.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Fluids hold volume per tile.
- [ ] Fluids equalize/flow to adjacent tiles.
- [ ] Dam/Wall structures correctly prevent fluid flow into or through their tiles.

## 7. Technical Guidance
- **Code Structure:** `src/layer1/fluid_sim.rs`.
- **Integration Points:**
  - Terrain Map: Store fluid volumes alongside terrain types.
  - Pathfinding: High water volume should block pathfinding or slow movement (or drown pops).
  - Rendering: Update visual representation based on fluid depth.
- **Gotchas:** Fluid conservation! Always ensure the total volume of fluid in the system remains constant unless explicitly destroyed (evaporation) or created (springs).

## 8. Questions
*Builder: add questions here if spec is unclear.*
