use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::health::Health;
use crate::layer1::items::{Clothing, Equipment};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::seasons::SeasonState;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Grid managing temperature simulation.
#[derive(Resource)]
pub struct TemperatureGrid {
    pub width: usize,
    pub height: usize,
    /// Current temperature values (Celsius).
    pub values: Vec<f32>,
    /// Scratch buffer for double-buffered updates.
    pub scratch: Vec<f32>,
    /// Target ambient temperature based on season/environment.
    pub ambient: f32,
}

impl TemperatureGrid {
    /// Create a new temperature grid.
    pub fn new(width: usize, height: usize, ambient: f32) -> Self {
        Self {
            width,
            height,
            values: vec![ambient; width * height],
            scratch: vec![ambient; width * height],
            ambient,
        }
    }

    /// Get temperature at coordinates.
    /// Returns ambient if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            return self.ambient;
        }
        self.values[y * self.width + x]
    }

    /// Set temperature at coordinates.
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[y * self.width + x] = value;
    }

    /// Add heat to a specific tile.
    pub fn add(&mut self, x: i32, y: i32, amount: f32) {
        if x < 0 || y < 0 {
            return;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= self.width || uy >= self.height {
            return;
        }
        self.values[uy * self.width + ux] += amount;
    }

    /// Run one step of diffusion simulation.
    ///
    /// - Diffuses heat between neighbors based on conductivity.
    /// - Drifts all cells slightly towards ambient temperature.
    pub fn diffuse(&mut self, conductivity: &HashMap<(i32, i32), f32>) {
        let diffusion_rate = 0.2; // How fast heat spreads
        let ambient_drift = 0.01; // How fast uninsulated areas return to ambient

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let current_temp = self.values[idx];
                let ix = x as i32;
                let iy = y as i32;

                // Get local conductivity (default 1.0 for air)
                // If a wall is HERE, it insulates this tile from others?
                // Or is conductivity strictly an edge property?
                // For simplicity, we use the minimum conductivity of the two tiles.
                // But `conductivity` map usually stores building properties AT a tile.
                // So if (x,y) has a Wall (0.05), heat flow in/out is restricted.

                let self_k = *conductivity.get(&(ix, iy)).unwrap_or(&1.0);

                // Neighbors: Up, Down, Left, Right
                let neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)];
                let mut flow_sum = 0.0;

                for (dx, dy) in neighbors {
                    let nx = ix + dx;
                    let ny = iy + dy;

                    let n_temp = if nx < 0
                        || ny < 0
                        || nx >= self.width as i32
                        || ny >= self.height as i32
                    {
                        self.ambient // Edge is ambient
                    } else {
                        self.values[(ny as usize) * self.width + (nx as usize)]
                    };

                    let neighbor_k = *conductivity.get(&(nx, ny)).unwrap_or(&1.0);

                    // Effective conductivity is the bottleneck (min or avg).
                    // Min is good for walls.
                    let k = self_k.min(neighbor_k);

                    flow_sum += (n_temp - current_temp) * k * diffusion_rate;
                }

                // Apply drift to ambient (simulating Z-axis loss or general loss)
                // Insulated tiles drift slower? Or walls themselves drift?
                // Let's say everything drifts a bit, but walls drift slower?
                // For now, constant drift.
                let drift = (self.ambient - current_temp) * ambient_drift;

                self.scratch[idx] = current_temp + flow_sum + drift;
            }
        }

        // Swap buffers
        std::mem::swap(&mut self.values, &mut self.scratch);
    }
}

/// System to update the temperature grid.
pub fn update_temperature_system(
    mut grid: Option<ResMut<TemperatureGrid>>,
    season: Option<Res<SeasonState>>,
    buildings: Query<(&Building, &GridPosition, Option<&PowerConsumer>)>,
) {
    let Some(mut grid) = grid else { return };

    // 1. Update Ambient
    if let Some(season) = season {
        grid.ambient = season.current_season.base_temperature();
    }

    // 2. Apply Heat Sources & Build Conductivity Map
    let mut conductivity_map = HashMap::new();

    for (b, pos, power) in &buildings {
        // Conductivity
        let k = b.building_type.thermal_conductivity();
        if (k - 1.0).abs() > f32::EPSILON {
            conductivity_map.insert((pos.x, pos.y), k);
        }

        // Heat Sources
        let heat = match b.building_type {
            BuildingType::Heater => {
                if power.is_some_and(|p| !p.active) {
                    0.0
                } else {
                    5.0
                }
            }
            BuildingType::Smelter => {
                if power.is_some_and(|p| !p.active) {
                    0.0
                } else {
                    2.0
                }
            }
            BuildingType::Generator => 2.0, // Generators produce heat when running (fuel logic is separate)
            BuildingType::AncientReactor => 10.0,
            // Fire? handled by Fire entity, usually.
            // If Fire is a separate entity, we might query it separately.
            // For now, buildings only.
            _ => 0.0,
        };

        if heat > 0.0 {
            grid.add(pos.x, pos.y, heat);
        }
    }

    // 3. Diffuse
    grid.diffuse(&conductivity_map);
}

/// System to apply thermal damage to pops.
pub fn thermal_damage_system(
    grid: Option<Res<TemperatureGrid>>,
    mut pops: Query<(
        &mut Health,
        &GridPosition,
        &Equipment,
    ), With<Pop>>,
    clothing_query: Query<&Clothing>,
) {
    let Some(grid) = grid else { return };

    for (mut health, pos, equipment) in &mut pops {
        // Get temperature at pop's location
        let temp = grid.get(pos.x as usize, pos.y as usize);

        // Calculate Insulation from Equipment
        let mut insulation = 0.0;
        if let Some(body_entity) = equipment.body {
            if let Ok(clothing) = clothing_query.get(body_entity) {
                insulation += clothing.insulation;
            }
        }

        // Define Safe Range
        // Base range: 10C to 35C
        // Insulation extends the lower bound.
        // E.g. 1.0 insulation -> -20C tolerance extension?
        // Let's say each 1.0 insulation adds 20 degrees of cold protection.
        let cold_tolerance = 10.0 - (insulation * 30.0);
        let heat_tolerance = 35.0; // Clothing might add heat, but usually protects from cold.

        if temp < cold_tolerance {
            // Hypothermia
            // Damage scales with severity?
            // Spec says "0.5 damage".
            health.take_damage(0.5);
        } else if temp > heat_tolerance {
            // Heatstroke
            health.take_damage(0.5);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::temperature::{TemperatureGrid, update_temperature_system, thermal_damage_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::items::{Clothing, Equipment, ClothingType};
    use crate::layer1::energy::PowerConsumer;

    #[test]
    fn test_grid_initialization_to_ambient() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter }); // -5.0 C

        // Initialize grid
        let grid = TemperatureGrid::new(10, 10, -5.0);

        assert_eq!(grid.get(5, 5), -5.0);
    }

    #[test]
    fn test_heat_source_emission() {
        let mut world = World::new();
        let grid = TemperatureGrid::new(10, 10, 0.0);
        world.insert_resource(grid);
        world.insert_resource(SeasonState::default()); // Need season or ambient update might fail/reset

        // Spawn Heater (Active)
        world.spawn((
            Building { building_type: BuildingType::Heater },
            GridPosition { x: 5, y: 5 },
            PowerConsumer { demand: 5.0, active: true },
        ));

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(5, 5) > 0.0, "Heater should raise temperature");
    }

    #[test]
    fn test_unpowered_heater_no_heat() {
        let mut world = World::new();
        // Initialize with Spring ambient (15.0) to avoid drift interference
        let grid = TemperatureGrid::new(10, 10, 15.0);
        world.insert_resource(grid);
        world.insert_resource(SeasonState::default());

        // Spawn Heater (Inactive)
        world.spawn((
            Building { building_type: BuildingType::Heater },
            GridPosition { x: 5, y: 5 },
            PowerConsumer { demand: 5.0, active: false },
        ));

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        assert!((grid.get(5, 5) - 15.0).abs() < f32::EPSILON, "Inactive heater should not raise temperature");
    }

    #[test]
    fn test_diffusion_and_insulation() {
        let mut world = World::new();
        let mut grid = TemperatureGrid::new(5, 1, 0.0);
        grid.set(0, 0, 100.0); // Hot source
        world.insert_resource(grid);
        world.insert_resource(SeasonState::default());

        // Wall at (2, 0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 2, y: 0 },
        ));

        // Run update (multiple ticks for diffusion)
        for _ in 0..10 {
            world.resource_mut::<TemperatureGrid>().set(0, 0, 100.0); // Replenish source to combat rapid edge loss
            world.run_system_once(update_temperature_system).unwrap();
        }

        let grid = world.resource::<TemperatureGrid>();
        // (1,0) should be warm (neighbor of source)
        assert!(grid.get(1, 0) > 10.0);
        // (3,0) should be cold (blocked by wall)
        // Walls are not perfect insulators (0.05 conductivity), but significantly colder than open air
        assert!(grid.get(3, 0) < grid.get(1, 0) * 0.5, "Wall should block most heat");
    }

    #[test]
    fn test_thermal_damage() {
        let mut world = World::new();
        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        grid.set(5, 5, -20.0); // Freezing
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop::default(),
            Health { current: 100.0, max: 100.0 },
            Equipment::default(),
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(thermal_damage_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Freezing temp should damage Pop");
    }

    #[test]
    fn test_ambient_drift() {
        // Tiles should slowly drift towards season ambient temp if not insulated
        let mut world = World::new();
        // let ambient = -10.0;
        world.insert_resource(SeasonState { current_season: Season::Winter }); // Assume Winter = -5.0
        let mut grid = TemperatureGrid::new(10, 10, 20.0); // Start warm (20 C)
        // Set explicit ambient on grid to match season for test clarity,
        // though system will overwrite it.
        grid.ambient = -5.0;
        world.insert_resource(grid);

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(0, 0) < 20.0, "Should cool down towards ambient");
    }

    #[test]
    fn test_clothing_protection() {
        let mut world = World::new();
        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        grid.set(5, 5, -15.0); // Cold (-15)
        world.insert_resource(grid);

        // Create Clothing entity
        let coat = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic, // Generic
            insulation: 1.0, // Protects ~30 degrees
            durability: 100.0,
            max_durability: 100.0,
        }).id();

        let pop = world.spawn((
            Pop::default(),
            Health { current: 100.0, max: 100.0 },
            Equipment {
                body: Some(coat),
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        world.run_system_once(thermal_damage_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        // Base safe min = 10.0. Insulation 1.0 -> 10 - 30 = -20.0 safe min.
        // Temp -15.0 is > -20.0, so should be safe.
        assert_eq!(health.current, 100.0, "Clothing should protect pop");
    }
}
