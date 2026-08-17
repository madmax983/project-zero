# 1370: Civilizational Compost

## 1. Overview
**Layer:** 1

**Fantasy:** We stand on the shoulders of giants, quite literally.

**Mechanic:** Dead Pops and destroyed buildings decompose into "Nutrient-Rich Soil" or "Scrap Heaps" over time. Farming on a graveyard yields higher crop output but causes "Haunted" moods.

**Emergence:** You fight a desperate battle in your cornfield. The dead soldiers fertilize the next harvest. The colony unknowingly eats the dead to survive the winter.

**Tension:** Respect the dead (burial) vs. Use the dead (fertilizer).

## 2. Dependencies
- Base ECS system
- Terrain/Grid system
- Pop/Building death events
- Farming yield system
- Mood/Psychology system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            process_decomposition_system,
            apply_haunted_mood_system,
        ));
        app
    }

    #[test]
    fn test_corpse_decomposes_into_nutrient_soil() {
        let mut app = setup_app();

        // Spawn a decomposing corpse
        let corpse = app.world_mut().spawn((
            Corpse,
            Decomposition { time_left: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Spawn a terrain tile underneath
        let tile = app.world_mut().spawn((
            TerrainTile { fertility: 1.0, is_haunted: false },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Tick down decomposition
        let mut decomp = app.world_mut().get_mut::<Decomposition>(corpse).unwrap();
        decomp.time_left = 0.0;

        app.update();

        // Check if the terrain changed
        let terrain = app.world().get::<TerrainTile>(tile).unwrap();

        assert!(terrain.fertility > 1.0);
        assert!(terrain.is_haunted);

        // Corpse should be removed
        assert!(app.world().get::<Corpse>(corpse).is_none());
    }

    #[test]
    fn test_farming_on_haunted_soil_causes_mood_debuff() {
        let mut app = setup_app();

        // Spawn a haunted terrain tile
        app.world_mut().spawn((
            TerrainTile { fertility: 2.0, is_haunted: true },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn a farmer working on that tile
        let farmer = app.world_mut().spawn((
            Pop,
            Mood { value: 50.0 },
            CurrentJob { location: GridPosition { x: 5, y: 5 } },
        )).id();

        app.update();

        let mood = app.world().get::<Mood>(farmer).unwrap();

        // Farmer should lose mood
        assert!(mood.value < 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct Decomposition {
    pub time_left: f32,
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct TerrainTile {
    pub fertility: f32,
    pub is_haunted: bool,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood {
    pub value: f32,
}

#[derive(Component)]
pub struct CurrentJob {
    pub location: GridPosition,
}

pub fn process_decomposition_system(
    mut commands: Commands,
    mut corpses: Query<(Entity, &mut Decomposition, &GridPosition), With<Corpse>>,
    mut tiles: Query<(&mut TerrainTile, &GridPosition)>,
) {
    for (entity, mut decomp, pos) in corpses.iter_mut() {
        if decomp.time_left <= 0.0 {
            // Find the matching tile
            for (mut tile, tile_pos) in tiles.iter_mut() {
                if pos == tile_pos {
                    // Enrich soil and make it haunted
                    tile.fertility += 1.0;
                    tile.is_haunted = true;

                    // Remove corpse
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }
    }
}

pub fn apply_haunted_mood_system(
    mut farmers: Query<(&mut Mood, &CurrentJob), With<Pop>>,
    tiles: Query<(&TerrainTile, &GridPosition)>,
) {
    for (mut mood, job) in farmers.iter_mut() {
        for (tile, tile_pos) in tiles.iter() {
            if job.location == *tile_pos && tile.is_haunted {
                // Apply a small mood debuff per tick/cycle
                mood.value -= 0.5;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Use spatial hashing or a `GridMap` resource to look up tiles efficiently instead of iterating all tiles O(N*M).
- Expand logic to handle destroyed buildings turning into `ScrapHeaps`.
- Add an explicit `Grave` building that prevents decomposition but preserves morale.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] `Corpse` entities with `Decomposition` <= 0 are despawned.
- [ ] `TerrainTile` at the corpse's location gains `fertility` and is marked `is_haunted`.
- [ ] `Pop` entities working on an `is_haunted` tile lose `Mood`.

## 7. Technical Guidance
- Be careful with the `Mood` debuff frequency; it should apply periodically (e.g., once per day or shift), not every tick, to prevent immediate breakdown.
- Ensure the `fertility` bonus actually scales with the existing crop growth formulas.

## 8. Questions
*Builder: add questions here if spec is unclear.*
