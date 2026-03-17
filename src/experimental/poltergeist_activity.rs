//! The Poltergeist Phenomenon (Nova Feature).
//!
//! # The Spark
//! We have a `StressTracker` system measuring the mental strain of the colony.
//! What if extreme, collective psychological pressure began to physically warp reality?
//!
//! # The Feature
//! When the colony's average stress exceeds a critical threshold, "Poltergeist Activity"
//! begins. Loose `Item` entities scattered across the colony will spontaneously and
//! violently teleport to random nearby tiles, causing logistical chaos and creating
//! a spooky, emergent narrative that the colony itself is haunted by their own despair.

use crate::layer1::items::Item;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

const POLTERGEIST_STRESS_THRESHOLD: f32 = 80.0; // 80% stress threshold for the colony
const ITEM_TELEPORT_CHANCE: f64 = 0.05; // 5% chance per tick for an item to jump
const JUMP_RADIUS: i32 = 3; // Maximum tiles an item can jump

/// System that calculates average colony stress and displaces items if it's too high.
pub fn poltergeist_activity_system(
    pops: Query<&StressTracker, With<Pop>>,
    mut items: Query<&mut GridPosition, With<Item>>,
    terrain: Res<TerrainGrid>,
) {
    if pops.is_empty() {
        return;
    }

    let mut total_stress = 0.0;
    let mut pop_count = 0;

    for stress in pops.iter() {
        total_stress += stress.accumulated_stress;
        pop_count += 1;
    }

    let avg_stress = total_stress / pop_count as f32;

    // If the colony is too stressed, reality starts to break down.
    if avg_stress >= POLTERGEIST_STRESS_THRESHOLD {
        let mut rng = rand::thread_rng();

        for mut pos in items.iter_mut() {
            if rng.gen_bool(ITEM_TELEPORT_CHANCE) {
                // The item violently jumps to a nearby location
                let dx = rng.gen_range(-JUMP_RADIUS..=JUMP_RADIUS);
                let dy = rng.gen_range(-JUMP_RADIUS..=JUMP_RADIUS);

                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                // Ensure it stays within bounds
                if new_x >= 0
                    && new_x < terrain.width as i32
                    && new_y >= 0
                    && new_y < terrain.height as i32
                {
                    pos.x = new_x;
                    pos.y = new_y;
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(poltergeist_activity_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::ItemType;

    use crate::layer1::terrain::TerrainType;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        world.insert_resource(terrain);
        world
    }

    #[test]
    fn test_poltergeist_activity_triggers_on_high_stress() {
        let mut world = setup_world();

        // High stress pop
        world.spawn((
            Pop,
            StressTracker {
                accumulated_stress: 95.0,
            },
        ));

        let item = world
            .spawn((
                Item {
                    item_type: ItemType::None,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run multiple times to overcome RNG
        let mut moved = false;
        for _ in 0..100 {
            world.run_system_once(poltergeist_activity_system).unwrap();
            let pos = world.get::<GridPosition>(item).unwrap();
            if pos.x != 5 || pos.y != 5 {
                moved = true;
                break;
            }
        }

        assert!(moved, "Item should have teleported under high stress");
    }

    #[test]
    fn test_poltergeist_activity_dormant_on_low_stress() {
        let mut world = setup_world();

        // Low stress pop
        world.spawn((
            Pop,
            StressTracker {
                accumulated_stress: 10.0,
            },
        ));

        let item = world
            .spawn((
                Item {
                    item_type: ItemType::None,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run multiple times to ensure it never moves
        for _ in 0..100 {
            world.run_system_once(poltergeist_activity_system).unwrap();
        }

        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5, "Item should not move under low stress");
    }
}
