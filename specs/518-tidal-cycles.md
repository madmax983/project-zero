# Spec 518: Tidal Cycles

## 1. Overview
Living on a world that breathes. The land is not safe, but it is rich. Water level rises and falls periodically (Daily/Seasonally). "Low Tide" reveals resources (shells, salt) and pathable terrain. "High Tide" floods low ground, drowning land-walkers and disabling non-waterproof buildings. This creates tension between building safe/high (limited space) or risky/low (rich resources).

**Layer:** 1
**Fantasy:** The land is not safe, but it is rich.

## 2. Dependencies
- `063` Atmospheric Simulation (Tide level application)
- `120` Crop Diversity (Flood impact on crops)
- `018` Mining Resources (Revealed resources)
- `034` Pop Health and Damage (Drowning mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tide_floods_low_elevation_tiles() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TideLevel { current_height: 5.0 });
        app.add_systems(Update, apply_tidal_flooding);

        let tile = app.world_mut().spawn((
            Tile,
            Elevation { height: 2.0 },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(tile).has::<Flooded>(), "Tile below tide level should flood");
    }

    #[test]
    fn test_high_elevation_tiles_remain_dry() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TideLevel { current_height: 5.0 });
        app.add_systems(Update, apply_tidal_flooding);

        let tile = app.world_mut().spawn((
            Tile,
            Elevation { height: 10.0 },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(!app.world().entity(tile).has::<Flooded>(), "Tile above tide level should stay dry");
    }

    #[test]
    fn test_flooded_buildings_are_disabled() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, disable_flooded_buildings);

        let building = app.world_mut().spawn((
            Building,
            Flooded,
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(building).has::<Disabled>(), "Flooded buildings should be disabled");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct TideLevel {
    pub current_height: f32,
}

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct Elevation {
    pub height: f32,
}

#[derive(Component)]
pub struct Flooded;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Disabled;

pub fn apply_tidal_flooding(
    mut commands: Commands,
    tide: Res<TideLevel>,
    query: Query<(Entity, &Elevation), With<Tile>>,
) {
    for (entity, elevation) in query.iter() {
        if elevation.height <= tide.current_height {
            commands.entity(entity).insert(Flooded);
        } else {
            commands.entity(entity).remove::<Flooded>();
        }
    }
}

pub fn disable_flooded_buildings(
    mut commands: Commands,
    query: Query<Entity, (With<Building>, With<Flooded>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Disabled);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - Water flow should probably propagate across adjacent tiles instead of instantly flooding all tiles below the threshold globally, although global threshold is simpler and less expensive.
  - Flooded buildings should regain function once the tide recedes (`Flooded` is removed). Needs a complementary `enable_dry_buildings` system.
- **Code Smells:**
  - `Disabled` component usage must integrate correctly with `Building Work AI` so workers know not to assign themselves jobs at submerged buildings.
- **Performance:**
  - Re-evaluating the entire grid of tiles every frame is extremely slow. `apply_tidal_flooding` should only run on `Changed<TideLevel>` or on a timer (e.g., once every few seconds).
- **API Improvements:**
  - Instead of standard `Flooded`, consider a component that also stores depth, e.g. `WaterDepth(f32)`, which opens up mechanics like "shallow wading" vs "deep swimming".

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for tidal modules.
- [ ] `TideLevel` correctly updates on a daily or seasonal cycle.
- [ ] Low-elevation tiles gain the `Flooded` component when the tide rises above them.
- [ ] Buildings on flooded tiles are correctly disabled.
- [ ] Pops traversing flooded tiles suffer a speed penalty or drowning risk.

## 7. Technical Guidance
- **Gotchas:** Make sure paths are recalculated when tiles flood to avoid Pops mindlessly walking into deep water and drowning. Hook into the pathfinding grid cost updates.
- **Integration Points:** You will need to spawn "Tidal Flats" resources (like salt flats or rare shells) on tiles that frequently oscillate between wet and dry (`src/layer1/generation.rs`).

## 8. Questions
*Builder: Add questions here if spec is unclear.*
