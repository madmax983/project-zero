//! The Atmospheric Pressure System.
//!
//! This module simulates the dispersion, containment, and loss of atmospheric pressure
//! within the colony. Because pops require oxygen to survive, maintaining a pressurized
//! environment is essential. If a room's pressure drops to a vacuum (0.0), occupants
//! will quickly take suffocation damage.
//!
//! Pressure is simulated as a cellular automata over a 2D grid (`PressureGrid`). Generators
//! (like `LifeSupport` buildings) inject pressure into the grid, which then diffuses to
//! adjacent cells. Blockers (like `Wall` or `Airlock` buildings) restrict or halt this
//! flow, allowing you to create sealed environments.
//!
//! # Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::pressure::{PressureGrid, update_pressure_system};
//! use scale::layer1::building::{Building, BuildingType};
//! use scale::layer1::map::GridPosition;
//!
//! let mut world = World::new();
//!
//! // 1. Create a 5x5 grid (defaults to 0.0 vacuum)
//! let grid = PressureGrid::new(5, 5);
//! world.insert_resource(grid);
//!
//! // 2. Add a Life Support generator
//! world.spawn((
//!     Building { building_type: BuildingType::LifeSupport },
//!     GridPosition { x: 2, y: 2 }
//! ));
//!
//! // 3. Run the system to generate and diffuse pressure
//! update_pressure_system(&mut world);
//!
//! // The grid now has pressure near the generator!
//! let pressure = world.resource::<PressureGrid>().get(2, 2);
//! assert!(pressure > 0.0);
//! ```
//!
//! # Details
//! - **Diffusion Algorithm:** Pressure diffuses by averaging a cell with its 4 cardinal
//!   neighbors, weighted by their `transmissivity` (0.0 for walls, 1.0 for empty space).
//! - **Vacuum Decay:** If an area is exposed to the void (edges of the grid or breached
//!   walls), pressure will rapidly equalize with the vacuum, depleting the room.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Represents the atmospheric pressure layer.
/// Values range from 0.0 (Vacuum) to 1.0 (Standard Atmosphere).
#[derive(Resource)]
pub struct PressureGrid {
    /// Width of the grid.
    pub(crate) width: usize,
    /// Height of the grid.
    pub(crate) height: usize,
    /// Flattened grid values.
    pub(crate) values: Vec<f32>,
}

impl PressureGrid {
    /// Fills the entire grid with a value.
    pub fn fill(&mut self, value: f32) {
        self.values.fill(value);
    }

    /// Create a new empty pressure grid (initialized to 0.0).
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    /// Get pressure at (x, y). Returns 0.0 if out of bounds.
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

    /// Set pressure at (x, y). Clamped between 0.0 and 1.0.
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

    /// Add pressure at (x, y). Clamped to max 1.0.
    pub fn add(&mut self, x: i32, y: i32, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, current + amount);
    }

    /// Simulate diffusion of pressure.
    ///
    /// # Arguments
    ///
    /// * `blockers` - Map of (x, y) to transmissivity (0.0 = Wall, 1.0 = Open).
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    pub fn diffuse(&mut self, blockers: &HashMap<(i32, i32), f32>) {
        let mut new_values = self.values.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                // If the cell itself is a solid blocker (Wall), it contains no pressure.
                if blockers
                    .get(&(x as i32, y as i32))
                    .is_some_and(|&trans| trans <= f32::EPSILON)
                {
                    if let Some(idx) = y.checked_mul(self.width).and_then(|i| i.checked_add(x)) {
                        new_values[idx] = 0.0;
                    }
                    continue;
                }

                let idx = y
                    .checked_mul(self.width)
                    .and_then(|i| i.checked_add(x))
                    .unwrap_or(usize::MAX);
                if idx >= self.values.len() {
                    continue;
                }
                let current_val = self.values[idx];

                let mut sum = current_val;
                let mut total_weight = 1.0;

                // Check 4 neighbors
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < self.width
                        && (ny as usize) < self.height
                    {
                        let neighbor_val = self.get(nx, ny);
                        let neighbor_trans = *blockers.get(&(nx, ny)).unwrap_or(&1.0);

                        if neighbor_trans > f32::EPSILON {
                            sum += neighbor_val * neighbor_trans;
                            total_weight += neighbor_trans;
                        }
                    } else {
                        // Edge is vacuum (0.0 pressure, 1.0 transmissivity)
                        sum += 0.0;
                        total_weight += 1.0;
                    }
                }

                if total_weight > 0.0 {
                    new_values[idx] = sum / total_weight;
                }
            }
        }
        self.values = new_values;
    }
}

/// System to update atmospheric pressure.
pub fn update_pressure_system(world: &mut World) {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::map::GridPosition;

    // 1. Identify blockers
    let mut blockers = HashMap::new();
    {
        let mut query = world.query::<(
            &Building,
            &GridPosition,
            Option<&crate::layer1::control::DoorControl>,
        )>();
        for (b, pos, control) in query.iter(world) {
            let mut transmissivity = b.building_type.flow_transmissivity();

            if let Some(ctrl) = control {
                match ctrl.state {
                    crate::layer1::control::DoorState::Open => transmissivity = Some(1.0),
                    crate::layer1::control::DoorState::Locked => transmissivity = Some(0.0),
                    crate::layer1::control::DoorState::Auto => {}
                }
            }

            if let Some(t) = transmissivity {
                blockers.insert((pos.x, pos.y), t);
            }
        }
    }

    // 2. Identify Generators
    let mut generators = Vec::new();
    {
        let mut query = world.query::<(&Building, &GridPosition, Option<&PowerConsumer>)>();
        for (b, pos, power) in query.iter(world) {
            if b.building_type == BuildingType::LifeSupport {
                let is_active = power.is_none_or(|p| p.active);
                if is_active {
                    generators.push(*pos);
                }
            }
        }
    }

    // 3. Apply to Grid
    if let Some(mut grid) = world.get_resource_mut::<PressureGrid>() {
        for pos in generators {
            let current = grid.get(pos.x, pos.y);
            if current < 1.0 {
                grid.set(pos.x, pos.y, (current + 0.2).min(1.0));
            }
        }
        grid.diffuse(&blockers);
    }
}

/// System to apply suffocation damage.
pub fn pressure_damage_system(world: &mut World) {
    use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;

    // 1. Collect candidate entities (Entity, Pos)
    let mut candidates = Vec::new();
    {
        let mut query = world.query_filtered::<(Entity, &GridPosition, &Health), With<Pop>>();
        for (entity, pos, health) in query.iter(world) {
            if health.current > 0.0 {
                candidates.push((entity, *pos));
            }
        }
    }

    // 2. Check pressure against Grid
    let mut damage_targets = Vec::new();
    if let Some(grid) = world.get_resource::<PressureGrid>() {
        for (entity, pos) in candidates {
            let pressure = grid.get(pos.x, pos.y);
            if pressure < 0.2 {
                damage_targets.push(entity);
            }
        }
    }

    // 3. Apply damage
    let mut killed = 0;
    for entity in damage_targets {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.take_damage(1.0);
            if health.current <= 0.0 {
                killed += 1;
            }
        }
    }

    // 4. Emit events
    if killed > 0 {
        if let Some(mut events) = world.get_resource_mut::<Events<AddChronicleEvent>>() {
            events.send(AddChronicleEvent {
                text: format!("{} Pops suffocated due to atmospheric breach.", killed),
                importance: EventImportance::Major,
            });
        }
    }
}

pub fn process_door_venting_system(
    mut grid: ResMut<PressureGrid>,
    query: Query<(
        &crate::layer1::building::Building,
        &crate::layer1::control::DoorControl,
        &crate::layer1::map::GridPosition,
    )>,
) {
    for (building, control, pos) in query.iter() {
        if control.state == crate::layer1::control::DoorState::Open {
            let vent_rate =
                if building.building_type == crate::layer1::building::BuildingType::Airlock {
                    0.04
                } else {
                    0.5
                };

            // Diffuse X and Y neighbors across the door
            for (dx, dy) in [(-1, 0), (0, -1)] {
                let p1 = grid.get(pos.x + dx, pos.y + dy);
                let p2 = grid.get(pos.x - dx, pos.y - dy);
                let diff = (p1 - p2) * vent_rate;
                grid.set(pos.x + dx, pos.y + dy, p1 - diff);
                grid.set(pos.x - dx, pos.y - dy, p2 + diff);
            }
        }
    }
}

pub fn apply_door_movement_penalties_system(
    mut pop_query: Query<
        (
            &crate::layer1::map::GridPosition,
            &mut crate::layer1::pop::Speed,
        ),
        With<crate::layer1::pop::Pop>,
    >,
    door_query: Query<(
        &crate::layer1::map::GridPosition,
        &crate::layer1::building::Building,
    )>,
) {
    let mut airlock_positions = std::collections::HashSet::new();
    for (door_pos, building) in door_query.iter() {
        if building.building_type == crate::layer1::building::BuildingType::Airlock {
            airlock_positions.insert((door_pos.x, door_pos.y));
        }
    }

    for (pop_pos, mut speed) in pop_query.iter_mut() {
        if airlock_positions.contains(&(pop_pos.x, pop_pos.y)) {
            speed.current *= 0.5; // Slow down
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
    use crate::layer1::structure::Structure;

    #[test]
    fn test_pressure_grid_defaults_to_vacuum() {
        let grid = PressureGrid::new(10, 10);
        assert_eq!(grid.get(5, 5), 0.0);
    }

    #[test]
    fn test_life_support_generates_pressure() {
        let mut world = World::new();
        let grid = PressureGrid::new(10, 10);
        world.insert_resource(grid);

        world.spawn((
            Building {
                building_type: BuildingType::LifeSupport,
            },
            GridPosition { x: 5, y: 5 },
        ));

        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(
            grid.get(5, 5) > 0.0,
            "Life Support should generate pressure"
        );
    }

    #[test]
    fn test_walls_block_diffusion() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0); // Source
        world.insert_resource(grid);

        // Wall at (2, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 2, y: 0 },
            Structure::default(),
        ));

        for _ in 0..5 {
            update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        // (1,0) source. (2,0) wall. (3,0) target.
        assert!(
            grid.get(3, 0) < 0.01,
            "Pressure should not pass through wall"
        );
    }

    #[test]
    fn test_gate_leaks_pressure() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Gate at (2, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Gate,
            },
            GridPosition { x: 2, y: 0 },
            Structure::default(),
        ));

        // Run multiple ticks
        for _ in 0..5 {
            update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(3, 0) > 0.0, "Pressure SHOULD leak through Gate");
    }

    #[test]
    fn test_airlock_blocks_pressure() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Airlock at (2, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            GridPosition { x: 2, y: 0 },
            Structure::default(),
        ));

        for _ in 0..5 {
            update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        assert!(
            grid.get(3, 0) < 0.01,
            "Pressure should NOT leak through Airlock"
        );
    }

    #[test]
    fn test_suffocation_damage() {
        let mut world = World::new();
        let grid = PressureGrid::new(10, 10);
        world.insert_resource(grid);

        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        pressure_damage_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Pop in vacuum should take damage");
    }

    #[test]
    fn test_pressure_passes_through_vent() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(0, 0, 1.0); // Source
        world.insert_resource(grid);

        // Source generator at (0, 0) to maintain pressure against vacuum
        world.spawn((
            Building {
                building_type: BuildingType::LifeSupport,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Vent at (1, 0)
        world.spawn((
            Building {
                building_type: BuildingType::Vent,
            },
            GridPosition { x: 1, y: 0 },
        ));

        // Run pressure update multiple times to allow diffusion
        for _ in 0..20 {
            // Manually refill source to fight vacuum decay for test purposes
            world.resource_mut::<PressureGrid>().set(0, 0, 1.0);
            update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(2, 0) > 0.05, "Pressure SHOULD pass through Vent");
    }

    #[test]
    fn test_door_vents_atmosphere_on_open() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        let _door_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Gate,
                },
                crate::layer1::control::DoorControl {
                    state: crate::layer1::control::DoorState::Open,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        app.world_mut().insert_resource(PressureGrid::new(20, 20));
        app.world_mut()
            .resource_mut::<PressureGrid>()
            .set(9, 10, 1.0);
        app.world_mut()
            .resource_mut::<PressureGrid>()
            .set(11, 10, 0.0);

        app.add_systems(bevy::prelude::Update, process_door_venting_system);
        app.update();

        let grid = app.world().resource::<PressureGrid>();
        assert!(grid.get(9, 10) < 1.0, "Interior pressure should drop");
        assert!(grid.get(11, 10) > 0.0, "Exterior pressure should rise");
    }

    #[test]
    fn test_airlock_minimizes_venting() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        let _airlock_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Airlock,
                },
                crate::layer1::control::DoorControl {
                    state: crate::layer1::control::DoorState::Open,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        app.world_mut().insert_resource(PressureGrid::new(20, 20));
        app.world_mut()
            .resource_mut::<PressureGrid>()
            .set(9, 10, 1.0);
        app.world_mut()
            .resource_mut::<PressureGrid>()
            .set(11, 10, 0.0);

        app.add_systems(bevy::prelude::Update, process_door_venting_system);
        app.update();

        let grid = app.world().resource::<PressureGrid>();
        assert!(grid.get(9, 10) < 1.0, "Airlock should vent slightly");
        assert!(
            grid.get(9, 10) > 0.95,
            "Airlock should preserve most interior pressure"
        );
        assert!(
            grid.get(11, 10) < 0.05,
            "Airlock should leak minimal pressure"
        );
    }

    #[test]
    fn test_airlock_slows_movement() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        let _airlock_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Airlock,
                },
                crate::layer1::control::DoorControl {
                    state: crate::layer1::control::DoorState::Auto,
                }, // Even if auto, passing through slows down
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                crate::layer1::pop::Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        app.add_systems(bevy::prelude::Update, apply_door_movement_penalties_system);
        app.update();

        let speed = app
            .world()
            .get::<crate::layer1::pop::Speed>(pop_entity)
            .unwrap();
        assert!(speed.current < speed.base, "Airlock should slow movement");
    }
}
