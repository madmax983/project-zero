// src/layer1/atmosphere.rs

use crate::layer1::building::{Building, BuildingType};
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// Represents the atmospheric pollution layer.
/// Values range from 0.0 (Clean) to 1.0 (Toxic).
#[derive(Resource)]
pub struct AtmosphereGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Flattened grid values.
    pub values: Vec<f32>,
    /// Secondary buffer for diffusion calculation (double buffering).
    /// Used to avoid allocating a new vector every tick.
    pub scratch: Vec<f32>,
    /// Retention rate of pollution per tick (0.0 to 1.0).
    /// Higher values mean pollution stays longer.
    /// Default: 0.99.
    pub diffusion_rate: f32,
}

impl AtmosphereGrid {
    /// Create a new empty atmosphere grid.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            values: vec![0.0; size],
            scratch: vec![0.0; size],
            diffusion_rate: 0.99,
        }
    }

    /// Get pollution level at (x, y). Returns 0.0 if out of bounds.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 {
            return 0.0;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return 0.0;
        }
        self.values[uy * self.width + ux]
    }

    /// Set pollution level at (x, y). Clamped between 0.0 and 1.0.
    #[allow(clippy::cast_sign_loss)]
    pub fn set(&mut self, x: i32, y: i32, value: f32) {
        if x < 0 || y < 0 {
            return;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return;
        }
        self.values[uy * self.width + ux] = value.clamp(0.0, 1.0);
    }

    /// Add pollution at (x, y). Clamped to max 1.0.
    pub fn add(&mut self, x: i32, y: i32, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, current + amount);
    }

    /// Simulate diffusion and natural decay of pollution.
    /// Uses a simple box blur.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    pub fn diffuse(&mut self) {
        // Ensure scratch buffer size matches (in case of dynamic resizing, though rare)
        if self.scratch.len() != self.values.len() {
            self.scratch = vec![0.0; self.values.len()];
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let mut sum = self.values[idx];
                let mut count = 1.0;

                // Check 4 neighbors
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < self.width
                        && (ny as usize) < self.height
                    {
                        sum += self.get(nx, ny);
                        count += 1.0;
                    }
                }

                // Average
                self.scratch[idx] = sum / count;
                // Decay
                self.scratch[idx] *= self.diffusion_rate;
            }
        }
        // Swap buffers
        std::mem::swap(&mut self.values, &mut self.scratch);
    }
}

/// System to update atmospheric simulation (emission + diffusion).
pub fn update_atmosphere_system(world: &mut World) {
    // 1. Emission
    let mut emitters = Vec::new();
    let mut query = world.query::<(&Building, &GridPosition)>();
    for (b, pos) in query.iter(world) {
        let emission = match b.building_type {
            BuildingType::Refinery | BuildingType::AncientReactor => 0.08,
            BuildingType::Smelter | BuildingType::Generator => 0.05,
            BuildingType::Smithy => 0.02,
            _ => 0.0,
        };
        if emission > 0.0 {
            emitters.push((*pos, emission));
        }
    }

    // 2. Apply emissions & Diffuse
    if let Some(mut grid) = world.get_resource_mut::<AtmosphereGrid>() {
        for (pos, amount) in emitters {
            grid.add(pos.x, pos.y, amount);
        }
        grid.diffuse();
    }
}

/// System to apply health effects from pollution.
pub fn pollution_effects_system(world: &mut World) {
    let mut damages = Vec::new();

    // 1. Calculate damages (Read-only phase)
    {
        let mut query = world.query_filtered::<(Entity, &GridPosition), With<Pop>>();
        let grid = world.resource::<AtmosphereGrid>();

        for (entity, pos) in query.iter(world) {
            let pollution = grid.get(pos.x, pos.y);
            // Threshold 0.3
            if pollution > 0.3 {
                let damage = (pollution - 0.3) * 0.1;
                damages.push((entity, damage));
            }
        }
    }

    // 2. Apply damages (Write phase)
    for (entity, damage) in damages {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.take_damage(damage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_atmosphere_grid_initialization() {
        let grid = AtmosphereGrid::new(10, 10);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_pollution_emission() {
        let mut world = World::new();
        let grid = AtmosphereGrid::new(10, 10);
        world.insert_resource(grid);

        // Spawn a Smelter (dirty building)
        world.spawn((
            Building {
                building_type: BuildingType::Smelter,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run update
        update_atmosphere_system(&mut world);

        // Check pollution at source
        let grid = world.resource::<AtmosphereGrid>();
        assert!(grid.get(5, 5) > 0.0, "Smelter should emit pollution");
    }

    #[test]
    fn test_pollution_diffusion() {
        let mut grid = AtmosphereGrid::new(3, 3);
        grid.set(1, 1, 10.0); // High pollution in center

        // Simulate one step of diffusion
        grid.diffuse();

        // Center should decrease, neighbors should increase
        assert!(
            grid.get(1, 1) < 10.0,
            "Pollution should diffuse away from center"
        );
        assert!(
            grid.get(0, 1) > 0.0,
            "Pollution should diffuse to neighbors"
        );
        assert!(
            grid.get(1, 0) > 0.0,
            "Pollution should diffuse to neighbors"
        );
    }

    #[test]
    fn test_pollution_health_impact() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(5, 5, 1.0); // Max pollution
        world.insert_resource(grid);

        // Spawn Pop in pollution
        let pop = world
            .spawn((Pop, Health::default(), GridPosition { x: 5, y: 5 }))
            .id();

        // Run effects
        pollution_effects_system(&mut world);

        // Health should drop
        let health = world.get::<Health>(pop).unwrap();
        assert!(
            health.current < 100.0,
            "Health should drop due to pollution"
        );
    }
}
