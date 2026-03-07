# 404: Thermal Management

## 1. Overview
Harnessing thermodynamics is key to survival. "Thermal Management" introduces a new systemic layer to the Layer 1 grid. Every tile has a Temperature value. Machines emit heat, walls insulate, and vents move heat between rooms.

This mechanic forces players to think about where they place their hot industrial buildings (like reactors and smelters) versus sensitive areas like greenhouses and housing. Poor thermal management can lead to freezing crops or overheating machinery that breaks down.

## 2. Dependencies
- `002-basic-map.md` (For the `TerrainGrid` that we will overlay with a Temperature grid)
- `006-building-placement.md` (For buildings that will act as heat sources/sinks/insulators)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{GridPosition, MapDimensions};

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(MapDimensions { width: 10, height: 10 });
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0)); // Default 20C
        app.add_systems(Update, (thermal_emission_system, thermal_diffusion_system));
        app
    }

    #[test]
    fn test_heat_source_increases_local_temperature() {
        let mut app = setup_app();

        // Spawn a heater at (5,5)
        app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            ThermalEmitter { heat_output: 10.0 }, // Emits 10 heat per tick
        ));

        app.update();

        let grid = app.world().get_resource::<TemperatureGrid>().unwrap();
        assert!(grid.get(5, 5) > 20.0, "Heater should raise the local tile temperature");
    }

    #[test]
    fn test_heat_diffuses_to_adjacent_tiles() {
        let mut app = setup_app();

        // Set high heat at center directly
        let mut grid = app.world_mut().get_resource_mut::<TemperatureGrid>().unwrap();
        grid.set(5, 5, 100.0);

        app.update();

        let updated_grid = app.world().get_resource::<TemperatureGrid>().unwrap();

        // Heat should lower in the center and increase in adjacent tiles
        assert!(updated_grid.get(5, 5) < 100.0);
        assert!(updated_grid.get(5, 4) > 20.0, "Heat should diffuse to neighbors");
    }

    #[test]
    fn test_insulation_blocks_diffusion() {
        let mut app = setup_app();

        // High heat at (5,5)
        app.world_mut().get_resource_mut::<TemperatureGrid>().unwrap().set(5, 5, 100.0);

        // Insulating wall at (5,4)
        app.world_mut().spawn((
            GridPosition { x: 5, y: 4 },
            ThermalInsulator { insulation_value: 0.9 }, // Blocks 90% of transfer
        ));

        app.update();

        let grid = app.world().get_resource::<TemperatureGrid>().unwrap();

        // Diffusion to the insulated tile should be much lower than to a normal tile (5,6)
        let insulated_temp = grid.get(5, 4);
        let open_temp = grid.get(5, 6);
        assert!(insulated_temp < open_temp, "Insulator should resist temperature changes");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::{GridPosition, MapDimensions};

#[derive(Resource)]
pub struct TemperatureGrid {
    pub cells: Vec<f32>,
    width: i32,
    height: i32,
}

impl TemperatureGrid {
    pub fn new(width: i32, height: i32, default_temp: f32) -> Self {
        Self {
            cells: vec![default_temp; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.cells[(y * self.width + x) as usize]
        } else {
            20.0 // Ambient boundary
        }
    }

    pub fn set(&mut self, x: i32, y: i32, temp: f32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.cells[(y * self.width + x) as usize] = temp;
        }
    }
}

#[derive(Component)]
pub struct ThermalEmitter {
    pub heat_output: f32, // Positive for heaters, negative for coolers
}

#[derive(Component)]
pub struct ThermalInsulator {
    pub insulation_value: f32, // 0.0 (conductive) to 1.0 (perfect insulator)
}

pub fn thermal_emission_system(
    mut grid: ResMut<TemperatureGrid>,
    query: Query<(&GridPosition, &ThermalEmitter)>,
) {
    for (pos, emitter) in query.iter() {
        let current = grid.get(pos.x, pos.y);
        // Simple additive mock logic
        grid.set(pos.x, pos.y, current + emitter.heat_output);
    }
}

pub fn thermal_diffusion_system(
    mut grid: ResMut<TemperatureGrid>,
    insulator_query: Query<(&GridPosition, &ThermalInsulator)>,
) {
    let mut next_grid = grid.cells.clone();
    let width = grid.width;
    let height = grid.height;

    // Cache insulators for quick lookup (Mocking this with a simple loop for green phase)
    let get_insulation = |x, y| -> f32 {
        for (pos, insulator) in insulator_query.iter() {
            if pos.x == x && pos.y == y {
                return insulator.insulation_value;
            }
        }
        0.0 // Default no insulation
    };

    let diffusion_rate = 0.1;

    for y in 0..height {
        for x in 0..width {
            let current_temp = grid.get(x, y);
            let mut diffused = 0.0;
            let mut neighbors = 0.0;

            let offsets = [(0, 1), (0, -1), (1, 0), (-1, 0)];
            for (dx, dy) in offsets {
                let nx = x + dx;
                let ny = y + dy;

                if nx >= 0 && nx < width && ny >= 0 && ny < height {
                    let neighbor_temp = grid.get(nx, ny);
                    let transfer = (neighbor_temp - current_temp) * diffusion_rate;

                    // Apply insulation (average insulation of the two cells)
                    let ins_here = get_insulation(x, y);
                    let ins_there = get_insulation(nx, ny);
                    let avg_insulation = (ins_here + ins_there) / 2.0;

                    let final_transfer = transfer * (1.0 - avg_insulation);
                    diffused += final_transfer;
                    neighbors += 1.0;
                }
            }

            next_grid[(y * width + x) as usize] = current_temp + diffused;
        }
    }

    grid.cells = next_grid;
}
```

## 5. REFACTOR Phase: Quality & Design
- **Performance:** `get_insulation` is extremely slow (O(N) per cell per neighbor). A secondary spatial map or grid storing `insulation_value` must be used instead of iterating the component query inside the nested loops.
- **Ambient Equilibrium:** The grid should naturally trend back towards a global ambient temperature representing the planet's baseline climate.
- **Visuals:** Hook up the `TemperatureGrid` to an optional rendering overlay (e.g., using `ratatui` colored blocks, red for hot, blue for cold) so the player can view the thermal landscape.

## 6. Acceptance Criteria (Testable!)
- [ ] `TemperatureGrid` resource exists and handles 2D spatial indexing.
- [ ] `ThermalEmitter` components correctly add or subtract heat locally.
- [ ] Heat naturally diffuses to neighboring tiles.
- [ ] `ThermalInsulator` reduces the rate of heat transfer.
- [ ] Tests pass locally with a `cargo test`.
- [ ] `cargo clippy -- -D warnings` runs without complaints.

## 7. Technical Guidance
- The cellular automata logic for diffusion needs to double buffer the grid (as shown in the green phase) to prevent order-of-operation bias.
- Add `TemperatureGrid` to the startup schedule inside `src/setup.rs` or a new module.

## 8. Questions
*Builder: add questions here if spec is unclear.*
