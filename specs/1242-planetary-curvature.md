# 1242: Planetary Curvature

## 1. Overview
On small worlds (Moons/Asteroids), the Horizon is close. Line-of-Sight is limited by distance unless the observer is elevated (High Z-level). This forces players to build "Watchtowers" or "Skyscrapers" to push the horizon back and prevent ground-level turrets from being useless against long-range snipers.

## 2. Dependencies
- Layer 1 Map / `TerrainGrid` pathing and line-of-sight systems.
- Entity elevation tracking (`ZLevel` or `Elevation` component).
- `WorldSize` or `PlanetCurvature` resource/component.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_planetary_curvature_limits_los() {
        let mut app = App::new();
        // Setup mock curvature
        app.insert_resource(PlanetCurvature { horizon_distance_base: 10.0 });

        let observer = app.world_mut().spawn((Position { x: 0.0, y: 0.0 }, Elevation(0.0))).id();
        let target = app.world_mut().spawn((Position { x: 15.0, y: 0.0 }, Elevation(0.0))).id();

        // At ground level, distance 15 is > horizon 10, should not be visible
        assert!(!has_line_of_sight(&app, observer, target));
    }

    #[test]
    fn test_elevation_extends_horizon() {
        let mut app = App::new();
        app.insert_resource(PlanetCurvature { horizon_distance_base: 10.0 });

        // Observer is elevated, pushing horizon to e.g., 20
        let observer = app.world_mut().spawn((Position { x: 0.0, y: 0.0 }, Elevation(10.0))).id();
        let target = app.world_mut().spawn((Position { x: 15.0, y: 0.0 }, Elevation(0.0))).id();

        // Now visible
        assert!(has_line_of_sight(&app, observer, target));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PlanetCurvature {
    pub horizon_distance_base: f32,
}

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Elevation(pub f32);

pub fn has_line_of_sight(app: &App, observer: Entity, target: Entity) -> bool {
    let curvature = app.world().get_resource::<PlanetCurvature>().unwrap();
    let observer_pos = app.world().get::<Position>(observer).unwrap();
    let observer_elev = app.world().get::<Elevation>(observer).map(|e| e.0).unwrap_or(0.0);

    let target_pos = app.world().get::<Position>(target).unwrap();
    let target_elev = app.world().get::<Elevation>(target).map(|e| e.0).unwrap_or(0.0);

    let dx = observer_pos.x - target_pos.x;
    let dy = observer_pos.y - target_pos.y;
    let dist = (dx * dx + dy * dy).sqrt();

    // Simple horizon extension formula: base + elevation factor
    let effective_horizon = curvature.horizon_distance_base + (observer_elev * 1.0) + (target_elev * 1.0);

    dist <= effective_horizon
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Integrate this `has_line_of_sight` logic into the primary `TerrainGrid` line-of-sight and fog-of-war systems.
- **Performance:** Pre-calculate effective horizon for static buildings to avoid recalculating per tick.
- **Raycasting:** Ensure the simple distance check is combined with actual terrain blocking raycasts.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85%.
- [ ] Line of sight is correctly limited by curvature based on planet size.
- [ ] High elevation increases the observer's line of sight range.

## 7. Technical Guidance
- Integrate neatly with existing FoW (Fog of War) or vision grids.
- Add `Elevation` components to Watchtowers to allow them to see further.

## 8. Questions
*Builder: add questions here if spec is unclear.*

*Architect:* I will address any implementation questions as they arise. For the MVP, proceed with standard Bevy patterns.
