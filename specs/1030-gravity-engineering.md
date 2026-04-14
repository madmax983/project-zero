# 1030: Gravity Engineering

## 1. Overview
Planets possess a "Max Structure Height" dependent on their Gravity. Exceeding this limit causes upper floors to collapse under their own weight unless constructed using expensive "Reinforced Materials". This forces varied architectural responses: high-G worlds necessitate wide, flat bunkers, while low-G worlds permit needle-spires. Importing a low-G blueprint to a high-G world will result in catastrophic structural failure upon completion.

## 2. Dependencies
- Layer 1 `TerrainGrid` (Z-level or height tracking for buildings).
- Layer 1 `Construction` system.
- Layer 2 `Planet` generation (Gravity parameter).
- Layer 1 `Material`/`Resource` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::buildings::{Building, ConstructionEvent, Height, Material};
    use crate::layer1::terrain::{PlanetEnvironment, Gravity};
    use crate::layer1::health::{Health, DamageEvent};

    #[test]
    fn test_building_exceeding_max_height_takes_structural_damage() {
        let mut app = App::new();
        app.insert_resource(PlanetEnvironment { gravity: 2.0 }); // High-G
        app.add_event::<ConstructionEvent>();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let tall_building = app.world_mut().spawn((
            Building,
            Height { floors: 5 }, // Too tall for High-G standard materials
            Material { is_reinforced: false },
        )).id();

        app.world_mut().resource_mut::<Events<ConstructionEvent>>().send(ConstructionEvent {
            building: tall_building,
        });

        app.update();

        let damage_events = app.world().resource::<Events<DamageEvent>>();
        let mut reader = damage_events.get_reader();
        let mut found = false;
        for event in reader.read(damage_events) {
            if event.target == tall_building {
                found = true;
            }
        }

        assert!(found, "A building exceeding the gravity height limit should suffer structural damage.");
    }

    #[test]
    fn test_reinforced_materials_bypass_height_limits() {
        let mut app = App::new();
        app.insert_resource(PlanetEnvironment { gravity: 2.0 });
        app.add_event::<ConstructionEvent>();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let reinforced_building = app.world_mut().spawn((
            Building,
            Height { floors: 10 },
            Material { is_reinforced: true },
        )).id();

        app.world_mut().resource_mut::<Events<ConstructionEvent>>().send(ConstructionEvent {
            building: reinforced_building,
        });

        app.update();

        let damage_events = app.world().resource::<Events<DamageEvent>>();
        assert_eq!(damage_events.get_reader().len(damage_events), 0, "Reinforced buildings should ignore gravity height limits.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/gravity_engineering.rs
use bevy::prelude::*;
use crate::layer1::buildings::{Building, ConstructionEvent, Height, Material};
use crate::layer1::terrain::{PlanetEnvironment, Gravity};
use crate::layer1::health::DamageEvent;

pub fn evaluate_structural_integrity_system(
    planet: Res<PlanetEnvironment>,
    mut construction_events: EventReader<ConstructionEvent>,
    query: Query<(&Height, &Material)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    // Inverse relationship: High gravity = lower max height
    let base_max_height = (10.0 / planet.gravity).floor() as u32;

    for event in construction_events.read() {
        if let Ok((height, material)) = query.get(event.building) {
            if !material.is_reinforced && height.floors > base_max_height {
                // Building is too tall for this gravity
                damage_events.send(DamageEvent {
                    target: event.building,
                    amount: 1000.0, // Catastrophic collapse for MVP
                    source: "Gravity Collapse".to_string(),
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Progressive Collapse:** Instead of instantly destroying the whole building, the system should only destroy the floors *above* the `base_max_height`. This requires modifying the `TerrainGrid` Z-levels.
- **UI Warning:** The building placement/blueprint UI needs to calculate the `base_max_height` and warn the player *before* they waste resources constructing an impossible tower.
- **Variable Gravity:** Asteroids might have virtually no gravity limit, making them ideal for massive sprawling orbital stations but terrible for un-tethered Pops.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_building_exceeding_max_height_takes_structural_damage` passes.
- [ ] Test `test_reinforced_materials_bypass_height_limits` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- The calculation `(10.0 / planet.gravity)` is a placeholder. Adjust it according to the actual scale of Layer 1 tiles and Z-levels.
- Ensure `Material` or an equivalent component tracks what the building was actually constructed with, differentiating standard steel from reinforced alloys.

## 8. Questions
*Builder: add questions here if spec is unclear.*
