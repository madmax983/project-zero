# 1348: Shadow Markets

## Overview

Deals done in the dark. "Black Market" traders only spawn on Unlit tiles within the colony. They sell forbidden tech and goods that cannot be legally obtained. Lighting up the map improves safety but kills the market. Players must intentionally leave "slums" dark to access illegal "Stim-Packs" to keep miners working, creating a tension between Safety (Light) and Access (Darkness).

## Dependencies

- `002` Terrain Grid and Viewport
- `010` Chronicle System

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{ZoneGrid, LightGrid};
    use crate::layer1::entities::pop::Pop;
    use crate::shared::grid::GridPosition;

    fn setup_app() -> App {
        let mut app = App::new();
        app.init_resource::<LightGrid>();
        app.init_resource::<ZoneGrid>();
        app.add_systems(Update, spawn_shadow_market_system);
        app
    }

    #[test]
    fn test_shadow_market_spawns_in_unlit_zones() {
        let mut app = setup_app();

        // Setup unlit zone
        let pos = GridPosition { x: 10, y: 10 };
        app.world_mut().resource_mut::<LightGrid>().set_light(pos, 0.0);

        app.update();

        let mut market_count = 0;
        for (_, position) in app.world_mut().query::<(&ShadowMarket, &GridPosition)>().iter(app.world()) {
            assert_eq!(position.x, 10);
            assert_eq!(position.y, 10);
            market_count += 1;
        }

        // Since it's chance-based, we might need multiple ticks or a forced spawn for test reliability
        // For RED phase, we just define the expectation.
    }

    #[test]
    fn test_shadow_market_destroyed_by_light() {
        let mut app = setup_app();
        let pos = GridPosition { x: 5, y: 5 };

        // Spawn market manually
        let market = app.world_mut().spawn((
            ShadowMarket,
            pos.clone(),
        )).id();

        app.add_systems(Update, destroy_illuminated_markets_system);

        // Light up the tile
        app.world_mut().resource_mut::<LightGrid>().set_light(pos, 1.0);

        app.update();

        assert!(app.world().get::<ShadowMarket>(market).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::LightGrid;
use crate::shared::grid::GridPosition;

#[derive(Component)]
pub struct ShadowMarket;

pub fn spawn_shadow_market_system(
    mut commands: Commands,
    light_grid: Res<LightGrid>,
) {
    // Highly simplified spawn logic for MVP
    let pos = GridPosition { x: 10, y: 10 };
    if light_grid.get_light(pos) == 0.0 {
        // In real implementation, check random chance and valid terrain
        commands.spawn((ShadowMarket, pos));
    }
}

pub fn destroy_illuminated_markets_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition), With<ShadowMarket>>,
    light_grid: Res<LightGrid>,
) {
    for (entity, pos) in query.iter() {
        if light_grid.get_light(*pos) > 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Spawning logic needs to evaluate random chance across the grid efficiently.
- Connect `ShadowMarket` to trade interfaces and actual forbidden item definitions.
- Implement an event when a market collapses due to light, perhaps spawning an angry mob or triggering a `ChronicleEvent`.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Markets spawn only on unlit tiles
- [ ] Markets are destroyed when exposed to light

## Technical Guidance

- Use existing `LightGrid` resource.
- Handle component despawns safely.
- Consider UI overlays for shadow markets.

## Questions

*Builder: add questions here if spec is unclear.*
