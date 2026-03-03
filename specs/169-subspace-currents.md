# 169: Subspace Currents

## 1. Overview

Adds a "terrain" equivalent to the System Map (Layer 2) in the form of **Subspace Currents**. These are invisible (or visualized via UI overlay) vectors that influence ship travel.

**Why:**
- Adds strategic depth to fleet movement (Layer 2).
- Creates "fast lanes" and "mud bogs" in space.
- Encourages route planning rather than straight-line travel.

**Mechanics:**
- The System Map has a `SubspaceGrid` of vector values (`Vec2`).
- Moving *with* the current reduces fuel cost / travel time.
- Moving *against* the current increases fuel cost / travel time.
- Moving perpendicular has neutral effect.

## 2. Dependencies

- `099` Fleet Movement (Conceptually, though this spec can be implemented as the foundational logic for movement costs).
- `094` System View Architecture.

## 3. RED Phase: Tests First

Write these tests in `src/layer2/subspace_tests.rs`.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use glam::Vec2;
    use crate::layer2::subspace::{SubspaceGrid, calculate_movement_cost};

    #[test]
    fn test_subspace_grid_initialization() {
        let grid = SubspaceGrid::new(10, 10);
        // Default should be zero vectors
        assert_eq!(grid.get_current(0, 0), Vec2::ZERO);
        assert_eq!(grid.width, 10);
    }

    #[test]
    fn test_movement_cost_neutral() {
        let mut grid = SubspaceGrid::new(10, 10);
        // No current
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(1.0, 0.0);

        let cost = calculate_movement_cost(&grid, start, end);
        // Base cost is distance (1.0)
        assert!((cost - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_movement_cost_with_current() {
        let mut grid = SubspaceGrid::new(10, 10);
        // Current flowing East (1.0, 0.0)
        grid.set_current(0, 0, Vec2::new(1.0, 0.0));

        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(1.0, 0.0); // Moving East (With current)

        let cost = calculate_movement_cost(&grid, start, end);
        // Should be cheaper than 1.0
        assert!(cost < 1.0);
    }

    #[test]
    fn test_movement_cost_against_current() {
        let mut grid = SubspaceGrid::new(10, 10);
        // Current flowing East
        grid.set_current(0, 0, Vec2::new(1.0, 0.0));

        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(-1.0, 0.0); // Moving West (Against current)

        let cost = calculate_movement_cost(&grid, start, end);
        // Should be more expensive than distance (1.0)
        assert!(cost > 1.0);
    }

    #[test]
    fn test_movement_cost_perpendicular() {
        let mut grid = SubspaceGrid::new(10, 10);
        // Current flowing East
        grid.set_current(0, 0, Vec2::new(1.0, 0.0));

        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(0.0, 1.0); // Moving North (Perpendicular)

        let cost = calculate_movement_cost(&grid, start, end);
        // Should be roughly neutral (1.0)
        assert!((cost - 1.0).abs() < 0.01);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define `SubspaceGrid` Resource

In `src/layer2/subspace.rs`:

```rust
use bevy_ecs::prelude::*;
use glam::Vec2;

#[derive(Resource, Debug, Clone)]
pub struct SubspaceGrid {
    pub width: usize,
    pub height: usize,
    vectors: Vec<Vec2>,
}

impl SubspaceGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vectors: vec![Vec2::ZERO; width * height],
        }
    }

    pub fn get_current(&self, x: i32, y: i32) -> Vec2 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return Vec2::ZERO;
        }
        self.vectors[y as usize * self.width + x as usize]
    }

    pub fn set_current(&mut self, x: i32, y: i32, vector: Vec2) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.vectors[y as usize * self.width + x as usize] = vector;
    }
}
```

### 2. Implement `calculate_movement_cost`

In `src/layer2/subspace.rs`:

```rust
pub fn calculate_movement_cost(grid: &SubspaceGrid, start: Vec2, end: Vec2) -> f32 {
    let distance = start.distance(end);
    if distance < f32::EPSILON {
        return 0.0;
    }

    let direction = (end - start).normalize();

    // Sample current at start position (simplification for MVP)
    // In full implementation, perform raycast sampling.
    let grid_x = start.x.round() as i32;
    let grid_y = start.y.round() as i32;
    let current = grid.get_current(grid_x, grid_y);

    // Dot product:
    // 1.0 = With current (Reduce cost)
    // -1.0 = Against current (Increase cost)
    // 0.0 = Perpendicular (No change)
    let alignment = current.dot(direction);

    // Configurable strength
    let current_strength = 0.5;

    // Cost Multiplier:
    // Alignment 1.0 -> Cost 0.5
    // Alignment -1.0 -> Cost 1.5
    let modifier = 1.0 - (alignment * current_strength);

    distance * modifier.max(0.1) // Minimum cost floor
}
```

## 5. REFACTOR Phase: Quality & Design

-   **Interpolation**: Instead of sampling the nearest grid cell, use bilinear interpolation for `get_current` at `Vec2` coordinates.
-   **Path Sampling**: For long movements, sample multiple points along the line segment to get an average current effect.
-   **Visualization**: Add a system to visualize currents as arrows in the System View UI.

## 6. Acceptance Criteria

- [ ] `SubspaceGrid` resource exists and can store vectors.
- [ ] `calculate_movement_cost` correctly modifies cost based on alignment.
- [ ] Tests pass.
- [ ] Code is in `src/layer2/` (or `src/system/` if architecture differs).

## 7. Technical Guidance

-   Use `glam::Vec2` for vector math.
-   Ensure `SubspaceGrid` dimensions match the System Map dimensions.
-   This system does NOT move ships automatically (yet); it only modifies the *cost* (fuel/time) of intentional movement.

## 8. Questions

- *Builder: Should currents drift over time?*
*Architect:* Yes, slowly rotate or shift the positions of subspace currents every few in-game "months" to force players to adapt their established trade routes.
  *Architect: Yes, subspace currents shift seasonally, changing the optimal travel paths.*
*Architect: Yes, currents should slowly shift direction over years or seasons.*
    -   *Architect:* Not for MVP. Static currents are fine. Dynamic shifting is part of "Subspace Currents" feature expansion later.
