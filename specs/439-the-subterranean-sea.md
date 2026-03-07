# 439: The Subterranean Sea

## Overview

Mining deep enough breaches an underground ocean, revealing new resources and terrors. It provides limitless water and unique aquatic food sources, but risks massive colony flooding if breached incorrectly, and occasionally spawns abyssal predators.

## Dependencies

- `018` Mining Resources
- `164` Modular Fauna

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, (subterranean_sea_discovery_system, subterranean_sea_flooding_system, subterranean_sea_predator_system));
        app
    }

    #[test]
    fn test_mining_breaches_sea() {
        let mut app = setup_test_app();

        let tile = app.world.spawn(TerrainGridTile { depth: 100, mined: true }).id();
        app.update();

        // Check if the tile turned into an AbyssalSeaTile
        assert!(app.world.get::<AbyssalSeaTile>(tile).is_some(), "Mining past depth threshold should breach the sea");
    }

    #[test]
    fn test_sea_provides_water() {
        let mut app = setup_test_app();

        app.world.spawn(AbyssalSeaTile);
        app.world.insert_resource(ColonyResources::default());

        app.update();

        let resources = app.world.resource::<ColonyResources>();
        assert!(resources.water > 0.0, "Sea tiles should passively provide water resources");
    }

    #[test]
    fn test_sea_spawns_predator() {
        let mut app = setup_test_app();

        // Spawn tile with max instability to guarantee a spawn in this deterministic test
        app.world.spawn((AbyssalSeaTile { instability: 100.0 }, Transform::from_translation(Vec3::ZERO)));

        app.update();

        let predators = app.world.query::<&AbyssalPredator>().iter(&app.world).count();
        assert_eq!(predators, 1, "Abyssal Sea should spawn a predator when instability reaches threshold");
    }

    #[test]
    fn test_incorrect_breach_causes_flooding() {
        let mut app = setup_test_app();

        // Spawn a tile that breaches incorrectly
        let tile = app.world.spawn((TerrainGridTile { depth: 100, mined: true }, BreachedIncorrectly)).id();

        app.update();

        // Neighboring tiles should get flooded
        // Implementation specifics will depend on grid
        assert!(app.world.get::<FloodedTile>(tile).is_some(), "Incorrect breach should flood the area");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TerrainGridTile {
    pub depth: i32,
    pub mined: bool,
}

#[derive(Component, Default)]
pub struct AbyssalSeaTile {
    pub instability: f32, // Reaches 100.0 to spawn predator deterministically
}

#[derive(Component)]
pub struct BreachedIncorrectly;

#[derive(Component)]
pub struct FloodedTile;

#[derive(Component)]
pub struct AbyssalPredator;

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub water: f32,
    pub food: f32,
}

pub fn subterranean_sea_discovery_system(
    mut commands: Commands,
    mut tiles: Query<(Entity, &TerrainGridTile)>,
) {
    let breach_depth = 100;

    for (entity, tile) in tiles.iter_mut() {
        if tile.mined && tile.depth >= breach_depth {
            commands.entity(entity).insert(AbyssalSeaTile::default());
        }
    }
}

pub fn subterranean_sea_flooding_system(
    mut commands: Commands,
    mut tiles: Query<(Entity, &TerrainGridTile), With<BreachedIncorrectly>>,
) {
    for (entity, _tile) in tiles.iter_mut() {
        commands.entity(entity).insert(FloodedTile);
        // Flood neighboring tiles
    }
}

pub fn subterranean_sea_predator_system(
    mut commands: Commands,
    mut tiles: Query<(Entity, &mut AbyssalSeaTile, &Transform)>,
    time: Res<Time>,
) {
    for (_entity, mut tile, transform) in tiles.iter_mut() {
        tile.instability += 1.0 * time.delta_seconds();

        if tile.instability >= 100.0 {
            commands.spawn((AbyssalPredator, transform.clone()));
            tile.instability = 0.0;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Tie `BreachedIncorrectly` to the skill level of the Pop doing the mining. Low-skill miners cause floods.
- Add specific `WaterPump` buildings that are required to safely harness the `water` resource from the `AbyssalSeaTile`, replacing the passive resource gain.
- Ensure the `AbyssalPredator` uses the existing `UtilityAI` to hunt nearby Pops.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Mining past `breach_depth` changes the tile into an `AbyssalSeaTile`.
- [ ] Unskilled or unlucky mining applies `FloodedTile` to the area.
- [ ] Sea tiles occasionally spawn hostile `AbyssalPredator` fauna.

## Technical Guidance

- Integrate with `Layer1Grid` or `TerrainGrid` (depending on the exact naming in `src/layer1/`) for depth checking and flood propagation.
- Leverage the existing `ColonyResources` struct for water.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
