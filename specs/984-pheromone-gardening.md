# Spec 984: Pheromone Gardening

## 1. Overview
"Pheromone Gardening" introduces biological communication systems that bypass traditional electronics. Alien flora emits pheromones or changes color based on the current state of the colony or environment (e.g., detecting invisible hazards like earthquakes before electronic sensors do). Players can "plant" specific flora, like "Calm-Lilies", near stressful areas (like reactors) to artificially influence Pop moods and reduce unrest.

## 2. Dependencies
- Layer 1 Flora systems (`flora.rs`)
- Layer 1 Olfactory/Pheromone diffusion (`olfactory/mod.rs` or `scent.rs`)
- Layer 1 Pop Needs/Mood simulation (`needs.rs`)
- Layer 1 Hazard/Event detection systems.

## 3. RED Phase: Tests First

```rust
// tests/layer1_pheromone_gardening_tests.rs
use bevy::prelude::*;
use crate::layer1::flora::PheromoneFlora;
use crate::layer1::olfactory::{PheromoneEmission, ScentGrid, emit_flora_pheromones_system};
use crate::layer1::needs::{NeedMorale, apply_pheromone_mood_system};

#[test]
fn test_flora_emits_calming_pheromones() {
    let mut app = App::new();
    app.add_systems(Update, emit_flora_pheromones_system);
    // Setup a dummy ScentGrid
    app.insert_resource(ScentGrid::new(10, 10));

    // Plant a Calm-Lily
    app.world_mut().spawn((
        PheromoneFlora { emission_type: PheromoneEmission::Calming, strength: 5.0 },
        crate::layer1::GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // Verify the grid now contains the calming scent at the flora's position
    let grid = app.world().resource::<ScentGrid>();
    assert!(grid.get_scent(5, 5, PheromoneEmission::Calming) > 0.0);
}

#[test]
fn test_calming_pheromones_boost_morale() {
    let mut app = App::new();
    app.add_systems(Update, apply_pheromone_mood_system);

    let mut grid = ScentGrid::new(10, 10);
    grid.add_scent(5, 5, PheromoneEmission::Calming, 10.0);
    app.insert_resource(grid);

    // Spawn a pop standing in the calming pheromone cloud
    let pop_entity = app.world_mut().spawn((
        crate::layer1::GridPosition { x: 5, y: 5 },
        NeedMorale { value: 30.0, max: 100.0 },
    )).id();

    app.update();

    // Pop morale should have increased due to the calming scent
    let morale = app.world().entity(pop_entity).get::<NeedMorale>().unwrap();
    assert!(morale.value > 30.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/flora/pheromones.rs
use bevy::prelude::*;
use crate::layer1::GridPosition;
use crate::layer1::needs::NeedMorale;
use crate::layer1::olfactory::{ScentGrid, PheromoneEmission};

#[derive(Component)]
pub struct PheromoneFlora {
    pub emission_type: PheromoneEmission,
    pub strength: f32,
}

pub fn emit_flora_pheromones_system(
    mut grid: ResMut<ScentGrid>,
    query: Query<(&PheromoneFlora, &GridPosition)>,
) {
    for (flora, pos) in query.iter() {
        grid.add_scent(pos.x, pos.y, flora.emission_type.clone(), flora.strength);
    }
}

pub fn apply_pheromone_mood_system(
    grid: Res<ScentGrid>,
    mut query: Query<(&GridPosition, &mut NeedMorale)>,
) {
    for (pos, mut morale) in query.iter_mut() {
        let calming_amount = grid.get_scent(pos.x, pos.y, PheromoneEmission::Calming);
        if calming_amount > 0.0 {
            morale.value = (morale.value + calming_amount * 0.1).min(morale.max);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Hazard Detection:** Extend `PheromoneFlora` to react to environmental state changes. For example, introduce a `detect_hazards_system` that changes a specific plant's `emission_type` or visual color to red (Danger) when it detects a nearby `EarthquakeEvent` or `ReactorLeakEvent` before traditional electronic sensors trigger.
- **Scent Diffusion:** Ensure `ScentGrid` correctly diffuses pheromones over time using existing diffusion mechanics, rather than just stacking up infinitely on the origin tile.
- **Maintenance:** Flora requires upkeep (water, suitable temperature) unlike electronic sensors which require power and components. Balance the low-tech advantage with biological vulnerability.

## 6. Acceptance Criteria (Testable!)
- [ ] `cargo test` returns 0 failures, including `test_flora_emits_calming_pheromones`.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] Test coverage ≥85% for the modified olfactory and flora systems.
- [ ] `PheromoneFlora` components correctly emit scents into the `ScentGrid`.
- [ ] Pops standing in tiles with calming scents receive a morale boost over time.

## 7. Technical Guidance
- Integrate with existing `ScentGrid` and olfactory diffusion logic if it already exists (e.g., in `src/layer1/olfactory/`).
- Use the `GridPosition` to map entity locations to the 2D grid structure correctly.
- Be careful with `ScentGrid` initialization in standalone tests to prevent "resource does not exist" panics. Use `app.insert_resource()`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
