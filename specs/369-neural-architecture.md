# 369 Neural Architecture

## 1. Overview
Advanced structures contain semi-organic neural gel. Over time, a building "imprints" on the Pops who use it. A hospital used for trauma victims becomes "haunted" by their stress, subtly degrading the mood of anyone who enters. A factory used by highly skilled workers becomes "optimized," increasing output automatically. This creates compounding benefits or dangerous negative imprints.

## 2. Dependencies
- `066-building-work-ai.md` (for Pops using buildings)
- `084-pop-traits.md` (for Stress and Skill tracking)

## 3. RED Phase: Tests First

```rust
// tests/integration/neural_architecture_test.rs

use crate::layer1::tech::neural_architecture::*;
use crate::layer1::jobs::JobType;
use crate::layer1::needs::Needs;
use bevy::prelude::*;

#[test]
fn test_building_imprints_positive_from_skilled_worker() {
    let mut app = setup_test_app();
    let building = app.world_mut().spawn((
        NeuralArchitecture { imprint: 0.0 },
        JobType::Smith,
    )).id();

    // Skilled worker uses building
    let pop = app.world_mut().spawn((
        Pop,
        JobTenure { job: JobType::Smith, ticks: 1000 },
    )).id();

    app.world_mut().send_event(BuildingUseEvent { building, pop });
    app.update();

    let arch = app.world().get::<NeuralArchitecture>(building).unwrap();
    assert!(arch.imprint > 0.0);
}

#[test]
fn test_building_imprints_negative_from_stressed_worker() {
    let mut app = setup_test_app();
    let building = app.world_mut().spawn((
        NeuralArchitecture { imprint: 0.0 },
        JobType::Hospital,
    )).id();

    // Stressed worker uses building
    let pop = app.world_mut().spawn((
        Pop,
        Needs { stress: 80.0, ..Default::default() },
    )).id();

    app.world_mut().send_event(BuildingUseEvent { building, pop });
    app.update();

    let arch = app.world().get::<NeuralArchitecture>(building).unwrap();
    assert!(arch.imprint < 0.0);
}

#[test]
fn test_building_imprint_affects_future_users() {
    let mut app = setup_test_app();
    let building = app.world_mut().spawn((
        NeuralArchitecture { imprint: -50.0 }, // Negative imprint
        JobType::Hospital,
    )).id();

    let pop = app.world_mut().spawn((
        Pop,
        Needs { stress: 10.0, ..Default::default() },
    )).id();

    app.world_mut().send_event(BuildingUseEvent { building, pop });
    app.update();

    let needs = app.world().get::<Needs>(pop).unwrap();
    assert!(needs.stress > 10.0); // Took stress damage from building
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/neural_architecture.rs
use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::jobs::JobTenure;

#[derive(Component)]
pub struct NeuralArchitecture {
    pub imprint: f32, // -100 to 100
}

#[derive(Event)]
pub struct BuildingUseEvent {
    pub building: Entity,
    pub pop: Entity,
}

pub fn imprint_building_system(
    mut events: EventReader<BuildingUseEvent>,
    mut buildings: Query<&mut NeuralArchitecture>,
    pops: Query<(Option<&JobTenure>, Option<&Needs>)>,
) {
    for event in events.read() {
        if let Ok(mut arch) = buildings.get_mut(event.building) {
            if let Ok((tenure, needs)) = pops.get(event.pop) {

                // Add positive imprint based on skill tenure
                if let Some(t) = tenure {
                    if t.ticks > 500 {
                        arch.imprint = (arch.imprint + 5.0).min(100.0);
                    }
                }

                // Add negative imprint based on stress
                if let Some(n) = needs {
                    if n.stress > 50.0 {
                        arch.imprint = (arch.imprint - 10.0).max(-100.0);
                    }
                }
            }
        }
    }
}

pub fn apply_building_imprint_system(
    mut events: EventReader<BuildingUseEvent>,
    buildings: Query<&NeuralArchitecture>,
    mut pops: Query<&mut Needs>,
) {
    for event in events.read() {
        if let Ok(arch) = buildings.get(event.building) {
            if arch.imprint < -10.0 {
                // Apply stress to anyone who uses a negatively imprinted building
                if let Ok(mut needs) = pops.get_mut(event.pop) {
                    needs.stress += (-arch.imprint * 0.1);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Efficiency Boost:** Implement the positive imprint effect (e.g., speed buff for workers).
- **Decay:** Add a slow normalization decay so buildings revert to 0 if unused.
- **Scrubbing:** Add a mechanism or job type to "scrub" negative imprints at a high cost.

## 6. Acceptance Criteria
- [ ] Tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85%.
- [ ] Skilled pops leave positive imprints on buildings.
- [ ] Stressed pops leave negative imprints on buildings.
- [ ] Negative imprints increase stress of subsequent users.

## 7. Technical Guidance
- `BuildingUseEvent` needs to be triggered whenever a worker initiates a crafting or interaction cycle.
- Add UI feedback so the player can see if a building is cursed.

## 8. Questions
*Builder: Add any questions here.*
