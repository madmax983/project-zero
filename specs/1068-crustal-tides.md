# Specification: Crustal Tides (Layer 2 -> Layer 1)

## 1. Overview
**What:** Orbital mechanics from Layer 2 (like Moons or Gas Giants) induce "Ground Tides" on the Layer 1 colony map. These tides physically deform the terrain—causing fault lines to crack open during high tide (revealing magma or deep resources) and snap shut during low tide.
**Why:** To create tension between building on stable, less productive ground versus unstable, highly productive fault lines. It bridges Layer 2 orbital data with Layer 1 terrestrial mechanics.

## 2. Dependencies
- Layer 1 Terrain System (`TerrainGrid` and `TerrainType`)
- Building Damage System (`Health` component for structures)
- Layer 2 Orbital System (must provide a `TidalForce` resource or component)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::architecture::Health;
    use crate::layer2::orbit::TidalForce;
    use super::*;

    #[test]
    fn test_high_tide_opens_fissures() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut grid = TerrainGrid::new(10, 10);
        // Set up a fault line
        grid.set_type(5, 5, TerrainType::FaultLine(false)); // false = closed
        app.insert_resource(grid);
        app.insert_resource(TidalForce { strength: 0.8 }); // High tide threshold > 0.7

        app.add_systems(Update, process_crustal_tides);
        app.update();

        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get_type(5, 5), TerrainType::FaultLine(true)); // true = open (magma exposed)
    }

    #[test]
    fn test_low_tide_closes_fissures_and_damages_buildings() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut grid = TerrainGrid::new(10, 10);
        grid.set_type(5, 5, TerrainType::FaultLine(true)); // Start open
        app.insert_resource(grid);
        app.insert_resource(TidalForce { strength: 0.2 }); // Low tide threshold < 0.3

        let building = app.world_mut().spawn((
            Health { current: 100, max: 100 },
            crate::layer1::map::GridPosition { x: 5, y: 5 },
        )).id();

        app.add_systems(Update, process_crustal_tides);
        app.update();

        // Terrain should close
        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get_type(5, 5), TerrainType::FaultLine(false));

        // Building should take crush damage
        let health = app.world().get::<Health>(building).unwrap();
        assert!(health.current < 100, "Building should take damage when fissure closes");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::architecture::Health;
use crate::layer2::orbit::TidalForce;
use crate::layer1::map::GridPosition;

pub fn process_crustal_tides(
    mut terrain: ResMut<TerrainGrid>,
    tidal_force: Res<TidalForce>,
    mut buildings: Query<(&GridPosition, &mut Health)>,
) {
    let high_tide = tidal_force.strength > 0.7;
    let low_tide = tidal_force.strength < 0.3;

    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if let TerrainType::FaultLine(is_open) = terrain.get_type(x, y) {
                if high_tide && !is_open {
                    terrain.set_type(x, y, TerrainType::FaultLine(true));
                } else if low_tide && is_open {
                    terrain.set_type(x, y, TerrainType::FaultLine(false));

                    // Damage buildings on the closing fissure
                    for (pos, mut health) in buildings.iter_mut() {
                        if pos.x == x && pos.y == y {
                            health.current = health.current.saturating_sub(50);
                        }
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Hashing:** Iterating over all buildings for every closing fault tile is `O(T * B)`. Refactor to use a spatial hash map or the existing `BuildingGrid` to `O(1)` lookup buildings at `(x, y)`.
- **Event Driven Damage:** Instead of directly mutating `Health`, fire a `CrushDamageEvent { entity, amount }` to hook into standard damage pipelines and UI alerts.
- **Tidal Smoothing:** Rather than hard thresholds (0.3, 0.7), consider interpolating the opening/closing to provide warning to the player (e.g., `TerrainType::FaultLine { openness: f32 }`).

## 6. Acceptance Criteria (Testable!)
- [ ] `test_high_tide_opens_fissures` passes.
- [ ] `test_low_tide_closes_fissures_and_damages_buildings` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `crustal_tides.rs`.
- [ ] The terrain grid correctly updates its state based on `TidalForce`.

## 7. Technical Guidance
- **TerrainType Definition:** You may need to add `FaultLine(bool)` to the `TerrainType` enum in `crate::layer1::nature::terrain`.
- **Global Schedule tuple limit:** If adding this system to a large schedule tuple, consider grouping it in a `CrustalTidesPlugin` to avoid Bevy's 21-parameter limit.

## 8. Questions
*Builder: add questions here if spec is unclear.*
