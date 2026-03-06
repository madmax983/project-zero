# 366 The Bio-Acoustic Choir

## 1. Overview
The colony encounters bioluminescent and sonic plants that emit frequencies. Pops can "tend" to these plants to tune their frequencies. Different frequencies can provide various colony-wide buffs (crop yield, healing rates, work state). However, misaligned frequencies can cause mass migraines or trigger stampedes of local fauna. This feature adds a layer of environmental manipulation and risk-reward tuning.

## 2. Dependencies
- `005-pop-needs.md` (for Mood/Needs tracking)
- `009-job-system.md` (for Tending jobs)
- `092-antagonistic-flora.md` (for dangerous plants)
- `017-designation-system.md` (for targeting plants)

## 3. RED Phase: Tests First

```rust
// tests/integration/bio_acoustic_choir_test.rs

use crate::layer1::needs::Needs;
use crate::layer1::tech::bio_acoustic_choir::*;

#[test]
fn test_choir_plant_tuning_job() {
    let mut app = setup_test_app();
    let plant = app.world_mut().spawn((
        BioAcousticPlant { frequency: 10.0, target_frequency: 10.0 },
        Transform::default(),
    )).id();

    let pop = app.world_mut().spawn((
        Pop,
        JobTenure { job: JobType::Tuner, ticks: 0 },
        Transform::default(),
    )).id();

    // Assign tuning job
    app.world_mut().send_event(TunePlantEvent { plant, pop, new_frequency: 42.0 });
    app.update();

    // Verify plant is tuned
    let plant_comp = app.world().get::<BioAcousticPlant>(plant).unwrap();
    assert_eq!(plant_comp.target_frequency, 42.0);
}

#[test]
fn test_choir_buff_application() {
    let mut app = setup_test_app();
    // Spawn plant with healing frequency
    app.world_mut().spawn((
        BioAcousticPlant { frequency: HealingFrequency::VALUE, target_frequency: HealingFrequency::VALUE },
        Transform::default(),
    ));

    let pop = app.world_mut().spawn((
        Pop,
        Health { current: 50.0, max: 100.0 },
        Transform::default(),
    )).id();

    app.update(); // Run systems

    // Healing should be applied due to the plant's frequency
    let health = app.world().get::<Health>(pop).unwrap();
    assert!(health.current > 50.0);
}

#[test]
fn test_choir_misalignment_penalty() {
    let mut app = setup_test_app();
    // Spawn plant with misaligned/discordant frequency
    app.world_mut().spawn((
        BioAcousticPlant { frequency: DiscordantFrequency::VALUE, target_frequency: DiscordantFrequency::VALUE },
        Transform::default(),
    ));

    let pop = app.world_mut().spawn((
        Pop,
        Needs { stress: 0.0, ..Default::default() },
        Transform::default(),
    )).id();

    app.update();

    // Stress should increase due to discordant frequency
    let needs = app.world().get::<Needs>(pop).unwrap();
    assert!(needs.stress > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/bio_acoustic_choir.rs
use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::health::Health;
use crate::layer1::jobs::JobType;

#[derive(Component)]
pub struct BioAcousticPlant {
    pub frequency: f32,
    pub target_frequency: f32,
}

pub struct HealingFrequency;
impl HealingFrequency {
    pub const VALUE: f32 = 432.0; // Example
}

pub struct DiscordantFrequency;
impl DiscordantFrequency {
    pub const VALUE: f32 = 666.0; // Example
}

#[derive(Event)]
pub struct TunePlantEvent {
    pub plant: Entity,
    pub pop: Entity,
    pub new_frequency: f32,
}

pub fn tune_plant_system(
    mut events: EventReader<TunePlantEvent>,
    mut plants: Query<&mut BioAcousticPlant>,
) {
    for event in events.read() {
        if let Ok(mut plant) = plants.get_mut(event.plant) {
            plant.target_frequency = event.new_frequency;
            // In a real system, frequency would approach target_frequency over time
            plant.frequency = event.new_frequency;
        }
    }
}

pub fn apply_choir_effects_system(
    plants: Query<&BioAcousticPlant>,
    mut pops: Query<(&mut Health, &mut Needs)>,
) {
    let mut has_healing = false;
    let mut has_discordant = false;

    for plant in plants.iter() {
        if (plant.frequency - HealingFrequency::VALUE).abs() < 1.0 {
            has_healing = true;
        }
        if (plant.frequency - DiscordantFrequency::VALUE).abs() < 1.0 {
            has_discordant = true;
        }
    }

    if has_healing || has_discordant {
        for (mut health, mut needs) in pops.iter_mut() {
            if has_healing {
                health.current = (health.current + 5.0).min(health.max);
            }
            if has_discordant {
                needs.stress += 10.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Limits:** Ensure frequencies only affect Pops within a certain radius, rather than globally.
- **Gradual Tuning:** Make the plant's `frequency` approach `target_frequency` gradually instead of instantly snapping.
- **Harmonics:** Implement complex harmonics where multiple plants interacting create synergistic buffs or destructive interference.

## 6. Acceptance Criteria
- [ ] All tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage is >= 85%.
- [ ] Tuning a plant changes its frequency.
- [ ] Healing frequencies increase Pop health.
- [ ] Discordant frequencies increase Pop stress.

## 7. Technical Guidance
- Integrate with `Layer1SystemSet::Observation` or `Layer1SystemSet::Update`.
- Consider using an Event to trigger stampedes when frequencies are critically misaligned.
- Expose the tuning mechanics through the UI so the player can actively set the target frequencies.

## 8. Questions
*Builder: Add any questions here.*
