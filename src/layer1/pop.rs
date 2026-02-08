//! Population management and entity definitions.
//!
//! # The "Pop" (Population Agent)
//!
//! A "Pop" is the atomic unit of agency in the colony. They are not just resources;
//! they are semi-autonomous agents driven by a hierarchy of needs (see `needs.rs`)
//! and decision-making logic (see `utility_ai.rs`).
//!
//! ## Lifecycle
//!
//! 1.  **Spawning**: Pops are created by `spawn_initial_pops` (or potential future immigration events).
//! 2.  **Simulation**: Every tick, systems in `needs.rs` update their physiological state.
//! 3.  **Decision**: The Utility AI (`utility_ai.rs`) evaluates options and assigns a `PopAction`.
//! 4.  **Execution**: The chosen action is carried out, modifying the world or the pop's state.
//!
//! ## Components
//!
//! A fully initialized Pop entity typically has:
//! * `Pop`: The marker component.
//! * `GridPosition`: Physical location.
//! * `Needs`: Hunger, rest, etc.
//! * `PopAction`: Current task state.
//! * `UtilityWeights`: Personality/learning factors.

use super::health::Health;
use super::map::GridPosition;
use super::memory::Memories;
use super::needs::Needs;
use super::rumor::Knowledge;
use super::skills::Skills;
use super::terrain::{TerrainGrid, TerrainType};
use super::utility_ai::{PopAction, UtilityWeights};
use bevy_ecs::prelude::*;
use rand::Rng;

/// A pop's individual name.
#[derive(Component, Clone, Debug)]
pub struct PopName(pub String);

const POP_NAMES: &[&str] = &[
    "Ada", "Bryn", "Cole", "Dara", "Eli", "Fern", "Gale", "Hana", "Iris", "Joss", "Kael", "Luna",
    "Milo", "Neva", "Orin", "Pax", "Quinn", "Rhea", "Sable", "Tarn", "Uma", "Vale", "Wren", "Xia",
    "Yara", "Zev",
];

/// Pick a random name from the hardcoded list.
fn generate_name<R: Rng>(rng: &mut R) -> PopName {
    let idx = rng.gen_range(0..POP_NAMES.len());
    PopName(POP_NAMES[idx].to_string())
}

/// Marker component for pop entities.
///
/// This component identifies an entity as a "Citizen" of the colony. It is the
/// primary query filter for most simulation systems (needs, AI, jobs).
///
/// # Examples
///
/// Querying all pops:
/// ```
/// use scale::layer1::pop::Pop;
/// use bevy_ecs::prelude::*;
///
/// fn count_pops(query: Query<&Pop>) -> usize {
///     query.iter().count()
/// }
/// ```
#[derive(Component)]
pub struct Pop;

/// Movement speed of a pop.
///
/// Speed is a multiplier for movement. Base speed is 1.0 (1 tile per tick).
/// Values < 1.0 slow down movement (e.g., 0.5 moves every 2 ticks).
/// Values > 1.0 speed up movement (e.g., 2.0 moves 2 tiles per tick).
#[derive(Component, Debug, Clone)]
pub struct Speed {
    /// Base speed multiplier (usually 1.0).
    pub base: f32,
    /// Current effective speed multiplier.
    pub current: f32,
    /// Accumulator for fractional movement.
    pub accumulator: f32,
}

impl Default for Speed {
    fn default() -> Self {
        Self {
            base: 1.0,
            current: 1.0,
            accumulator: 0.0,
        }
    }
}

/// Spawn 5 initial pops at random walkable positions.
///
/// This function attempts to find valid starting locations for the initial colony.
/// It will retry random coordinates until it finds a tile that is:
/// * Within bounds
/// * Not Water
/// * Not Rock
///
/// If it fails to find a spot after `MAX_ATTEMPTS` (1000), it gives up for that pop.
pub fn spawn_initial_pops(world: &mut World) {
    let mut rng = rand::thread_rng();
    spawn_initial_pops_internal(world, &mut rng);
}

fn spawn_initial_pops_internal<R: Rng>(world: &mut World, rng: &mut R) {
    const MAX_ATTEMPTS: usize = 1000;

    // Get dimensions first to release borrow
    let (width, height) = {
        let terrain = world.resource::<TerrainGrid>();
        (terrain.width, terrain.height)
    };

    // Safe bounds for GridPosition (i32)
    // If map is larger than i32::MAX, we just spawn within the i32 limit
    // because GridPosition cannot represent coordinates beyond that anyway.
    let max_x = i32::try_from(width).unwrap_or(i32::MAX);
    let max_y = i32::try_from(height).unwrap_or(i32::MAX);

    let mut spawned = 0;
    let mut attempts = 0;
    // Safety: we assume there is at least one walkable tile to avoid infinite loop.
    // In a real game we might want a timeout or more robust search.
    while spawned < 5 {
        attempts += 1;
        if attempts >= MAX_ATTEMPTS {
            break;
        }

        let x = rng.gen_range(0..max_x);
        let y = rng.gen_range(0..max_y);

        // Check terrain type in a separate scope to handle borrowing
        let is_walkable = {
            let terrain = world.resource::<TerrainGrid>();
            #[allow(clippy::cast_sign_loss)]
            terrain
                .get(x as usize, y as usize)
                .is_some_and(|t| t != TerrainType::Water && t != TerrainType::Rock)
        };

        if is_walkable {
            world.spawn((
                Pop,
                generate_name(rng),
                GridPosition { x, y },
                Health::default(),
                Needs::default(),
                Memories::default(),
                Skills::default(),
                Speed::default(),
                PopAction::default(),
                UtilityWeights::default(),
                Knowledge::default(),
            ));
            spawned += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType, generate_terrain};

    #[test]
    fn test_pop_component_exists() {
        let mut world = World::new();
        let entity = world.spawn(Pop).id();

        assert!(world.get::<Pop>(entity).is_some());
    }

    #[test]
    fn test_spawn_initial_pops_count() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5, "Should spawn exactly 5 pops");
    }

    #[test]
    fn test_spawn_initial_pops_have_positions() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let mut query = world.query::<(&Pop, &GridPosition)>();
        let all_have_positions = query.iter(&world).count() == 5;
        assert!(all_have_positions, "All pops should have GridPosition");
    }

    #[test]
    fn test_spawn_initial_pops_have_skills() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let mut query = world.query::<(&Pop, &Skills)>();
        let all_have_skills = query.iter(&world).count() == 5;
        assert!(all_have_skills, "All pops should have Skills component");
    }

    #[test]
    fn test_spawn_only_on_walkable_terrain() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        // Initialize query before borrowing resource to avoid conflict
        let mut query = world.query::<(&Pop, &GridPosition)>();
        let terrain = world.resource::<TerrainGrid>();

        for (_, pos) in query.iter(&world) {
            let x = usize::try_from(pos.x).expect("Pop x should be non-negative");
            let y = usize::try_from(pos.y).expect("Pop y should be non-negative");
            if let Some(tile_type) = terrain.get(x, y) {
                assert_ne!(tile_type, TerrainType::Water, "Pop spawned on water");
                assert_ne!(tile_type, TerrainType::Rock, "Pop spawned on rock");
            }
        }
    }

    #[test]
    fn test_spawn_within_terrain_bounds() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        for (_, pos) in world.query::<(&Pop, &GridPosition)>().iter(&world) {
            assert!(pos.x >= 0 && pos.x < 80, "Pop x out of bounds");
            assert!(pos.y >= 0 && pos.y < 50, "Pop y out of bounds");
        }
    }

    #[test]
    fn test_spawn_initial_pops_retries() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut world = World::new();
        let width = 10;
        let height = 10;
        let mut tiles = vec![TerrainType::Water; width * height];
        // Only one walkable tile
        tiles[0] = TerrainType::Grass;
        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        // Use internal spawning with seeded RNG to ensure we hit the 1/100 chance enough times
        // or effectively test the retry logic deterministically.
        // With Seed 42, we know it works.
        let mut rng = StdRng::seed_from_u64(42);
        spawn_initial_pops_internal(&mut world, &mut rng);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);

        // All pops should be at (0,0)
        for (_, pos) in world.query::<(&Pop, &GridPosition)>().iter(&world) {
            assert_eq!(pos.x, 0);
            assert_eq!(pos.y, 0);
        }
    }

    #[test]
    fn test_spawn_initial_pops_internal_with_custom_rng() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        // Use seeded RNG for determinism
        let mut rng = StdRng::seed_from_u64(42);
        spawn_initial_pops_internal(&mut world, &mut rng);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5, "Should spawn exactly 5 pops");
    }

    #[test]
    fn test_spawn_with_mixed_terrain() {
        let mut world = World::new();
        let width = 10;
        let height = 10;
        let mut tiles = vec![TerrainType::Grass; width * height];

        // Add some non-walkable tiles to force retries
        tiles[5] = TerrainType::Water;
        tiles[15] = TerrainType::Rock;
        tiles[25] = TerrainType::Water;
        tiles[35] = TerrainType::Rock;

        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);

        // Verify no pops on water or rock
        let mut query = world.query::<(&Pop, &GridPosition)>();
        let terrain = world.resource::<TerrainGrid>();
        for (_, pos) in query.iter(&world) {
            let x = usize::try_from(pos.x).expect("x should be non-negative");
            let y = usize::try_from(pos.y).expect("y should be non-negative");
            if let Some(tile) = terrain.get(x, y) {
                assert_ne!(tile, TerrainType::Water);
                assert_ne!(tile, TerrainType::Rock);
            }
        }
    }

    #[test]
    fn test_spawn_internal_multiple_attempts() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut world = World::new();
        let width = 20;
        let height = 20;
        let mut tiles = vec![TerrainType::Water; width * height];

        // Create a sparse walkable area (only 20 out of 400 tiles)
        for i in 0..20 {
            tiles[i * 20] = TerrainType::Grass;
        }

        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        let mut rng = StdRng::seed_from_u64(123);
        spawn_initial_pops_internal(&mut world, &mut rng);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_spawn_on_grass() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        };
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_spawn_on_dirt() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Dirt; 100];
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        };
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_pop_component_on_entity() {
        let mut world = World::new();
        let entity = world.spawn((Pop, GridPosition { x: 1, y: 2 })).id();

        assert!(world.get::<Pop>(entity).is_some());
        let pos = world.get::<GridPosition>(entity).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 2);
    }

    use rand::RngCore;

    struct LimitedRng {
        count: usize,
        limit: usize,
    }

    impl RngCore for LimitedRng {
        fn next_u32(&mut self) -> u32 {
            #[allow(clippy::cast_possible_truncation)]
            {
                self.next_u64() as u32
            }
        }

        fn next_u64(&mut self) -> u64 {
            self.count += 1;
            assert!(
                self.count <= self.limit,
                "Too many RNG calls - infinite loop detected"
            );
            0
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            for b in dest {
                *b = 0;
            }
            self.next_u64();
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    #[test]
    fn test_spawn_initial_pops_terminates_on_full_map_repro() {
        let mut world = World::new();
        // 1x1 map
        let width = 1;
        let height = 1;
        let tiles = vec![TerrainType::Water; width * height];
        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        let mut rng = LimitedRng {
            count: 0,
            limit: 2000,
        };
        spawn_initial_pops_internal(&mut world, &mut rng);

        // Verify that we didn't spawn anything (because map is full of water)
        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_pop_name_component() {
        let mut world = World::new();
        let entity = world.spawn(PopName("Ada".to_string())).id();
        let name = world.get::<PopName>(entity).unwrap();
        assert_eq!(name.0, "Ada");
    }

    #[test]
    fn test_spawn_initial_pops_have_names() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let mut query = world.query::<(&Pop, &PopName)>();
        let count = query.iter(&world).count();
        assert_eq!(count, 5, "All 5 pops should have names");
    }

    #[test]
    fn test_generated_names_not_empty() {
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let name = generate_name(&mut rng);
            assert!(!name.0.is_empty(), "Generated name should not be empty");
        }
    }
}
