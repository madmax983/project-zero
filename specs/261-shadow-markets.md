# 261: Shadow Markets

## Overview

"Deals done in the dark."

**Black Market Traders** spawn occasionally, but *only* on tiles with **Light Level 0**. They sell forbidden tech, contraband, or cheap resources without tax. However, they are fleeting. If the player builds a light source nearby or the sun rises (if outdoors), the trader vanishes immediately.

This forces players to leave parts of their colony in "The Dark" (Slums/Maintenance tunnels) if they want access to illicit goods, creating a literal "Shadow Economy".

## Dependencies

- `039` — Trade System (Merchant logic)
- `053` — Lighting System (Light detection)
- `203` — Prohibition & Contraband (Goods sold)

## RED Phase: Tests First

Write these tests in `src/layer1/economy/shadow_market_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, LightMap};
    use crate::layer1::economy::shadow::{ShadowTrader, spawn_shadow_trader_system, despawn_in_light_system};

    #[test]
    fn test_spawn_only_in_darkness() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);

        // (5,5) is Lit
        light_map.set(5, 5, 1.0);
        // (0,0) is Dark
        light_map.set(0, 0, 0.0);

        world.insert_resource(light_map);

        // Run spawner
        // We need to mock the "Spawn Request" event or logic
        // For test, assume system tries to spawn at random locations and we check validity.
        // Or simpler: Helper function `try_spawn_trader(&world, pos)` returns result.

        let success_lit = crate::layer1::economy::shadow::can_spawn_trader(&world, GridPosition { x: 5, y: 5 });
        assert!(!success_lit, "Should not spawn in light");

        let success_dark = crate::layer1::economy::shadow::can_spawn_trader(&world, GridPosition { x: 0, y: 0 });
        assert!(success_dark, "Should spawn in darkness");
    }

    #[test]
    fn test_despawn_when_lit() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        world.insert_resource(light_map);

        // Spawn trader in dark
        let trader = world.spawn((
            ShadowTrader,
            GridPosition { x: 5, y: 5 }
        )).id();

        // 1. Verify safe in dark
        let mut schedule = Schedule::default();
        schedule.add_systems(despawn_in_light_system);
        schedule.run(&mut world);
        assert!(world.get_entity(trader).is_some());

        // 2. Turn on lights
        let mut light_map = world.resource_mut::<LightMap>();
        light_map.set(5, 5, 1.0);

        // 3. Run system
        schedule.run(&mut world);

        // 4. Verify despawn
        assert!(world.get_entity(trader).is_none(), "Trader should flee light");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/economy/shadow.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::{GridPosition, LightMap};

#[derive(Component)]
pub struct ShadowTrader;

pub fn can_spawn_trader(world: &World, pos: GridPosition) -> bool {
    if let Some(light_map) = world.get_resource::<LightMap>() {
        return light_map.get(pos.x as u32, pos.y as u32) <= 0.1; // Tolerance
    }
    false
}

pub fn despawn_in_light_system(
    mut commands: Commands,
    query: Query<(Entity, &GridPosition), With<ShadowTrader>>,
    light_map: Res<LightMap>,
) {
    for (entity, pos) in query.iter() {
        if light_map.get(pos.x as u32, pos.y as u32) > 0.1 {
            // "The shadows flee!"
            commands.entity(entity).despawn_recursive();
            // TODO: Emit visual smoke effect
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Trader Inventory**: Integrate with `Merchant` component from `039`. Shadow traders should have `inventory` filled with `Contraband`.
- **Spawn Logic**: Create a `ShadowMarketManager` resource that periodically tries to spawn a trader on a random dark tile (pathable by pops).
- **Notification**: "A Shadow Market has opened in the Lower Decks."

## Acceptance Criteria

- [ ] `ShadowTrader` component exists.
- [ ] Spawning logic checks light level.
- [ ] Existing traders despawn if illuminated.
- [ ] Tests pass.

## Technical Guidance

- Ensure `LightMap` is up to date (after `DayNightCycle`).
- Use `despawn_recursive` to clean up any UI attached to the trader.
