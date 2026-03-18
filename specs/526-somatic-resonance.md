# Overview

**Somatic Resonance**
**Layer:** 1
**Fantasy:** The colony literally hums with the mood of its people. The physical environment reflects the psychological state of the population.
**Mechanic:** Pops emit a low-level "Resonance" based on their dominant mood. High concentrations of a specific resonance alter the environment slightly—joy makes crops grow marginally faster, despair causes minor electrical glitches, anger increases the flammability of materials.
**Emergence:** A deeply unhappy industrial sector becomes a massive fire hazard not because of the machines, but because the concentrated anger of the workers is literally making the air dry and static-charged.
**Tension:** Grouping pops by job efficiency (creating dangerous emotional monocultures) vs. mixing populations to balance out the resonance.

# Dependencies

- `031-pop-morale.md`
- `033-fire-propagation.md`
- `042-energy-system.md`
- `120-crop-diversity.md`

# RED Phase: Tests First

```rust
// tests/layer1/somatic_resonance_tests.rs

#[test]
fn test_joy_resonance_boosts_crop_growth() {
    let mut app = setup_world();

    // Arrange: Spawns a crop and nearby happy pops
    let crop = app.world_mut().spawn(Crop { growth: 0.0, rate: 1.0 }).id();
    for _ in 0..5 {
        app.world_mut().spawn((
            PopBundle::default(),
            Mood { value: 90 }, // Joyful
            Position { x: 0, y: 0 },
        ));
    }

    // Act
    app.update(); // Resonance system applies buffs
    app.world_mut().resource_mut::<SimulationTime>().tick += 1;
    app.update(); // Crop growth system runs

    // Assert: Growth rate should be > 1.0 due to joy resonance
    let crop_data = app.world().get::<Crop>(crop).unwrap();
    assert!(crop_data.rate > 1.0);
}

#[test]
fn test_anger_resonance_increases_flammability() {
    let mut app = setup_world();

    // Arrange: Spawn an item and nearby angry pops
    let item = app.world_mut().spawn((Flammable { chance: 0.1 }, Position { x: 10, y: 10 })).id();
    for _ in 0..5 {
        app.world_mut().spawn((
            PopBundle::default(),
            Mood { value: 10 }, // Angry/Despair
            Position { x: 10, y: 10 },
        ));
    }

    // Act
    app.update();

    // Assert: Flammability chance should increase
    let flam = app.world().get::<Flammable>(item).unwrap();
    assert!(flam.chance > 0.1);
}
```

# GREEN Phase: Minimal Implementation

```rust
// src/layer1/somatic_resonance.rs

use bevy::prelude::*;
use crate::layer1::pop::Mood;
use crate::layer1::map::Position;
use crate::layer1::farm::Crop;
use crate::layer1::fire::Flammable;

pub fn calculate_somatic_resonance_system(
    pops: Query<(&Mood, &Position)>,
    mut crops: Query<(&mut Crop, &Position)>,
    mut flammables: Query<(&mut Flammable, &Position)>,
) {
    // Collect resonances
    // Simplified for minimal: Check distances (O(N^2) naive approach)

    for (mut crop, crop_pos) in crops.iter_mut() {
        let mut joy_count = 0;
        for (mood, pop_pos) in pops.iter() {
            if crop_pos.distance(pop_pos) < 5.0 && mood.value >= 80 {
                joy_count += 1;
            }
        }
        if joy_count >= 3 {
            crop.rate = 1.2; // 20% boost
        } else {
            crop.rate = 1.0;
        }
    }

    for (mut flam, flam_pos) in flammables.iter_mut() {
        let mut anger_count = 0;
        for (mood, pop_pos) in pops.iter() {
            if flam_pos.distance(pop_pos) < 5.0 && mood.value <= 20 {
                anger_count += 1;
            }
        }

        // Base flammability is assumed 0.1 for this dummy example.
        // Real implementation should store base vs modified.
        if anger_count >= 3 {
            flam.chance = 0.2; // Doubled risk
        } else {
            flam.chance = 0.1;
        }
    }
}
```

# REFACTOR Phase: Quality & Design

- **Performance**: The O(N^2) distance check is bad. We must use a spatial grid (`ResonanceGrid` resource) where pops "paint" their mood onto tiles.
- **Design**: Create a `ResonanceGrid` that diffuses like temperature. Crops and flammables just read the local cell's `Resonance` value rather than looping over all pops.
- **Modifiers**: Use Bevy's component observer pattern or a dedicated `BaseStats` vs `CurrentStats` architecture so we don't permanently alter base `Flammable` chances.

# Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/somatic_resonance.rs`.
- [ ] High concentration of joyful pops accelerates nearby crop growth.
- [ ] High concentration of angry/stressed pops increases local flammability.
- [ ] O(N^2) loop is avoided; uses spatial mapping.

# Technical Guidance

- Implement a `ResonanceMap` resource similar to `ScentMap` or `NoiseMap` (Spec 446 / 283).
- Joy adds positive values, Anger/Despair adds negative values.
- In `farm_system` and `fire_system`, sample the `ResonanceMap` at the entity's position to apply multipliers.

# Questions

*Builder: add questions here if spec is unclear.*
