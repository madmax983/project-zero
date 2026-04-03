//! Operational Detritus (Clutter) System.
//!
//! This module implements the "Clutter" mechanic, representing the physical mess left behind by
//! active `Pop`s. Clutter negatively impacts beauty and increases pathfinding cost.
//!
//! # Context
//! The [`ClutterGrid`] stores the current clutter value for every tile.
//! *   The [`clutter_accumulation_system`] passively increases clutter on tiles where `Pop`s perform
//!     actions. The messiness depends on the [`ActionType`](crate::layer1::ActionType) (e.g., Working generates more clutter than Idling).
//! *   The [`clutter_cleaning_system`] is run when `Pop`s are assigned the `Clean` action. It reduces
//!     clutter on their current tile and has a small chance to spawn a `Scrap` item.
//!
//! # Usage
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::clutter::ClutterGrid;
//!
//! let mut grid = ClutterGrid::new(10, 10);
//!
//! // Add clutter
//! grid.add_clutter(5, 5, 20.0);
//! assert_eq!(grid.get(5, 5), 20.0);
//!
//! // Remove clutter
//! grid.remove_clutter(5, 5, 5.0);
//! assert_eq!(grid.get(5, 5), 15.0);
//!
//! // Clutter is clamped at 0.0 and 100.0
//! grid.remove_clutter(5, 5, 50.0);
//! assert_eq!(grid.get(5, 5), 0.0);
//! ```

use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
pub struct ClutterGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl ClutterGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");
        Self {
            width,
            height,
            values: vec![0.0; size],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.values[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[y * self.width + x] = val;
    }

    pub fn add_clutter(&mut self, x: usize, y: usize, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, (current + amount).min(100.0));
    }

    pub fn remove_clutter(&mut self, x: usize, y: usize, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, (current - amount).max(0.0));
    }
}

pub fn clutter_accumulation_system(
    mut grid: ResMut<ClutterGrid>,
    pops: Query<(&crate::layer1::GridPosition, &crate::layer1::PopAction)>,
) {
    for (pos, action) in &pops {
        // Spec values: Work => 0.05, Move => 0.02, Idle => 0.01
        // We map "Move" to Explore for test purposes, or any action that implies heavy movement.
        // Actually, let's map actions to "Activity Level".
        let amount = match action.current {
            crate::layer1::ActionType::Work
            | crate::layer1::ActionType::Refine
            | crate::layer1::ActionType::Farm
            | crate::layer1::ActionType::Repair => 0.05,

            crate::layer1::ActionType::Explore
            | crate::layer1::ActionType::Haul
            | crate::layer1::ActionType::FetchTool
            | crate::layer1::ActionType::FetchClothing => 0.02,

            crate::layer1::ActionType::Idle
            | crate::layer1::ActionType::Socialize
            | crate::layer1::ActionType::SatisfyRest => 0.01,

            // Default low accumulation for others
            _ => 0.01,
        };

        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            grid.add_clutter(x, y, amount);
        }
    }
}

pub fn clutter_cleaning_system(
    mut commands: Commands,
    mut grid: ResMut<ClutterGrid>,
    pops: Query<(&crate::layer1::GridPosition, &crate::layer1::PopAction)>,
) {
    for (pos, action) in &pops {
        if action.current == crate::layer1::ActionType::Clean {
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                // Remove clutter
                grid.remove_clutter(x, y, 5.0);

                // Scavenge chance (1%)
                if rand::thread_rng().gen_bool(0.01) {
                    commands.spawn((
                        crate::layer1::items::Item {
                            item_type: crate::layer1::items::ItemType::Scrap,
                        },
                        *pos,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::clutter::ClutterGrid;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_types::{ActionType, PopAction};

    // Helper to setup world
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ClutterGrid::new(10, 10));
        world.insert_resource(BeautyGrid::new(10, 10));
        // Add other necessary resources (TerrainGrid, etc.)
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });
        world
    }

    #[test]
    fn test_clutter_grid_initialization() {
        let world = setup_world();
        let grid = world.resource::<ClutterGrid>();
        assert_eq!(grid.get(0, 0), 0.0);
        assert_eq!(grid.width, 10);
    }

    #[test]
    fn test_clutter_accumulation_movement() {
        let mut world = setup_world();
        // Spawn a pop moving through (5, 5)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Explore, // Using Explore as proxy for Move
                ..Default::default()
            },
        ));

        // Run accumulation system once
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_accumulation_system);
        schedule.run(&mut world);

        let grid = world.resource::<ClutterGrid>();
        assert!(
            grid.get(5, 5) > 0.0,
            "Movement (Explore) should generate clutter"
        );
    }

    #[test]
    fn test_clutter_accumulation_work() {
        let mut world = setup_world();
        // Spawn a pop working at (5, 5)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Work,
                ..Default::default()
            },
        ));

        // Run accumulation system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_accumulation_system);
        schedule.run(&mut world);

        let grid = world.resource::<ClutterGrid>();
        let work_clutter = grid.get(5, 5);
        assert!(work_clutter > 0.0, "Work should generate clutter");

        // Reset and test idle
        let mut world2 = setup_world();
        world2.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Idle,
                ..Default::default()
            },
        ));

        let mut schedule2 = Schedule::default();
        schedule2.add_systems(crate::layer1::clutter::clutter_accumulation_system);
        schedule2.run(&mut world2);

        let idle_clutter = world2.resource::<ClutterGrid>().get(5, 5);

        assert!(
            work_clutter > idle_clutter,
            "Work should be messier than Idle"
        );
    }

    #[test]
    fn test_clutter_impact_on_beauty() {
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(0, 0, 50.0); // High clutter

        // Run beauty update system (modified to read ClutterGrid)
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::beauty::update_beauty_grid_system);
        schedule.run(&mut world);

        let beauty = world.resource::<BeautyGrid>();
        // Expect negative beauty from clutter
        // Default grass beauty is 1.0.
        // If clutter 50.0 -> penalty -5.0. Net -4.0.
        assert!(beauty.get(0, 0) < 0.0, "Clutter should reduce beauty");
    }

    #[test]
    fn test_clutter_impact_on_pathfinding_cost() {
        // This test requires modifying pathfinding.rs to read ClutterGrid.

        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        // Make (1, 0) very cluttered
        clutter.set(1, 0, 100.0);

        // Populate standard pathfinding resources
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        world.insert_resource(crate::layer1::building::BuildingMap::default());
        world.insert_resource(crate::layer1::crowding::CrowdingGrid::new(10, 10));

        // Path from (0,0) to (2,0)
        let path = crate::layer1::pathfinding::find_path(&world, (0, 0), (2, 0));

        if let Some(p) = path {
            assert!(
                !p.contains(&(1, 0)),
                "Path should avoid high clutter at (1,0)"
            );
        }
    }

    #[test]
    fn test_janitor_clean_action() {
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(5, 5, 50.0);

        // Spawn Janitor performing Clean action
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Clean,
                ..Default::default()
            },
        ));

        // Run cleaning system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_cleaning_system);
        schedule.run(&mut world);

        let grid = world.resource::<ClutterGrid>();
        assert!(grid.get(5, 5) < 50.0, "Cleaning should reduce clutter");
    }

    #[test]
    fn test_cleaning_spawns_scrap() {
        // Run cleaning loop many times to verify probability
        let mut world = setup_world();
        let mut clutter = world.resource_mut::<ClutterGrid>();
        clutter.set(5, 5, 1000.0); // Infinite clutter for testing

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Clean,
                ..Default::default()
            },
        ));

        // Need to run system many times
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::clutter::clutter_cleaning_system);

        let mut found_scrap = false;
        for _ in 0..1000 {
            schedule.run(&mut world);

            // Check for Scrap item
            let mut query = world.query::<(&Item, &GridPosition)>();
            for (item, pos) in query.iter(&world) {
                if item.item_type == ItemType::Scrap && pos.x == 5 && pos.y == 5 {
                    found_scrap = true;
                    break;
                }
            }
            if found_scrap {
                break;
            }
        }

        assert!(found_scrap, "Cleaning should eventually spawn Scrap");
    }
}
