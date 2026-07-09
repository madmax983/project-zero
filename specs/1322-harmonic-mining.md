# 1322: Harmonic Mining

## 1. Overview
**Layer:** 1

**Fantasy:** Mining without pickaxes. Singing the stone to dust.

**Mechanic:** "Sonic Drills" disintegrate ore instantly in a radius. However, they vibrate at specific frequencies. If the frequency matches other materials (Glass, Crystal, Bone), those shatter too.

**Emergence:** You tune the drill to mine "Iron". It works great. But your "Glass Greenhouses" vibrate and shatter, venting your crops to vacuum.

**Tension:** Fast, area-of-effect mining vs. Collateral damage risk.

## 2. Dependencies
- ECS (`bevy_ecs`)
- Map/Terrain grid system (`src/layer1/terrain.rs`)
- Building health/integrity system (`src/layer1/architecture/structure.rs`)

## 3. RED Phase: Tests First

```rust
// tests/harmonic_mining_tests.rs
use bevy::prelude::*;

#[test]
fn test_sonic_drill_disintegrates_ore() {
    let mut app = App::new();
    app.add_systems(Update, sonic_drill_system);

    // Add terrain
    let terrain_id = app.world_mut().spawn((
        Terrain { material: Material::IronOre },
        GridPosition { x: 5, y: 5 },
    )).id();

    // Add active drill
    app.world_mut().spawn((
        SonicDrill { frequency: Frequency::Iron, radius: 2.0, active: true },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // Verify terrain changed/destroyed
    let terrain = app.world().get::<Terrain>(terrain_id).unwrap();
    assert_eq!(terrain.material, Material::Dust, "Sonic drill should disintegrate matching ore.");
}

#[test]
fn test_sonic_drill_collateral_damage() {
    let mut app = App::new();
    app.add_systems(Update, sonic_drill_system);

    // Add susceptible structure (Glass greenhouse)
    let greenhouse_id = app.world_mut().spawn((
        Structure { current_hp: 100.0, max_hp: 100.0 },
        ResonantMaterial::Glass,
        GridPosition { x: 6, y: 5 },
    )).id();

    // Add active drill tuned to Iron (which might happen to have a harmonic resonance with Glass in this test setup)
    app.world_mut().spawn((
        SonicDrill { frequency: Frequency::Iron, radius: 2.0, active: true },
        HarmonicResonance { damages: vec![ResonantMaterial::Glass] }, // Iron freq damages glass
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    let structure = app.world().get::<Structure>(greenhouse_id).unwrap();
    assert!(structure.current_hp < 100.0, "Sonic drill should cause collateral damage to harmonically resonant materials.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/harmonic_mining.rs
use bevy::prelude::*;
use crate::layer1::architecture::structure::Structure;

#[derive(Component)]
pub struct SonicDrill {
    pub frequency: Frequency,
    pub radius: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct HarmonicResonance {
    pub damages: Vec<ResonantMaterial>,
}

#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum Frequency {
    Iron,
    Copper,
    Gold,
}

#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum ResonantMaterial {
    Glass,
    Crystal,
    Bone,
}

#[derive(Component)]
pub struct Terrain {
    pub material: Material,
}

#[derive(PartialEq, Debug)]
pub enum Material {
    IronOre,
    CopperOre,
    Dust,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub fn sonic_drill_system(
    mut terrain_query: Query<(&mut Terrain, &GridPosition)>,
    mut structure_query: Query<(&mut Structure, &ResonantMaterial, &GridPosition)>,
    drill_query: Query<(&SonicDrill, Option<&HarmonicResonance>, &GridPosition)>,
) {
    for (drill, resonance, d_pos) in drill_query.iter() {
        if !drill.active { continue; }

        let radius_sq = drill.radius * drill.radius;

        // Disintegrate terrain
        for (mut terrain, t_pos) in terrain_query.iter_mut() {
            let dx = (d_pos.x - t_pos.x) as f32;
            let dy = (d_pos.y - t_pos.y) as f32;
            if dx*dx + dy*dy <= radius_sq {
                if (drill.frequency == Frequency::Iron && terrain.material == Material::IronOre) ||
                   (drill.frequency == Frequency::Copper && terrain.material == Material::CopperOre) {
                       terrain.material = Material::Dust;
                }
            }
        }

        // Collateral damage
        if let Some(res) = resonance {
            for (mut structure, s_mat, s_pos) in structure_query.iter_mut() {
                let dx = (d_pos.x - s_pos.x) as f32;
                let dy = (d_pos.y - s_pos.y) as f32;
                if dx*dx + dy*dy <= radius_sq {
                    if res.damages.contains(s_mat) {
                        structure.current_hp -= 50.0; // Apply damage directly to current_hp
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** Using N^2 distance checks is inefficient. Use `TerrainGrid` to query only tiles within the radius.
- **Harmonics Table:** The relationship between frequencies and collateral materials should probably be a `Resource` rather than attached to every drill component.
- **Yield:** Mining should drop items (e.g., `Item::IronOre`), not just turn the terrain to dust.

## 6. Acceptance Criteria
- [ ] All RED tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Sonic drills destroy target ore within their radius.
- [ ] Sonic drills deal direct damage to `Structure.current_hp` of structures with resonant materials.

## 7. Technical Guidance
- When applying damage to structures, remember that `Structure` tracks `current_hp` directly. There is no `take_damage()` method.
- Be careful with Bevy system tuples. If adding systems to a large group, watch out for the 21-system limit.

## 8. Questions
*Builder: Add any questions here.*
