# 317: The Orbital Junkyard

## 1. Overview

An orbital debris field surrounding the planet rains down scrap onto the colony (Layer 1) and poses a hazard to any passing ships (Layer 2). This field can be incredibly lucrative for harvesting advanced alloys but carries a high risk of crushing structures below.

## 2. Dependencies

- `018` Mining Resources (for scrap collection)
- `184` Orbital Debris (for Layer 2 mechanics)
- `006` Building Placement (for structural damage)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_debris_rain_spawn() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_debris_rain);

        app.world_mut().insert_resource(DebrisRainChance(1.0)); // Always rain

        let grid = app.world_mut().spawn(TerrainGrid { width: 10, height: 10 }).id();

        // Act
        app.update();

        // Assert
        let scrap_count = app.world_mut().query::<&ScrapPile>().iter(&app.world()).count();
        assert!(scrap_count > 0, "Debris should have spawned a scrap pile on the grid");
    }

    #[test]
    fn test_debris_crushes_building() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_falling_debris);

        let building_pos = GridPosition { x: 5, y: 5 };
        let building = app.world_mut().spawn((
            building_pos,
            Health { current: 100.0, max: 100.0 },
            Structure,
        )).id();

        let debris = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            FallingDebris { damage: 150.0 }, // Fatal damage
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<Health>(building).is_none(), "Building should have been destroyed by falling debris");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ScrapPile {
    pub amount: u32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Structure;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct FallingDebris {
    pub damage: f32,
}

#[derive(Resource)]
pub struct DebrisRainChance(pub f32);

#[derive(Component)]
pub struct TerrainGrid {
    pub width: u32,
    pub height: u32,
}

pub fn process_debris_rain(
    mut commands: Commands,
    grid_query: Query<&TerrainGrid>,
    chance: Res<DebrisRainChance>,
) {
    if rand::random::<f32>() < chance.0 {
        if let Ok(grid) = grid_query.get_single() {
            let x = rand::random::<u32>() % grid.width;
            let y = rand::random::<u32>() % grid.height;

            // Spawn falling debris which resolves next tick
            commands.spawn((
                GridPosition { x: x as i32, y: y as i32 },
                FallingDebris { damage: 50.0 }, // Base random damage
            ));
        }
    }
}

pub fn process_falling_debris(
    mut commands: Commands,
    debris_query: Query<(Entity, &GridPosition, &FallingDebris)>,
    mut structure_query: Query<(Entity, &GridPosition, &mut Health), With<Structure>>,
) {
    for (debris_entity, debris_pos, debris) in debris_query.iter() {
        let mut hit_structure = false;

        for (struct_entity, struct_pos, mut health) in structure_query.iter_mut() {
            if struct_pos.x == debris_pos.x && struct_pos.y == debris_pos.y {
                health.current -= debris.damage;
                hit_structure = true;

                if health.current <= 0.0 {
                    commands.entity(struct_entity).despawn();
                }
            }
        }

        if !hit_structure {
            // Spawn scrap on empty tiles
            commands.spawn((
                GridPosition { x: debris_pos.x, y: debris_pos.y },
                ScrapPile { amount: 10 },
            ));
        }

        // Remove the falling debris hazard
        commands.entity(debris_entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Grid Validation:** Prevent debris from spawning out of bounds or on unwalkable tiles. Ensure valid targeting before falling.
- **Damage Component:** The `Health` logic should use standard event buses for damage `DamageEvent` and destruction `EntityDestroyedEvent` instead of handling it directly here.
- **Layer 2 Integration:** Create a system tying the intensity of the orbital debris (Layer 2) to the `DebrisRainChance` on Layer 1. More orbital battles = more scrap.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Debris periodically spawns on the map.
- [ ] Structures under falling debris take significant damage and are destroyed if health < 0.
- [ ] Tiles hit without structures spawn usable `ScrapPile` resources.

## 7. Technical Guidance

- Consider defining `DebrisRainChance` as a map-wide condition updated by events on Layer 2 (e.g., when a ship is destroyed in orbit).
- Debris size could vary, with small debris dealing minor damage and huge chunks outright flattening buildings.

## 8. Questions

*Builder: add questions here if spec is unclear.*
