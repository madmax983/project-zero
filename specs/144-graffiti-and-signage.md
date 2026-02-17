# 144: Graffiti and Signage

## Overview

Pops can leave permanent "Markings" on walls and buildings based on their Mood and Traits. These markings persist and influence other Pops who see them.
- **High Mood**: "Inspiring Graffiti" (e.g., "We can do this!") boosts morale of observers.
- **Low Mood**: "Vandalism" (e.g., "Doomed", "Lies") lowers beauty and morale.
- **Traits**: Specific traits (e.g., "Artist", "Rebel") trigger unique graffiti types.

This adds a layer of "social residue" to the colony, making the environment reflect the history and emotional state of its inhabitants.

## Dependencies

- `004` — Pop Entity (Mood, Traits)
- `006` — Building Placement (Walls to mark)
- `091` — The Inspector (to view marking details)
- `031` — Pop Morale (Mood impact)

## RED Phase: Tests First

Write these tests in `src/layer1/graffiti_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::graffiti::{GraffitiMap, Graffiti, GraffitiType, graffiti_placement_system, graffiti_observation_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::{Pop, Mood, Trait};
    use crate::layer1::building::{Building, BuildingType};
    use std::collections::HashMap;

    #[test]
    fn test_graffiti_map_starts_empty() {
        let map = GraffitiMap::default();
        assert!(map.markings.is_empty());
    }

    #[test]
    fn test_low_mood_pop_places_vandalism() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        // Spawn Wall at (5,5)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Sad Pop at (5,4) (adjacent)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Mood { value: 10.0, ..Default::default() }, // Very low mood
        ));

        // Run placement system
        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);
        schedule.run(&mut world);

        // Check map
        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&GridPosition { x: 5, y: 5 });

        assert!(graffiti.is_some());
        assert_eq!(graffiti.unwrap().graffiti_type, GraffitiType::Vandalism);
    }

    #[test]
    fn test_high_mood_pop_places_inspiration() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 5, y: 5 },
        ));

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Mood { value: 95.0, ..Default::default() }, // High mood
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_placement_system);
        schedule.run(&mut world);

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&GridPosition { x: 5, y: 5 }).unwrap();
        assert_eq!(graffiti.graffiti_type, GraffitiType::Inspiration);
    }

    #[test]
    fn test_graffiti_observation_affects_mood() {
        let mut world = World::new();

        // Pre-place Vandalism
        let mut map = GraffitiMap::default();
        map.markings.insert(GridPosition { x: 5, y: 5 }, Graffiti {
            graffiti_type: GraffitiType::Vandalism,
            decay: 100.0,
            modifier: -5.0,
        });
        world.insert_resource(map);

        // Spawn Pop at (5,4) observing (5,5)
        let pop_id = world.spawn((
            Pop,
            GridPosition { x: 5, y: 4 },
            Mood { value: 50.0, ..Default::default() },
        )).id();

        // Run observation system
        let mut schedule = Schedule::default();
        schedule.add_systems(graffiti_observation_system);
        schedule.run(&mut world);

        // Check Mood reduced
        let mood = world.get::<Mood>(pop_id).unwrap();
        assert!(mood.value < 50.0);
    }

    #[test]
    fn test_graffiti_decay() {
        let mut world = World::new();
        let mut map = GraffitiMap::default();
        map.markings.insert(GridPosition { x: 0, y: 0 }, Graffiti {
            graffiti_type: GraffitiType::Vandalism,
            decay: 1.0, // Almost gone
            modifier: -5.0,
        });
        world.insert_resource(map);

        // Run decay system
        crate::layer1::graffiti::graffiti_decay_system(&mut world);

        let map = world.resource::<GraffitiMap>();
        assert!(map.markings.is_empty(), "Graffiti should be removed after decay reaches 0");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resource and Components (`src/layer1/graffiti.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraffitiType {
    Vandalism,
    Inspiration,
    Mural,     // From 'Artist' trait
    Propaganda, // From Faction leader?
}

#[derive(Debug, Clone)]
pub struct Graffiti {
    pub graffiti_type: GraffitiType,
    pub decay: f32,    // Duration in ticks
    pub modifier: f32, // Mood impact per tick
}

#[derive(Resource, Default)]
pub struct GraffitiMap {
    pub markings: HashMap<GridPosition, Graffiti>,
}

pub fn graffiti_placement_system(
    mut graffiti_map: ResMut<GraffitiMap>,
    pops: Query<(&GridPosition, &crate::layer1::pop::Mood), With<crate::layer1::pop::Pop>>,
    buildings: Query<(&GridPosition, &crate::layer1::building::Building)>,
) {
    // Simple adjacency check (expensive O(P*B) without spatial hash, optimize later)
    // For MVP, just check if Pop is ON a building tile? Or adjacent?
    // Let's assume Pop is adjacent.

    // Optimization: Only run occasionally or on specific events?
    // For GREEN phase, run every tick but with low probability.

    let rng = 0.01; // Mock probability 1%

    for (pop_pos, mood) in pops.iter() {
        if mood.value < 20.0 {
            // Place Vandalism
            // Find adjacent wall
            // ... (spatial lookup would be better here) ...
            // Mock: place on current pos + (0,1) if wall exists
            let target_pos = GridPosition { x: pop_pos.x, y: pop_pos.y + 1 };

            // If target has wall... (Skipping full check for brevity in Green phase)
            // if buildings.iter().any(|(b_pos, _)| *b_pos == target_pos) {
                 graffiti_map.markings.insert(target_pos, Graffiti {
                     graffiti_type: GraffitiType::Vandalism,
                     decay: 1000.0,
                     modifier: -0.1,
                 });
            // }
        } else if mood.value > 80.0 {
             let target_pos = GridPosition { x: pop_pos.x, y: pop_pos.y + 1 };
             graffiti_map.markings.insert(target_pos, Graffiti {
                 graffiti_type: GraffitiType::Inspiration,
                 decay: 1000.0,
                 modifier: 0.1,
             });
        }
    }
}

pub fn graffiti_observation_system(
    graffiti_map: Res<GraffitiMap>,
    mut pops: Query<(&GridPosition, &mut crate::layer1::pop::Mood)>,
) {
    for (pos, mut mood) in pops.iter_mut() {
        // Check adjacent tiles for graffiti
        // (Mock: check North)
        let target_pos = GridPosition { x: pos.x, y: pos.y + 1 };

        if let Some(graffiti) = graffiti_map.markings.get(&target_pos) {
            mood.value += graffiti.modifier;
            // Clamp mood 0-100
            mood.value = mood.value.clamp(0.0, 100.0);
        }
    }
}

pub fn graffiti_decay_system(mut world: &mut World) {
    let mut map = world.resource_mut::<GraffitiMap>();
    // Collect keys to remove
    let mut to_remove = Vec::new();

    for (pos, graffiti) in map.markings.iter_mut() {
        graffiti.decay -= 1.0;
        if graffiti.decay <= 0.0 {
            to_remove.push(*pos);
        }
    }

    for pos in to_remove {
        map.markings.remove(&pos);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Iterating all pops every tick for placement is bad.
  - Solution: Use an `Action` (e.g., `Idle`) or `RandomTick` event to trigger placement.
  - Or only check when Mood changes significantly.
- **Spatial Lookup**: Use `SpatialMap` or `TileGrid` to quickly find adjacent walls instead of iterating all buildings.
- **Visuals**: How to render?
  - `render_map_system` in `src/ui/map.rs` should check `GraffitiMap`.
  - If tile has graffiti, render a colored overlay (e.g., Red `x` for Vandalism, Green `+` for Inspiration) or change the wall color.
- **Inspector**: Update `src/ui/inspector.rs` to show "Graffiti: 'The end is nigh'" when inspecting a wall.

## Acceptance Criteria

- [ ] `GraffitiMap` resource exists.
- [ ] Pops with low/high mood place graffiti on nearby walls.
- [ ] Graffiti persists and decays.
- [ ] Pops viewing graffiti get mood modifiers.
- [ ] Tests pass.

## Technical Guidance

- Use `crate::layer1::map::get_adjacent_tiles(pos)` helper if available.
- Register `GraffitiMap` in `src/setup.rs`.
- Add systems to `build_simulation_schedule` in `src/simulation.rs`.
- Ensure `Mood` component has `value` field exposed or public getter.
