# 1180: State-Sanctioned Delusion

## 1. Overview
When a colony is irreparably failing (e.g., irreversible atmospheric collapse, unstoppable invasion), the player can construct a "Reality Scrubber." It projects a massive AR illusion and alters neurochemistry, making the Pops believe the colony is thriving in a golden age. Productivity maxes out, but the colony is physically dying around them.

## 2. Dependencies
- Layer 1 `Pop` mood, needs, and productivity system.
- `Building` system for the Reality Scrubber.
- Events or states for tracking physical reality vs. perceived reality.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_reality_scrubber_maxes_mood_and_productivity() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_reality_scrubber_delusion_system);

    // Starving, depressed pop with low productivity
    let pop = app.world_mut().spawn((
        Pop,
        Mood(10.0),
        Productivity(0.2),
        Hunger(90.0), // very hungry
    )).id();

    // Reality Scrubber is active
    app.world_mut().insert_resource(RealityScrubberActive(true));

    // Act
    app.update();

    // Assert: Mood and productivity are maxed out despite hunger
    let mood = app.world().get::<Mood>(pop).unwrap().0;
    let prod = app.world().get::<Productivity>(pop).unwrap().0;
    assert_eq!(mood, 100.0, "Reality Scrubber should max out mood");
    assert_eq!(prod, 1.0, "Reality Scrubber should max out productivity");
}

#[test]
fn test_delusion_ignores_negative_environment() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_reality_scrubber_delusion_system);

    let pop = app.world_mut().spawn((
        Pop,
        Mood(50.0),
        Productivity(0.5),
        ToxicEnvironmentExposure, // Usually gives -40 mood
    )).id();

    app.world_mut().insert_resource(RealityScrubberActive(true));

    // Act
    app.update();

    // Assert: Pop is still perfectly happy and productive
    let mood = app.world().get::<Mood>(pop).unwrap().0;
    assert_eq!(mood, 100.0, "Toxic environment should be ignored while deluded");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood(pub f32);

#[derive(Component)]
pub struct Productivity(pub f32);

#[derive(Component)]
pub struct Hunger(pub f32);

#[derive(Component)]
pub struct ToxicEnvironmentExposure;

#[derive(Resource)]
pub struct RealityScrubberActive(pub bool);

pub fn apply_reality_scrubber_delusion_system(
    scrubber_active: Option<Res<RealityScrubberActive>>,
    mut pops: Query<(&mut Mood, &mut Productivity), With<Pop>>,
) {
    if let Some(active) = scrubber_active {
        if active.0 {
            for (mut mood, mut prod) in pops.iter_mut() {
                mood.0 = 100.0;
                prod.0 = 1.0; // 100% productivity
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **System Ordering**: `apply_reality_scrubber_delusion_system` must run *after* all other systems that calculate mood/productivity (like hunger, toxic environment, etc.) so it can override them.
- **Physical Toll**: While the UI says they are happy, the physical simulation (health, hunger) must still decay in the background. The delusion only masks the UI and productivity multipliers. Pops will still die of starvation.
- **Range / Aura**: Instead of a global resource, it might be better as an aura emitted by the `RealityScrubber` building so it only affects nearby pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] The Reality Scrubber forces max mood and productivity, overriding negative environmental or needs-based debuffs.

## 7. Technical Guidance
- The tension is that Pops will work themselves to death while smiling. Ensure that `Health` depletion still occurs and `PopDied` events are still emitted, even if `Mood` is 100.
- This feature is heavily reliant on UI feedback. Consider an effect where the UI becomes unnaturally vibrant or "glitchy" when the scrubber is active.

## 8. Questions
*Builder: add questions here if spec is unclear.*
