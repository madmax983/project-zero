# Specification: Orbital Shadow

## 1. Overview
**Layer:** 2 -> 1 (Orbit to Colony)
**Fantasy:** The space infrastructure is so massive it blocks the sun.
**Mechanic:** Large orbital stations or fleets cast dynamic shadows on the colony map, reducing solar power and temperature in those zones.
**Emergence:** You park your dreadnought fleet in orbit for repairs, and accidentally freeze your crops.
**Tension:** Defense positioning vs. Planetary agriculture.

This feature maps the `GridPosition` of large Layer 2 entities (Ships, Stations) down to Layer 1's `LightGrid` and `TemperatureGrid`. Tiles under the shadow experience massive light reduction and minor temperature drops.

## 2. Dependencies
- `LightGrid` and `TemperatureGrid` (Layer 1 Simulation)
- `OrbitalEntity` with `Mass` or `Size` (Layer 2)
- `GridPosition` mapping between layers

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::utils::HashMap;

    #[test]
    fn test_orbital_shadow_blocks_light() {
        let mut app = App::new();
        app.add_systems(Update, apply_orbital_shadows_system);

        let mut light_grid = LightGrid { levels: HashMap::new() };
        light_grid.levels.insert(GridPosition { x: 0, y: 0 }, 100.0);

        let mut shadow_map = OrbitalShadowMap { shadows: HashMap::new() };
        shadow_map.shadows.insert(GridPosition { x: 0, y: 0 }, 50.0); // 50% block

        app.world.insert_resource(light_grid);
        app.world.insert_resource(shadow_map);

        app.update();

        let grid = app.world.get_resource::<LightGrid>().unwrap();
        assert!(grid.levels[&GridPosition { x: 0, y: 0 }] < 100.0, "Shadow should reduce light");
    }

    #[test]
    fn test_orbital_shadow_drops_temperature() {
        let mut app = App::new();
        app.add_systems(Update, apply_orbital_shadows_system);

        let mut temp_grid = TemperatureGrid { temperatures: HashMap::new() };
        temp_grid.temperatures.insert(GridPosition { x: 10, y: 10 }, 25.0);

        let mut shadow_map = OrbitalShadowMap { shadows: HashMap::new() };
        shadow_map.shadows.insert(GridPosition { x: 10, y: 10 }, 50.0);

        app.world.insert_resource(temp_grid);
        app.world.insert_resource(shadow_map);

        app.update();

        let grid = app.world.get_resource::<TemperatureGrid>().unwrap();
        assert!(grid.temperatures[&GridPosition { x: 10, y: 10 }] < 25.0, "Shadow should drop temperature");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource)]
pub struct LightGrid {
    pub levels: HashMap<GridPosition, f32>,
}

#[derive(Resource)]
pub struct TemperatureGrid {
    pub temperatures: HashMap<GridPosition, f32>,
}

#[derive(Resource, Default)]
pub struct OrbitalShadowMap {
    pub shadows: HashMap<GridPosition, f32>, // Percent block (0.0 to 100.0)
}

pub fn apply_orbital_shadows_system(
    shadow_map: Res<OrbitalShadowMap>,
    mut light_grid: ResMut<LightGrid>,
    mut temp_grid: ResMut<TemperatureGrid>,
) {
    for (pos, block_percent) in shadow_map.shadows.iter() {
        // Reduce light based on block percentage
        if let Some(light) = light_grid.levels.get_mut(pos) {
            *light *= 1.0 - (block_percent / 100.0);
        }

        // Minor temperature drop (e.g., 0.1 degree per % blocked)
        if let Some(temp) = temp_grid.temperatures.get_mut(pos) {
            *temp -= block_percent * 0.1;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Shadow Movement:** Orbital bodies move. The shadow map should shift across the Layer 1 grid based on Layer 2 velocity vectors and time of day.
- **Solar Panels:** Ensure `apply_orbital_shadows_system` runs *before* the solar power calculation system in the Bevy schedule.
- **Edge Softening:** Shadows shouldn't be hard blocks. Use a gradient radius (100% block at center, 50% at edge).

## 6. Acceptance Criteria
- [ ] `OrbitalShadowMap` entries reduce values in the `LightGrid`.
- [ ] `OrbitalShadowMap` entries reduce values in the `TemperatureGrid`.
- [ ] System handles missing or sparse map entries correctly.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- **Scheduling:** Add this to a specific system set, such as `Layer1SystemSet::Environment`, ordered before `UpdatePowerGrid` and `CropGrowth`.
- **Scaling:** The size of a dreadnought in Layer 2 (e.g. 5x5) might translate to a 50x50 shadow on Layer 1 due to projection scaling. Use a constant `ORBITAL_PROJECTION_SCALE` for clarity.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
