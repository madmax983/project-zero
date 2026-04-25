# 1172: Biological Obsolescence

## 1. Overview
The slow, heartbreaking realization that original, organic Pops are no longer competitive. As the colony heavily invests in cybernetics and genetic engineering, "Baseline" (unmodified) Pops become statistically less efficient at every job compared to their augmented peers. Eventually, Baseline Pops begin to suffer chronic "Obsolescence Depression" and demand "Organic Preserves"—inefficient biomes where they can live without technology.

## 2. Dependencies
- Layer 1 `Pop` entity system with Traits/Modifiers.
- `Job` efficiency calculation system.
- `Mood` / `Needs` system.
- Colony-wide `Technology` or `AugmentationLevel` tracker.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_baseline_pop_efficiency_penalty() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, calculate_job_efficiency_system);

    // Set high colony augmentation level
    app.world_mut().insert_resource(ColonyAugmentationLevel(10));

    // Spawn an augmented pop and a baseline pop
    let augmented_pop = app.world_mut().spawn((Pop, Augmented, JobEfficiency(1.0))).id();
    let baseline_pop = app.world_mut().spawn((Pop, Baseline, JobEfficiency(1.0))).id();

    // Act
    app.update();

    // Assert: Baseline pop should have lower efficiency
    let aug_eff = app.world().get::<JobEfficiency>(augmented_pop).unwrap().0;
    let base_eff = app.world().get::<JobEfficiency>(baseline_pop).unwrap().0;
    assert!(base_eff < aug_eff, "Baseline pop should be less efficient than augmented pop in a highly augmented colony");
}

#[test]
fn test_obsolescence_depression_onset() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, evaluate_obsolescence_depression_system);

    // Set high colony augmentation level
    app.world_mut().insert_resource(ColonyAugmentationLevel(20));

    // Spawn a baseline pop
    let baseline_pop = app.world_mut().spawn((Pop, Baseline, Mood(100.0))).id();

    // Act
    app.update();

    // Assert: Pop should gain ObsolescenceDepression
    assert!(app.world().get::<ObsolescenceDepression>(baseline_pop).is_some(), "Baseline pop should suffer from obsolescence depression");
}

#[test]
fn test_organic_preserve_mitigates_depression() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_organic_preserve_aura_system);

    // Spawn an organic preserve
    let preserve_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((Building, OrganicPreserve { radius: 3.0 }, preserve_pos));

    // Spawn a depressed baseline pop within radius
    let baseline_pop = app.world_mut().spawn((
        Pop,
        Baseline,
        ObsolescenceDepression,
        GridPosition { x: 5, y: 6 },
        Mood(50.0)
    )).id();

    // Act
    app.update();

    // Assert: Depression should be mitigated (e.g. component removed or mood boosted)
    assert!(app.world().get::<ObsolescenceDepression>(baseline_pop).is_none(), "Organic Preserve should mitigate obsolescence depression");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Augmented;

#[derive(Component)]
pub struct Baseline;

#[derive(Component)]
pub struct JobEfficiency(pub f32);

#[derive(Component)]
pub struct Mood(pub f32);

#[derive(Component)]
pub struct ObsolescenceDepression;

#[derive(Resource)]
pub struct ColonyAugmentationLevel(pub u32);

#[derive(Component, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn distance(&self, other: &GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct OrganicPreserve {
    pub radius: f32,
}

pub fn calculate_job_efficiency_system(
    aug_level: Res<ColonyAugmentationLevel>,
    mut pops: Query<(&mut JobEfficiency, Option<&Baseline>)>,
) {
    let penalty = (aug_level.0 as f32) * 0.05; // 5% penalty per level
    for (mut eff, is_baseline) in pops.iter_mut() {
        if is_baseline.is_some() {
            eff.0 = (1.0 - penalty).max(0.1);
        } else {
            eff.0 = 1.0 + (aug_level.0 as f32 * 0.02); // slight boost for augmented
        }
    }
}

pub fn evaluate_obsolescence_depression_system(
    mut commands: Commands,
    aug_level: Res<ColonyAugmentationLevel>,
    pops: Query<Entity, (With<Pop>, With<Baseline>, Without<ObsolescenceDepression>)>,
) {
    // Arbitrary threshold for test
    if aug_level.0 >= 15 {
        for entity in pops.iter() {
            commands.entity(entity).insert(ObsolescenceDepression);
        }
    }
}

pub fn apply_organic_preserve_aura_system(
    mut commands: Commands,
    preserves: Query<(&OrganicPreserve, &GridPosition)>,
    pops: Query<(Entity, &GridPosition), With<ObsolescenceDepression>>,
) {
    for (preserve, preserve_pos) in preserves.iter() {
        for (pop_entity, pop_pos) in pops.iter() {
            if preserve_pos.distance(pop_pos) <= preserve.radius {
                commands.entity(pop_entity).remove::<ObsolescenceDepression>();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Gradual Efficiency Loss**: Instead of immediately applying a penalty based on `ColonyAugmentationLevel`, calculate it dynamically per job type (e.g. physical vs mental tasks).
- **Depression Mechanics**: `ObsolescenceDepression` should probably be a gradual mood drain rather than a binary state, eventually leading to refusal to work.
- **Preserve Needs**: `OrganicPreserve` should have massive upkeep costs to reflect the "Tension" in the design doc—it's expensive to maintain a low-tech museum for your people.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Baseline pops suffer efficiency penalties and depression based on colony augmentation levels, mitigable by Organic Preserves.

## 7. Technical Guidance
- Integrate with existing `Pop` generation so new births have a chance to be `Augmented` or `Baseline` based on current edicts/policies.
- Ensure the `OrganicPreserve` is added to the building registry and requires appropriate resources to construct.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
