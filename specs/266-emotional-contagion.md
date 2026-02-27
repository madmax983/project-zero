# Emotional Contagion

## 1. Overview
The **Emotional Contagion** system simulates how extreme moods spread through a crowd on Layer 1. When a Pop experiences intense emotion (e.g., Panic, Joy, Rage), they emit an "emotional aura" that affects nearby Pops. This creates emergent crowd behaviors where a single terrified individual can cause a localized panic, or a highly joyful Pop can elevate the mood of an entire room.

## 2. Dependencies
- `004` Pop Entity
- `031` Pop Morale
- `016` Utility AI System (for mood evaluation)

## 3. RED Phase: Tests First

```rust
// tests/emotional_contagion_tests.rs

use bevy::prelude::*;
use scale::layer1::morale::{Morale, MoodModifier};
use scale::layer1::contagion::{EmotionalContagion, ContagionSystem, ContagionType};

#[test]
fn test_contagion_spreads_to_nearby_pops() {
    let mut app = App::new();
    app.add_systems(Update, ContagionSystem);

    // Arrange: Two pops, one panicking, one neutral, within radius
    let source = app.world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Morale { current: 10.0, ..default() }, // Very low morale
        EmotionalContagion {
            contagion_type: ContagionType::Panic,
            radius: 5.0,
            strength: -15.0,
        },
    )).id();

    let target = app.world.spawn((
        Transform::from_xyz(3.0, 0.0, 0.0),
        Morale { current: 50.0, ..default() },
    )).id();

    // Act
    app.update();

    // Assert: Target should receive a negative mood modifier from contagion
    let target_morale = app.world.get::<Morale>(target).unwrap();
    assert!(
        target_morale.modifiers.iter().any(|m| m.source == "Contagion: Panic"),
        "Target did not receive Panic contagion modifier"
    );
}

#[test]
fn test_contagion_ignores_distant_pops() {
    let mut app = App::new();
    app.add_systems(Update, ContagionSystem);

    // Arrange: Target is outside the 5.0 radius
    let source = app.world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        EmotionalContagion {
            contagion_type: ContagionType::Joy,
            radius: 5.0,
            strength: 10.0,
        },
    )).id();

    let target = app.world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        Morale { current: 50.0, ..default() },
    )).id();

    // Act
    app.update();

    // Assert: Target should NOT receive a mood modifier
    let target_morale = app.world.get::<Morale>(target).unwrap();
    assert!(
        !target_morale.modifiers.iter().any(|m| m.source == "Contagion: Joy"),
        "Target incorrectly received contagion outside radius"
    );
}

#[test]
fn test_contagion_stacking_limits() {
    // Tests that a pop doesn't get infinitely depressed by standing next to a panicking pop for a long time.
    // Ensure the modifier resets or caps.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/contagion.rs

use bevy::prelude::*;
use crate::layer1::morale::{Morale, MoodModifier};

#[derive(Component)]
pub struct EmotionalContagion {
    pub contagion_type: ContagionType,
    pub radius: f32,
    pub strength: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContagionType {
    Panic,
    Joy,
    Rage,
}

pub fn contagion_system(
    sources: Query<(&Transform, &EmotionalContagion)>,
    mut targets: Query<(&Transform, &mut Morale), Without<EmotionalContagion>>,
) {
    for (source_transform, contagion) in sources.iter() {
        for (target_transform, mut target_morale) in targets.iter_mut() {
            let distance = source_transform.translation.distance(target_transform.translation);

            if distance <= contagion.radius {
                let modifier_name = match contagion.contagion_type {
                    ContagionType::Panic => "Contagion: Panic",
                    ContagionType::Joy => "Contagion: Joy",
                    ContagionType::Rage => "Contagion: Rage",
                };

                // Add or refresh modifier
                if !target_morale.modifiers.iter().any(|m| m.source == modifier_name) {
                    target_morale.modifiers.push(MoodModifier {
                        source: modifier_name.to_string(),
                        value: contagion.strength,
                        duration: 100, // Minimal fixed duration
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Spatial Partitioning:** The O(N^2) check between sources and targets will not scale well with a large population. Refactor to use a spatial hash or grid-based lookup for nearby Pops.
- **Modifier Falloff:** Instead of a hard radius, implement a distance-based falloff for contagion strength (e.g., closer = stronger effect).
- **Line of Sight/Acoustics Integration:** Contagion shouldn't spread through solid walls. Consider integrating with `Acoustic Shadows` (Spec 258) or simple line-of-sight checks.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `contagion.rs`.
- [ ] Panicking Pops successfully apply a temporary negative morale modifier to nearby Pops.

## 7. Technical Guidance
- **Triggering Contagion:** Contagion components shouldn't be permanent. Add a system that attaches the `EmotionalContagion` component when a Pop hits extreme morale thresholds (e.g., Morale < 15 triggers Panic) and removes it when they recover.
- **Decay/Stacking:** Be careful with modifier stacking. The minimal implementation just prevents duplicates, but consider how multiple panicking Pops nearby might amplify the effect up to a reasonable cap.
- **Integration:** Hook this into the `Utility AI System` so that panicking pops might choose to run away (flee), further spreading the contagion, instead of just standing still.

## 8. Questions
*Builder: add questions here if spec is unclear.*