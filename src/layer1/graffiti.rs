//! Graffiti and Signage system (Spec 144).
//!
//! Pops can leave permanent "Markings" on walls and buildings based on their Morale.
//! These markings persist and influence other Pops who see them.

use crate::layer1::building::OccupiedTiles;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Types of graffiti that can be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraffitiType {
    /// Placed by pops with low morale. Lowers morale of observers.
    Vandalism,
    /// Placed by pops with high morale. Increases morale of observers.
    Inspiration,
    /// Artistic expression (requires Artist trait).
    Mural,
    /// Faction propaganda (requires Rebel trait or Faction Leader).
    Propaganda,
}

/// A graffiti marking on a tile.
#[derive(Debug, Clone)]
pub struct Graffiti {
    /// The type of graffiti.
    pub graffiti_type: GraffitiType,
    /// Remaining duration in ticks.
    pub decay: f32,
    /// Morale impact when observed.
    pub modifier: f32,
}

/// Resource storing all active graffiti on the map.
#[derive(Resource, Default)]
pub struct GraffitiMap {
    /// Map from grid coordinates to graffiti data.
    pub markings: HashMap<(i32, i32), Graffiti>,
}

/// System to place graffiti based on pop morale.
pub fn graffiti_placement_system(
    mut graffiti_map: ResMut<GraffitiMap>,
    pops: Query<(&GridPosition, &Morale), With<crate::layer1::pop::Pop>>,
    occupied_tiles: Option<Res<OccupiedTiles>>,
) {
    let mut rng = rand::thread_rng();
    // 1% chance per tick to place graffiti if conditions are met
    // This prevents map from being covered instantly
    let placement_chance = 0.01;

    // We use OccupiedTiles for fast lookup if available.
    // If not (e.g. in tests), we might need another way or just skip.
    // Tests might mock OccupiedTiles or we can use the slow query method if needed,
    // but using OccupiedTiles is better for performance.
    // For tests that don't add OccupiedTiles, we might fail to place graffiti if we strictly rely on it.
    // The RED phase tests added `Building` entities but didn't populate `OccupiedTiles`.
    // I should probably update the tests to populate `OccupiedTiles` or use a fallback.
    // Given the codebase uses `OccupiedTiles` for validity, let's rely on it.
    // But I must update tests! (Or I can check if I can query buildings directly as a fallback, which is slow).

    // Actually, let's look at the tests I wrote.
    // `world.spawn((Building...))`
    // This does NOT automatically update `OccupiedTiles` unless `try_place_building` was used or a system runs.
    // My tests manually spawn.
    // So I should stick to the "slow" way (iterating buildings) OR update tests.
    // Updating tests is cleaner but "REFACTOR" step.
    // OR I can just iterate buildings in the system? No, that's O(P*B).

    // Compromise: In the system, I will assume `OccupiedTiles` is the source of truth for "is there a building".
    // I will update the tests to manually insert into `OccupiedTiles`.
    // Wait, I can't update tests in this `write_file` call easily without rewriting the whole file.
    // I'll rewrite the whole file including tests.

    let Some(occupied) = occupied_tiles else {
        return;
    };

    for (pos, morale) in &pops {
        // Optimization: Quick RNG check first
        if rng.gen_range(0.0..1.0) > placement_chance {
            continue;
        }

        let graffiti_type = if morale.value < 0.2 {
            GraffitiType::Vandalism
        } else if morale.value > 0.8 {
            GraffitiType::Inspiration
        } else {
            continue;
        };

        // Find adjacent building
        let neighbors = [
            (pos.x, pos.y - 1),
            (pos.x + 1, pos.y),
            (pos.x, pos.y + 1),
            (pos.x - 1, pos.y),
        ];

        for target in neighbors {
            if occupied.0.contains(&target) {
                // Found a wall/building!
                let (decay, modifier) = match graffiti_type {
                    GraffitiType::Vandalism => (1000.0, -0.05),
                    GraffitiType::Inspiration => (1000.0, 0.05),
                    _ => (1000.0, 0.0),
                };

                graffiti_map.markings.insert(
                    target,
                    Graffiti {
                        graffiti_type,
                        decay,
                        modifier,
                    },
                );
                break; // Only place one
            }
        }
    }
}

/// System to apply mood modifiers when observing graffiti.
pub fn graffiti_observation_system(
    graffiti_map: Res<GraffitiMap>,
    mut pops: Query<(&GridPosition, &mut Morale)>,
) {
    for (pos, mut morale) in &mut pops {
        // Check adjacent tiles for graffiti
        // Similar neighbor check
        let neighbors = [
            (pos.x, pos.y - 1),
            (pos.x + 1, pos.y),
            (pos.x, pos.y + 1),
            (pos.x - 1, pos.y),
        ];

        for target in neighbors {
            if let Some(graffiti) = graffiti_map.markings.get(&target) {
                // Apply modifier
                // We should check if we already have this specific modifier to avoid stacking infinitely?
                // MoodModifier doesn't have a unique ID, just label.
                // We can check if we have a modifier with this label recently?
                // Or just add it with a short duration.

                let label = match graffiti.graffiti_type {
                    GraffitiType::Vandalism => "Saw Vandalism",
                    GraffitiType::Inspiration => "Saw Inspiration",
                    GraffitiType::Mural => "Saw Mural",
                    GraffitiType::Propaganda => "Saw Propaganda",
                };

                // Check if already affected
                let already_affected = morale.modifiers.iter().any(|m| m.label == label);
                if !already_affected {
                    morale.add_modifier(MoodModifier {
                        label: label.to_string(),
                        value: graffiti.modifier,
                        duration: 50, // Short duration
                    });
                }
            }
        }
    }
}

/// System to decay graffiti over time.
pub fn graffiti_decay_system(world: &mut World) {
    // We need to mutate GraffitiMap, so we extract it.
    // Using `world` directly allows this system to be registered as `fn(world: &mut World)` if needed,
    // but usually systems take `ResMut`.
    // The spec defined it as `fn(&mut World)`. I'll stick to that signature for compatibility with spec,
    // but normally in Bevy it's `ResMut<GraffitiMap>`.
    // However, looking at the code, it's cleaner to use `ResMut`.
    // But I must match the signature used in the Test if I export it.
    // The test calls `super::graffiti_decay_system(&mut world)`.
    // So I must keep the signature.

    world.resource_scope(|_world, mut map: Mut<GraffitiMap>| {
        let mut to_remove = Vec::new();
        for (pos, graffiti) in &mut map.markings {
            graffiti.decay -= 1.0;
            if graffiti.decay <= 0.0 {
                to_remove.push(*pos);
            }
        }
        for pos in to_remove {
            map.markings.remove(&pos);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_graffiti_map_starts_empty() {
        let map = GraffitiMap::default();
        assert!(map.markings.is_empty());
    }

    #[test]
    fn test_low_mood_pop_places_vandalism() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        // Setup OccupiedTiles manually for test
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        // Spawn Wall at (5,5)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Sad Pop at (5,4) (adjacent)
        // Mood 10.0 in spec -> Morale 0.1
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Morale {
                value: 0.1,
                ..Default::default()
            },
        ));

        // Run placement system repeatedly to overcome 1% chance
        // Or mock RNG? We can't easily mock `rand::thread_rng` inside the system without dependency injection.
        // For test purposes, we can try running it many times, or modify the system to take a config.
        // Or we can just trust the logic if we could force probability.
        // Given I implemented `let placement_chance = 0.01;`, this test will flake or fail if run once.
        // I should probably make `placement_chance` configurable or 1.0 in tests.
        // But `graffiti_placement_system` doesn't take config.

        // HACK: Run it 1000 times.
        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);

        let mut placed = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            if world
                .resource::<GraffitiMap>()
                .markings
                .contains_key(&(5, 5))
            {
                placed = true;
                break;
            }
        }

        assert!(placed, "Graffiti should be placed eventually");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5));
        assert!(graffiti.is_some());
        assert_eq!(graffiti.unwrap().graffiti_type, GraffitiType::Vandalism);
    }

    #[test]
    fn test_high_mood_pop_places_inspiration() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Mood 95.0 in spec -> Morale 0.95
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Morale {
                value: 0.95,
                ..Default::default()
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);

        let mut placed = false;
        for _ in 0..1000 {
            schedule.run(&mut world);
            if world
                .resource::<GraffitiMap>()
                .markings
                .contains_key(&(5, 5))
            {
                placed = true;
                break;
            }
        }

        assert!(placed, "Inspiration should be placed eventually");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5));
        assert!(graffiti.is_some());
        assert_eq!(graffiti.unwrap().graffiti_type, GraffitiType::Inspiration);
    }

    #[test]
    fn test_graffiti_observation_affects_mood() {
        let mut world = World::new();

        // Pre-place Vandalism
        let mut map = GraffitiMap::default();
        map.markings.insert(
            (5, 5),
            Graffiti {
                graffiti_type: GraffitiType::Vandalism,
                decay: 100.0,
                modifier: -0.05,
            },
        );
        world.insert_resource(map);

        // Spawn Pop at (5,4) observing (5,5)
        let pop_id = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 4 },
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Run observation system
        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_observation_system);
        schedule.run(&mut world);

        // Check Mood reduced
        let morale = world.get::<Morale>(pop_id).unwrap();
        assert!(
            !morale.modifiers.is_empty(),
            "Should have added a mood modifier"
        );
        let modifier = &morale.modifiers[0];
        // Value should be -0.05
        assert!(
            (modifier.value - -0.05).abs() < f32::EPSILON,
            "Modifier should be -0.05"
        );
        assert_eq!(modifier.label, "Saw Vandalism");
    }

    #[test]
    fn test_graffiti_decay() {
        let mut world = World::new();
        let mut map = GraffitiMap::default();
        map.markings.insert(
            (0, 0),
            Graffiti {
                graffiti_type: GraffitiType::Vandalism,
                decay: 1.0, // Almost gone
                modifier: -0.05,
            },
        );
        world.insert_resource(map);

        // Run decay system
        super::graffiti_decay_system(&mut world);

        let map = world.resource::<GraffitiMap>();
        assert!(
            map.markings.is_empty(),
            "Graffiti should be removed after decay reaches 0"
        );
    }
}
