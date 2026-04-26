# 1199: Orbital Shadow Economies

## 1. Overview
**Layer:** 2 -> 1 (Orbit to Colony)
**Fantasy:** Massive orbital structures inadvertently dictate the economic cycles of the planet below simply by casting shadows.
**Mechanic:** Megastructures in orbit (like massive solar arrays or shipyards) cast colossal, moving shadows across the planetary surface (Layer 1). Tiles in shadow lose solar power efficiency and agricultural yield, but become hotspots for illicit activities and "Shadow Markets." Crime rates spike in the darkness, but exotic, black-market goods become available only in these transient shadowed zones.
**Emergence:** You build a colossal, planet-spanning orbital ring to solve your energy crisis. It works, but it plunges an entire equatorial band of your planet into perpetual twilight. The agriculture collapses, but a massive, untaxable shadow economy erupts in the darkness, turning your former breadbasket into a sprawling, crime-ridden metropolis that produces illegal tech you secretly rely on.
**Tension:** Do you design orbital infrastructure around optimizing the surface ecosystem, or do you intentionally plunge sectors into darkness to cultivate black markets and exotic tech, accepting the crime and unrest that follows?

## 2. Dependencies
- `LightGrid` and `TemperatureGrid` (Layer 1 Environment)
- `OrbitalShadowMap` (from spec 335)
- Pop Utility AI (Crime / Illicit activity tasks)
- Market system (Black Market trades)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::utils::HashMap;

    #[test]
    fn test_shadow_increases_crime_probability() {
        let mut app = App::new();
        app.add_systems(Update, apply_shadow_economy_effects);

        // Setup a pop in a shadowed tile
        let pop_id = app.world.spawn((Pop, GridPosition { x: 5, y: 5 }, CrimePropensity(0.1))).id();

        let mut shadow_map = OrbitalShadowMap { shadows: HashMap::new() };
        shadow_map.shadows.insert(GridPosition { x: 5, y: 5 }, 80.0); // 80% shadow block
        app.world.insert_resource(shadow_map);

        app.update();

        // Crime propensity should increase due to the shadow
        let propensity = app.world.get::<CrimePropensity>(pop_id).unwrap();
        assert!(propensity.0 > 0.1, "Crime propensity should be elevated in shadowed areas");
    }

    #[test]
    fn test_shadow_market_spawn() {
        let mut app = App::new();
        app.add_systems(Update, spawn_shadow_markets_system);

        let mut shadow_map = OrbitalShadowMap { shadows: HashMap::new() };
        shadow_map.shadows.insert(GridPosition { x: 10, y: 10 }, 90.0); // Deep shadow
        app.world.insert_resource(shadow_map);

        // Add a random chance override to guarantee spawn for test
        app.world.insert_resource(ForceMarketSpawnConfig { force: true });

        app.update();

        // Assert that a shadow market entity exists at the shadowed location
        let mut market_query = app.world.query_filtered::<&GridPosition, With<ShadowMarket>>();
        let markets: Vec<_> = market_query.iter(&app.world).collect();
        assert_eq!(markets.len(), 1, "A shadow market should spawn in deep shadow");
        assert_eq!(markets[0].x, 10);
        assert_eq!(markets[0].y, 10);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;

// Assuming these exist from dependencies
#[derive(Component)]
pub struct Pop;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct CrimePropensity(pub f32);

#[derive(Component)]
pub struct ShadowMarket;

#[derive(Resource, Default)]
pub struct OrbitalShadowMap {
    pub shadows: HashMap<GridPosition, f32>, // Percent block (0.0 to 100.0)
}

#[derive(Resource, Default)]
pub struct ForceMarketSpawnConfig {
    pub force: bool,
}

pub fn apply_shadow_economy_effects(
    shadow_map: Res<OrbitalShadowMap>,
    mut pop_query: Query<(&GridPosition, &mut CrimePropensity), With<Pop>>,
) {
    for (pos, mut crime) in pop_query.iter_mut() {
        if let Some(shadow_pct) = shadow_map.shadows.get(pos) {
            // Elevate crime propensity based on how dark the shadow is
            // E.g., at 100% shadow, propensity increases by 0.5
            crime.0 += (shadow_pct / 100.0) * 0.5;
        }
    }
}

pub fn spawn_shadow_markets_system(
    mut commands: Commands,
    shadow_map: Res<OrbitalShadowMap>,
    force_config: Option<Res<ForceMarketSpawnConfig>>,
) {
    let force = force_config.map(|c| c.force).unwrap_or(false);

    for (pos, shadow_pct) in shadow_map.shadows.iter() {
        // Only deep shadows (e.g. >75%) can spawn black markets
        if *shadow_pct > 75.0 && force {
            commands.spawn((
                ShadowMarket,
                *pos,
            ));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate shadow duration: A shadow that passes quickly shouldn't immediately spawn a market. Add a `ShadowDurationGrid` to track how many consecutive ticks a tile has been dark.
- Shadow Markets should despawn if the shadow moves away and the area is exposed to light again, forcing the criminal element to relocate.
- Hook into the `UtilityAI` so pops actively seek out these transient markets to fulfill illicit needs.

## 6. Acceptance Criteria
- [ ] Pops in shadowed tiles have their `CrimePropensity` increased.
- [ ] `ShadowMarket` entities spawn in deep shadows (>75%).
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] Code has 0 clippy warnings.

## 7. Technical Guidance
- `apply_shadow_economy_effects` needs to run after `OrbitalShadowMap` is updated for the tick.
- Coordinate with the `Market` system so that `ShadowMarket`s offer distinct (illicit) goods compared to regular markets.
- Use `rand` for the actual market spawning logic (instead of just `ForceMarketSpawnConfig`), but ensure it can be deterministic/seeded for tests.

## 8. Questions
*Builder: Add any questions here.*
