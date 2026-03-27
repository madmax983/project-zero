# 659 - Orbital Bombardment

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Calling down the thunder. Ordering Layer 2 warships to fire on Layer 1 targets to obliterate threats, but accepting the risk of massive collateral damage.
**Mechanic:** Players can target specific Layer 1 tiles from Layer 2 warships. Bombardment causes massive damage in a blast radius, potentially terraforming the terrain. However, there's a "Scatter" chance influenced by atmospheric density and technology levels, which might result in the shot landing off-target.
**Emergence:** You call a strike on a bug hive. The shot scatters and vaporizes your own forward operating base.
**Tension:** Risk the "Rod from God" (friendly fire) or fight on the ground (attrition)?

## 2. Dependencies
- `099` Fleet Movement (Layer 2 warships existence)
- `002` Basic Map (TerrainGrid and health/damage system)
- `063` Atmospheric Simulation (Atmospheric density affects scatter)

## 3. RED Phase: Tests First

```rust
// tests/layer2/bombardment.rs

use bevy::prelude::*;
use scale::layer1::map::{TerrainGrid, TileType};
use scale::layer1::health::Health;
use scale::layer1::environment::AtmosphereGrid;
use scale::layer2::bombardment::{BombardmentEvent, execute_bombardment_system};

#[test]
fn test_bombardment_direct_hit() {
    let mut app = App::new();
    app.add_systems(Update, execute_bombardment_system);

    let mut grid = TerrainGrid::new(10, 10);
    // Target entity with health at 5,5
    let target_id = app.world_mut().spawn((Transform::from_xyz(5.0, 5.0, 0.0), Health { current: 100.0, max: 100.0 })).id();

    app.world_mut().insert_resource(grid);

    // Fire bombardment event directly at 5,5 with no scatter
    app.world_mut().send_event(BombardmentEvent {
        target: Vec2::new(5.0, 5.0),
        damage: 200.0,
        scatter_radius: 0.0, // Perfect accuracy
        blast_radius: 1.0,
    });

    app.update();

    // Target should be obliterated
    assert!(app.world().get_entity(target_id).is_none() || app.world().get::<Health>(target_id).unwrap().current <= 0.0);
}

#[test]
fn test_bombardment_scatter() {
    let mut app = App::new();
    app.add_systems(Update, execute_bombardment_system);

    let mut grid = TerrainGrid::new(20, 20);
    // Friendly entity at 10,10
    let friendly_id = app.world_mut().spawn((Transform::from_xyz(10.0, 10.0, 0.0), Health { current: 100.0, max: 100.0 })).id();

    app.world_mut().insert_resource(grid);

    // Fire bombardment event at 15,15 but with a huge scatter radius that might hit 10,10
    // We mock RNG in real implementation, but for test we ensure scatter logic is called
    // We can simulate scatter by forcing the event to hit 10,10
    app.world_mut().send_event(BombardmentEvent {
        target: Vec2::new(15.0, 15.0),
        damage: 200.0,
        scatter_radius: 10.0, // Large scatter
        blast_radius: 5.0,
    });

    // We'd ideally mock RNG to guarantee a hit on friendly, but testing the event reception and damage application is key
    // ...
}

#[test]
fn test_bombardment_terrain_deformation() {
    let mut app = App::new();
    app.add_systems(Update, execute_bombardment_system);

    let mut grid = TerrainGrid::new(10, 10);
    grid.set_tile(5, 5, TileType::Grass);
    app.world_mut().insert_resource(grid);

    app.world_mut().send_event(BombardmentEvent {
        target: Vec2::new(5.0, 5.0),
        damage: 1000.0,
        scatter_radius: 0.0,
        blast_radius: 1.0,
    });

    app.update();

    let updated_grid = app.world().resource::<TerrainGrid>();
    // Grass should turn into crater/ash
    assert_eq!(updated_grid.get_tile(5, 5), TileType::Crater);
}

#[test]
fn test_atmosphere_increases_scatter() {
    // Tests that high atmospheric pressure increases the scatter radius of a bombardment command
    // ...
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/bombardment.rs

use bevy::prelude::*;
use rand::Rng;
use crate::layer1::map::{TerrainGrid, TileType};
use crate::layer1::health::Health;
use crate::layer1::environment::AtmosphereGrid;

#[derive(Event)]
pub struct BombardmentEvent {
    pub target: Vec2,
    pub damage: f32,
    pub scatter_radius: f32,
    pub blast_radius: f32,
}

pub fn execute_bombardment_system(
    mut events: EventReader<BombardmentEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Health)>,
    mut grid: ResMut<TerrainGrid>,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        // Calculate actual impact point based on scatter
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(0.0..=event.scatter_radius);
        let impact_point = event.target + Vec2::new(angle.cos() * distance, angle.sin() * distance);

        // Apply damage to entities in blast radius
        for (entity, transform, mut health) in query.iter_mut() {
            let dist_to_impact = transform.translation.truncate().distance(impact_point);
            if dist_to_impact <= event.blast_radius {
                health.current -= event.damage;
                if health.current <= 0.0 {
                    commands.entity(entity).despawn();
                }
            }
        }

        // Deform terrain (minimal implementation: turn center tile to Crater)
        let tx = impact_point.x as usize;
        let ty = impact_point.y as usize;
        if grid.is_in_bounds(tx, ty) {
            grid.set_tile(tx, ty, TileType::Crater);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance:** Calculating distance to every entity in the world for every bombardment could be slow. Use spatial partitioning or query filters to only check entities near the impact point.
- **Code Smells:** Hardcoded `TileType::Crater`. This should ideally consult a terrain reaction matrix (e.g., Water might boil to Steam instead of leaving a crater).
- **API:** The scatter logic should be extracted to a utility function and tested deterministically using a seeded RNG.
- **Integration:** The UI needs a targeting state that shows the predicted scatter radius (which scales with atmospheric density in the target area).

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/bombardment.rs`.
- [ ] `BombardmentEvent` correctly calculates scatter and applies area-of-effect damage.
- [ ] Terrain at the impact point is modified (e.g., converted to `Crater`).

## 7. Technical Guidance
- **Gotchas:** Make sure `AtmosphereGrid` density is sampled at the *target* location, as atmospheric pressure might not be uniform across the map.
- **Seam:** Integrate with `BombardmentEvent` firing from a UI action when a Layer 2 fleet is selected and in orbit.
- **UI:** Add a visual indicator in the Layer 1 view showing the incoming strike and its potential scatter zone.

## 8. Questions
*Builder: add questions here if spec is unclear.*
